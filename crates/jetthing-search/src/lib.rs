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
    pub line: usize,
    pub column: usize,
    pub preview: String,
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
                        let lowercase = line.to_lowercase();
                        let mut offset = 0;
                        while let Some(found) = lowercase[offset..].find(&needle) {
                            let byte_column = offset + found;
                            let column = line[..byte_column].chars().count() + 1;
                            let event = SearchEvent::Match(TextMatch {
                                path: relative_path.clone(),
                                line: line_index + 1,
                                column,
                                preview: line.trim().chars().take(240).collect(),
                            });
                            if sender.send(event).is_err() {
                                return;
                            }
                            result_count += 1;
                            if result_count >= MAX_RESULTS {
                                truncated = true;
                                break;
                            }
                            offset = byte_column + needle.len();
                        }
                        if truncated {
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
