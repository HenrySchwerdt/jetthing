use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver},
    },
    thread,
};

const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_RESULTS: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextMatch {
    pub path: PathBuf,
    /// 1-based line of the match.
    pub line: usize,
    /// 1-based character column of the first occurrence on the line.
    pub column: usize,
    /// Length in characters of the first occurrence.
    pub length: usize,
    /// The trimmed line, at most [`PREVIEW_CHARS`] characters.
    pub preview: String,
    /// Character ranges of every occurrence within `preview`.
    pub highlights: Vec<std::ops::Range<usize>>,
}

/// Longest preview kept per result line.
pub const PREVIEW_CHARS: usize = 240;

/// Case-insensitive occurrences of `needle` (already lowercased) in `line`, as
/// character ranges of `line`. Lowercasing can change a character's byte length
/// (e.g. `İ`), so matches are mapped back per character instead of by byte offset.
pub fn find_occurrences(line: &str, needle: &str) -> Vec<std::ops::Range<usize>> {
    if needle.is_empty() {
        return Vec::new();
    }
    let mut lowered = String::with_capacity(line.len());
    // For each byte of `lowered`, the index of the source character.
    let mut source = Vec::with_capacity(line.len());
    for (index, ch) in line.chars().enumerate() {
        for lower in ch.to_lowercase() {
            lowered.push(lower);
            source.extend(std::iter::repeat_n(index, lower.len_utf8()));
        }
    }
    let mut ranges: Vec<std::ops::Range<usize>> = Vec::new();
    let mut offset = 0;
    while let Some(found) = lowered[offset..].find(needle) {
        let start_byte = offset + found;
        let end_byte = start_byte + needle.len();
        let start = source[start_byte];
        let end = source[end_byte - 1] + 1;
        if ranges.last().is_none_or(|last| last.end <= start) {
            ranges.push(start..end);
        }
        offset = end_byte;
    }
    ranges
}

/// Builds the result row for one line, or `None` when nothing matches.
pub fn line_match(path: &Path, line_index: usize, line: &str, needle: &str) -> Option<TextMatch> {
    let ranges = find_occurrences(line, needle);
    let first = ranges.first()?.clone();
    let leading = line.chars().take_while(|c| c.is_whitespace()).count();
    let preview: String = line.trim().chars().take(PREVIEW_CHARS).collect();
    let preview_len = preview.chars().count();
    let highlights = ranges
        .iter()
        .filter_map(|range| {
            let start = range.start.saturating_sub(leading).min(preview_len);
            let end = range.end.saturating_sub(leading).min(preview_len);
            (start < end).then_some(start..end)
        })
        .collect();
    Some(TextMatch {
        path: path.to_owned(),
        line: line_index + 1,
        column: first.start + 1,
        length: first.end - first.start,
        preview,
        highlights,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchEvent {
    Match(TextMatch),
    Finished {
        job: u64,
        files_scanned: usize,
        truncated: bool,
    },
}

pub struct SearchService {
    generation: Arc<AtomicU64>,
    receiver: Option<Receiver<SearchEvent>>,
    active_job: Option<u64>,
}

impl Default for SearchService {
    fn default() -> Self {
        Self {
            generation: Arc::new(AtomicU64::new(0)),
            receiver: None,
            active_job: None,
        }
    }
}

impl SearchService {
    pub fn start(
        &mut self,
        root: PathBuf,
        files: Vec<PathBuf>,
        query: String,
    ) -> Result<u64, std::io::Error> {
        let job = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let generation = Arc::clone(&self.generation);
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        self.active_job = Some(job);
        let spawn_result = thread::Builder::new()
            .name(format!("jetthing-search-{job}"))
            .spawn(move || {
                let needle = query.to_lowercase();
                if needle.is_empty() {
                    let _ = sender.send(SearchEvent::Finished {
                        job,
                        files_scanned: 0,
                        truncated: false,
                    });
                    return;
                }
                let mut files_scanned = 0;
                let mut result_count = 0;
                let mut truncated = false;
                for relative_path in files {
                    if generation.load(Ordering::SeqCst) != job {
                        return;
                    }
                    let path = root.join(&relative_path);
                    let Ok(metadata) = fs::metadata(&path) else {
                        continue;
                    };
                    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
                        continue;
                    }
                    let Ok(contents) = fs::read_to_string(path) else {
                        continue;
                    };
                    files_scanned += 1;
                    for (line_index, line) in contents.lines().enumerate() {
                        if generation.load(Ordering::Relaxed) != job {
                            return;
                        }
                        // One row per line, however many occurrences it has.
                        let Some(found) = line_match(&relative_path, line_index, line, &needle)
                        else {
                            continue;
                        };
                        if sender.send(SearchEvent::Match(found)).is_err() {
                            return;
                        }
                        result_count += 1;
                        if result_count >= MAX_RESULTS {
                            truncated = true;
                            break;
                        }
                    }
                    if truncated {
                        break;
                    }
                }
                let _ = sender.send(SearchEvent::Finished {
                    job,
                    files_scanned,
                    truncated,
                });
            });
        if let Err(error) = spawn_result {
            self.receiver = None;
            self.active_job = None;
            return Err(error);
        }
        Ok(job)
    }

    pub fn cancel(&mut self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.receiver = None;
        self.active_job = None;
    }

    pub fn poll(&mut self) -> Vec<SearchEvent> {
        let Some(receiver) = &self.receiver else {
            return Vec::new();
        };
        let events: Vec<_> = receiver.try_iter().collect();
        if events
            .iter()
            .any(|event| matches!(event, SearchEvent::Finished { .. }))
        {
            self.active_job = None;
        }
        events
    }

    pub fn is_running(&self) -> bool {
        self.active_job.is_some()
    }
}

impl Drop for SearchService {
    fn drop(&mut self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
    }
}

pub fn searchable_paths(root: &Path, paths: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    paths
        .into_iter()
        .filter(|path| root.join(path).is_file())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn streams_case_insensitive_matches_and_completion() {
        let root = std::env::temp_dir().join(format!("jetthing-search-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("one.rs"), "fn JetThing() {}\n// jetthing\n").unwrap();
        let mut service = SearchService::default();
        service
            .start(
                root.clone(),
                vec![PathBuf::from("one.rs")],
                "jetthing".into(),
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(service.poll());
            if !service.is_running() {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, SearchEvent::Match(_)))
                .count(),
            2
        );
        assert!(events.iter().any(|event| matches!(
            event,
            SearchEvent::Finished {
                files_scanned: 1,
                ..
            }
        )));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_line_with_two_hits_is_one_row_with_two_highlights() {
        let found = line_match(
            Path::new("a.kt"),
            4,
            "    val greeting = greet(name)",
            "greet",
        )
        .unwrap();
        assert_eq!(found.line, 5);
        assert_eq!(found.column, 9, "first hit, 1-based, before trimming");
        assert_eq!(found.length, 5);
        assert_eq!(found.preview, "val greeting = greet(name)");
        assert_eq!(found.highlights, vec![4..9, 15..20]);
        assert!(line_match(Path::new("a.kt"), 0, "nothing here", "greet").is_none());
    }

    #[test]
    fn case_folding_that_changes_byte_length_does_not_panic() {
        // 'İ' lowercases to two characters; the match after it must map back.
        assert_eq!(find_occurrences("İx jet JET", "jet"), vec![3..6, 7..10]);
        assert_eq!(
            find_occurrences("ẞtraße", "ss"),
            Vec::<std::ops::Range<usize>>::new()
        );
        assert_eq!(find_occurrences("aaaa", "aa"), vec![0..2, 2..4]);
    }

    #[test]
    fn new_search_cancels_previous_generation() {
        let mut service = SearchService::default();
        let first = service
            .start(PathBuf::from("/tmp"), Vec::new(), "one".into())
            .unwrap();
        let second = service
            .start(PathBuf::from("/tmp"), Vec::new(), "two".into())
            .unwrap();
        assert!(second > first);
    }
}
