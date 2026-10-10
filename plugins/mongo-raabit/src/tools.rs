//! State and background operations of the tool tabs: Aggregation Editor, SQL Query,
//! Schema Explorer and Index Manager.

use crate::app::{Load, MongoRaabit, ResultView, SessionId, Tab, TabId, TabKind};
use mongo_raabit_core::Connection;
use mongo_raabit_core::aggregate::{Pipeline, RunOptions, Stage, StagePreview};
use mongo_raabit_core::bson::Document;
use mongo_raabit_core::connection::{IndexInfo, NewIndex};
use mongo_raabit_core::schema::{SampleMethod, SchemaReport};
use mongo_raabit_core::shell;
use mongo_raabit_core::sql::{self, SqlTranslation};
use std::collections::HashSet;
use std::time::Duration;

/// Results of a pipeline or SQL run.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RunResult {
    pub documents: Vec<Document>,
    pub elapsed: Duration,
}

/// Maximum documents fetched by "Run" in the aggregation and SQL tabs.
pub const RUN_LIMIT: u64 = 500;

pub struct AggregationTab {
    pub session: SessionId,
    pub db: String,
    pub collection: String,
    pub pipeline: Pipeline,
    pub selected: usize,
    pub preview: Load<StagePreview>,
    pub preview_seq: u64,
    pub preview_limit: u64,
    /// Input sample size for previews (0 = whole collection).
    pub sample: u64,
    pub result: Load<RunResult>,
    pub view: ResultView,
    pub open: HashSet<String>,
    pub stage_error: Option<String>,
}

pub struct SqlTab {
    pub session: SessionId,
    pub db: String,
    pub sql: String,
    pub result: Load<RunResult>,
    pub view: ResultView,
    pub open: HashSet<String>,
}

impl SqlTab {
    /// Live translation of the current SQL text.
    pub fn translation(&self) -> Result<SqlTranslation, String> {
        sql::translate(&self.sql).map_err(|e| e.to_string())
    }
}

pub struct SchemaTab {
    pub session: SessionId,
    pub db: String,
    pub collection: String,
    pub size: u64,
    pub method: SampleMethod,
    pub report: Load<SchemaReport>,
    pub selected: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndexForm {
    pub keys: String,
    pub name: String,
    pub unique: bool,
    pub sparse: bool,
    pub hidden: bool,
    pub ttl: String,
    pub partial: String,
}

impl IndexForm {
    pub fn to_new_index(&self) -> Result<NewIndex, String> {
        let keys = shell::parse_document(&self.keys).map_err(|e| format!("keys: {e}"))?;
        if keys.is_empty() {
            return Err("add at least one key, e.g. { email: 1 }".into());
        }
        let partial =
            shell::parse_document(&self.partial).map_err(|e| format!("partial filter: {e}"))?;
        let ttl = if self.ttl.trim().is_empty() {
            None
        } else {
            Some(
                self.ttl
                    .trim()
                    .parse::<u64>()
                    .map_err(|_| "TTL must be seconds".to_owned())?,
            )
        };
        Ok(NewIndex {
            keys,
            name: Some(self.name.trim().to_owned()).filter(|n| !n.is_empty()),
            unique: self.unique,
            sparse: self.sparse,
            hidden: self.hidden,
            ttl_seconds: ttl,
            partial_filter: (!partial.is_empty()).then_some(partial),
        })
    }
}

pub struct IndexTab {
    pub session: SessionId,
    pub db: String,
    pub collection: String,
    pub indexes: Load<Vec<IndexInfo>>,
    pub form: IndexForm,
    pub form_open: bool,
    pub error: Option<String>,
    pub busy: bool,
    /// Index awaiting a second click to drop.
    pub confirm_drop: Option<String>,
}

impl MongoRaabit {
    fn connection_for(&self, session: SessionId) -> Option<Connection> {
        self.session(session)?.connection().cloned()
    }

    fn push_tab(&mut self, kind: TabKind) -> TabId {
        let id = self.next_id();
        self.tabs.push(Tab { id, kind });
        self.active_tab = self.tabs.len() - 1;
        id
    }

    fn tab_kind_mut(&mut self, id: TabId) -> Option<&mut TabKind> {
        self.tab_mut(id).map(|t| &mut t.kind)
    }

    // ----- aggregation ------------------------------------------------------------------

    pub fn open_aggregation(
        &mut self,
        session: SessionId,
        db: &str,
        collection: &str,
        pipeline: Option<Pipeline>,
    ) -> TabId {
        let pipeline = pipeline.unwrap_or_else(|| Pipeline {
            stages: vec![Stage::new("$match")],
        });
        let id = self.push_tab(TabKind::Aggregation(Box::new(AggregationTab {
            session,
            db: db.to_owned(),
            collection: collection.to_owned(),
            pipeline,
            selected: 0,
            preview: Load::Idle,
            preview_seq: 0,
            preview_limit: 20,
            sample: 1000,
            result: Load::Idle,
            view: ResultView::Tree,
            open: HashSet::new(),
            stage_error: None,
        })));
        self.preview_stage(id);
        id
    }

    pub fn aggregation_tab_mut(&mut self, id: TabId) -> Option<&mut AggregationTab> {
        match self.tab_kind_mut(id)? {
            TabKind::Aggregation(tab) => Some(tab),
            _ => None,
        }
    }

    /// Recompute the input/output preview of the selected stage.
    pub fn preview_stage(&mut self, tab_id: TabId) {
        let Some(tab) = self.aggregation_tab_mut(tab_id) else {
            return;
        };
        if tab.pipeline.stages.is_empty() {
            tab.preview = Load::Idle;
            return;
        }
        tab.selected = tab.selected.min(tab.pipeline.stages.len() - 1);
        if let Err(err) = tab.pipeline.documents(Some(tab.selected)) {
            tab.stage_error = Some(err.to_string());
            return;
        }
        tab.stage_error = None;
        tab.preview_seq += 1;
        tab.preview = Load::Loading;
        let (seq, session, db, coll, pipeline, index, limit, sample) = (
            tab.preview_seq,
            tab.session,
            tab.db.clone(),
            tab.collection.clone(),
            tab.pipeline.clone(),
            tab.selected,
            tab.preview_limit,
            tab.sample,
        );
        let Some(conn) = self.connection_for(session) else {
            return;
        };
        let options = RunOptions {
            input_sample: sample,
            ..RunOptions::default()
        };
        self.jobs.spawn(
            async move {
                conn.preview_stage(&db, &coll, &pipeline, index, limit, options)
                    .await
            },
            move |app: &mut MongoRaabit, result| {
                if let Some(tab) = app.aggregation_tab_mut(tab_id)
                    && tab.preview_seq == seq
                {
                    tab.preview = match result {
                        Ok(preview) => Load::Ready(preview),
                        Err(err) => Load::Failed(err.to_string()),
                    };
                }
            },
        );
    }

    /// Run the whole pipeline (writing stages included) and show up to [`RUN_LIMIT`] results.
    pub fn run_pipeline(&mut self, tab_id: TabId) {
        let Some(tab) = self.aggregation_tab_mut(tab_id) else {
            return;
        };
        let stages = match tab.pipeline.documents(None) {
            Ok(stages) => stages,
            Err(err) => {
                tab.stage_error = Some(err.to_string());
                return;
            }
        };
        tab.result = Load::Loading;
        let (session, db, coll) = (tab.session, tab.db.clone(), tab.collection.clone());
        let Some(conn) = self.connection_for(session) else {
            return;
        };
        self.jobs.spawn(
            async move {
                conn.aggregate(&db, &coll, stages, RunOptions::default(), RUN_LIMIT)
                    .await
            },
            move |app: &mut MongoRaabit, result| {
                if let Some(tab) = app.aggregation_tab_mut(tab_id) {
                    tab.result = match result {
                        Ok((documents, elapsed)) => Load::Ready(RunResult { documents, elapsed }),
                        Err(err) => Load::Failed(err.to_string()),
                    };
                }
            },
        );
    }

    // ----- SQL --------------------------------------------------------------------------

    pub fn open_sql(&mut self, session: SessionId, db: &str, collection: Option<&str>) -> TabId {
        let sql = match collection {
            Some(coll) => format!("SELECT *\nFROM {coll}\nLIMIT 50"),
            None => "SELECT *\nFROM collection\nLIMIT 50".into(),
        };
        self.push_tab(TabKind::Sql(Box::new(SqlTab {
            session,
            db: db.to_owned(),
            sql,
            result: Load::Idle,
            view: ResultView::Tree,
            open: HashSet::new(),
        })))
    }

    pub fn sql_tab_mut(&mut self, id: TabId) -> Option<&mut SqlTab> {
        match self.tab_kind_mut(id)? {
            TabKind::Sql(tab) => Some(tab),
            _ => None,
        }
    }

    pub fn run_sql(&mut self, tab_id: TabId) {
        let Some(tab) = self.sql_tab_mut(tab_id) else {
            return;
        };
        let translation = match tab.translation() {
            Ok(t) => t,
            Err(err) => {
                tab.result = Load::Failed(err);
                return;
            }
        };
        tab.result = Load::Loading;
        let (session, db) = (tab.session, tab.db.clone());
        let Some(conn) = self.connection_for(session) else {
            return;
        };
        let collection = translation.collection().to_owned();
        let pipeline = translation.pipeline();
        self.jobs.spawn(
            async move {
                conn.aggregate(&db, &collection, pipeline, RunOptions::default(), RUN_LIMIT)
                    .await
            },
            move |app: &mut MongoRaabit, result| {
                if let Some(tab) = app.sql_tab_mut(tab_id) {
                    tab.result = match result {
                        Ok((documents, elapsed)) => Load::Ready(RunResult { documents, elapsed }),
                        Err(err) => Load::Failed(err.to_string()),
                    };
                }
            },
        );
    }

    // ----- schema -----------------------------------------------------------------------

    pub fn open_schema(&mut self, session: SessionId, db: &str, collection: &str) -> TabId {
        let id = self.push_tab(TabKind::Schema(Box::new(SchemaTab {
            session,
            db: db.to_owned(),
            collection: collection.to_owned(),
            size: 1000,
            method: SampleMethod::Random,
            report: Load::Idle,
            selected: None,
        })));
        self.analyze_schema(id);
        id
    }

    pub fn schema_tab_mut(&mut self, id: TabId) -> Option<&mut SchemaTab> {
        match self.tab_kind_mut(id)? {
            TabKind::Schema(tab) => Some(tab),
            _ => None,
        }
    }

    pub fn analyze_schema(&mut self, tab_id: TabId) {
        let Some(tab) = self.schema_tab_mut(tab_id) else {
            return;
        };
        tab.report = Load::Loading;
        let (session, db, coll, size, method) = (
            tab.session,
            tab.db.clone(),
            tab.collection.clone(),
            tab.size,
            tab.method,
        );
        let Some(conn) = self.connection_for(session) else {
            return;
        };
        self.jobs.spawn(
            async move {
                conn.sample_schema(&db, &coll, size, method, Document::new())
                    .await
            },
            move |app: &mut MongoRaabit, result| {
                if let Some(tab) = app.schema_tab_mut(tab_id) {
                    tab.report = match result {
                        Ok(report) => {
                            tab.selected = report
                                .fields
                                .iter()
                                .position(|f| f.histogram.len() > 1 || f.top_values.len() > 1);
                            Load::Ready(report)
                        }
                        Err(err) => Load::Failed(err.to_string()),
                    };
                }
            },
        );
    }

    // ----- indexes ----------------------------------------------------------------------

    pub fn open_indexes(&mut self, session: SessionId, db: &str, collection: &str) -> TabId {
        let id = self.push_tab(TabKind::Indexes(Box::new(IndexTab {
            session,
            db: db.to_owned(),
            collection: collection.to_owned(),
            indexes: Load::Idle,
            form: IndexForm {
                keys: "{  : 1 }".into(),
                ..IndexForm::default()
            },
            form_open: false,
            error: None,
            busy: false,
            confirm_drop: None,
        })));
        self.load_indexes(id);
        id
    }

    pub fn index_tab_mut(&mut self, id: TabId) -> Option<&mut IndexTab> {
        match self.tab_kind_mut(id)? {
            TabKind::Indexes(tab) => Some(tab),
            _ => None,
        }
    }

    pub fn load_indexes(&mut self, tab_id: TabId) {
        let Some(tab) = self.index_tab_mut(tab_id) else {
            return;
        };
        tab.indexes = Load::Loading;
        let (session, db, coll) = (tab.session, tab.db.clone(), tab.collection.clone());
        let Some(conn) = self.connection_for(session) else {
            return;
        };
        self.jobs.spawn(
            async move { conn.indexes(&db, &coll).await },
            move |app: &mut MongoRaabit, result| {
                if let Some(tab) = app.index_tab_mut(tab_id) {
                    tab.indexes = match result {
                        Ok(list) => Load::Ready(list),
                        Err(err) => Load::Failed(err.to_string()),
                    };
                }
            },
        );
    }

    /// Run an index change, then reload the list.
    fn index_operation<F>(
        &mut self,
        tab_id: TabId,
        done: String,
        op: impl FnOnce(Connection, String, String) -> F + Send + 'static,
    ) where
        F: std::future::Future<Output = mongo_raabit_core::Result<()>> + Send + 'static,
    {
        let Some(tab) = self.index_tab_mut(tab_id) else {
            return;
        };
        tab.busy = true;
        tab.error = None;
        let (session, db, coll) = (tab.session, tab.db.clone(), tab.collection.clone());
        let Some(conn) = self.connection_for(session) else {
            return;
        };
        self.jobs
            .spawn(op(conn, db, coll), move |app: &mut MongoRaabit, result| {
                if let Some(tab) = app.index_tab_mut(tab_id) {
                    tab.busy = false;
                    match result {
                        Ok(()) => {
                            tab.form_open = false;
                            app.status = done;
                            app.load_indexes(tab_id);
                        }
                        Err(err) => tab.error = Some(err.to_string()),
                    }
                }
            });
    }

    pub fn create_index(&mut self, tab_id: TabId) {
        let Some(tab) = self.index_tab_mut(tab_id) else {
            return;
        };
        let index = match tab.form.to_new_index() {
            Ok(index) => index,
            Err(err) => {
                tab.error = Some(err);
                return;
            }
        };
        self.index_operation(tab_id, "Index created".into(), move |conn, db, coll| async move {
            conn.create_index(&db, &coll, index).await.map(|_| ())
        });
    }

    pub fn drop_index(&mut self, tab_id: TabId, name: String) {
        self.index_operation(
            tab_id,
            format!("Dropped index {name}"),
            move |conn, db, coll| async move { conn.drop_index(&db, &coll, &name).await },
        );
    }

    pub fn set_index_hidden(&mut self, tab_id: TabId, name: String, hidden: bool) {
        let verb = if hidden { "Hid" } else { "Unhid" };
        self.index_operation(tab_id, format!("{verb} index {name}"), move |conn, db, coll| async move {
            conn.set_index_hidden(&db, &coll, &name, hidden).await
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_form_validation() {
        let mut form = IndexForm {
            keys: "{ email: 1, 'address.city': -1 }".into(),
            ttl: "3600".into(),
            partial: "{ active: true }".into(),
            ..IndexForm::default()
        };
        let index = form.to_new_index().unwrap();
        assert_eq!(index.keys.len(), 2);
        assert_eq!(index.ttl_seconds, Some(3600));
        assert!(index.partial_filter.is_some());
        form.keys = "{}".into();
        assert!(form.to_new_index().is_err());
        form.keys = "{a: 1}".into();
        form.ttl = "soon".into();
        assert!(form.to_new_index().is_err());
    }
}

// ----- shell ------------------------------------------------------------------------------

/// How the shell tab evaluates scripts.
#[derive(Debug, Clone, PartialEq)]
pub enum ShellEngine {
    Detecting,
    Mongosh {
        path: std::path::PathBuf,
        version: String,
    },
    /// mongosh is not on PATH.
    Builtin,
}

/// One run in the shell history.
#[derive(Debug, Clone, PartialEq)]
pub struct ShellEntry {
    pub script: String,
    /// Console text (stdout and stderr, or per-statement results).
    pub output: String,
    pub ok: bool,
    pub elapsed: Duration,
    /// Parsed result for the tree view.
    pub value: Option<mongo_raabit_core::bson::Bson>,
    pub engine: &'static str,
}

pub struct ShellTab {
    pub session: SessionId,
    pub db: String,
    pub script: String,
    pub engine: ShellEngine,
    /// Use the built-in evaluator even when mongosh exists.
    pub force_builtin: bool,
    pub history: Vec<ShellEntry>,
    pub running: bool,
    pub open: HashSet<String>,
    pub collections: Vec<String>,
}

/// Results as documents for the tree view: arrays of documents stay as they are, a
/// single document or scalar is wrapped.
pub fn shell_documents(value: &mongo_raabit_core::bson::Bson) -> Vec<Document> {
    use mongo_raabit_core::bson::{Bson, doc};
    match value {
        Bson::Array(items) => items
            .iter()
            .map(|item| match item {
                Bson::Document(d) => d.clone(),
                other => doc! {"value": other.clone()},
            })
            .collect(),
        Bson::Document(d) => vec![d.clone()],
        other => vec![doc! {"value": other.clone()}],
    }
}

impl MongoRaabit {
    pub fn open_shell(&mut self, session: SessionId, db: &str, collection: Option<&str>) -> TabId {
        let script = match collection {
            Some(coll) => format!("db.getCollection(\"{coll}\").find({{}}).limit(20)"),
            None => "show collections".into(),
        };
        let id = self.push_tab(TabKind::Shell(Box::new(ShellTab {
            session,
            db: db.to_owned(),
            script,
            engine: ShellEngine::Detecting,
            force_builtin: false,
            history: Vec::new(),
            running: false,
            open: HashSet::new(),
            collections: Vec::new(),
        })));
        self.jobs.spawn(
            async move {
                match mongo_raabit_core::mongosh::find_mongosh() {
                    Some(path) => {
                        let version = mongo_raabit_core::mongosh::version(&path)
                            .await
                            .unwrap_or_default();
                        ShellEngine::Mongosh { path, version }
                    }
                    None => ShellEngine::Builtin,
                }
            },
            move |app: &mut MongoRaabit, engine| {
                if let Some(tab) = app.shell_tab_mut(id) {
                    tab.engine = engine;
                }
            },
        );
        if let Some(conn) = self.connection_for(session) {
            let db = db.to_owned();
            self.jobs.spawn(
                async move { conn.collections(&db).await },
                move |app: &mut MongoRaabit, result| {
                    if let (Some(tab), Ok(list)) = (app.shell_tab_mut(id), result) {
                        tab.collections = list.into_iter().map(|c| c.name).collect();
                    }
                },
            );
        }
        id
    }

    pub fn shell_tab_mut(&mut self, id: TabId) -> Option<&mut ShellTab> {
        match self.tab_kind_mut(id)? {
            TabKind::Shell(tab) => Some(tab),
            _ => None,
        }
    }

    /// Run the shell tab's script (or `selection`, when given).
    pub fn run_shell(&mut self, tab_id: TabId, selection: Option<String>) {
        let Some(tab) = self.shell_tab_mut(tab_id) else {
            return;
        };
        if tab.running {
            return;
        }
        let script = selection.unwrap_or_else(|| tab.script.clone());
        if script.trim().is_empty() {
            return;
        }
        tab.running = true;
        let engine = if tab.force_builtin {
            ShellEngine::Builtin
        } else {
            tab.engine.clone()
        };
        let (session, db) = (tab.session, tab.db.clone());
        let Some(conn) = self.connection_for(session) else {
            return;
        };
        self.jobs.spawn(
            async move {
                let start = std::time::Instant::now();
                match engine {
                    ShellEngine::Mongosh { path, .. } => {
                        match conn
                            .run_mongosh(&path, &db, &script, Duration::from_secs(120))
                            .await
                        {
                            Ok(out) => {
                                let mut output = out.stdout.trim_end().to_owned();
                                if !out.stderr.trim().is_empty() {
                                    if !output.is_empty() {
                                        output.push('\n');
                                    }
                                    output.push_str(out.stderr.trim_end());
                                }
                                (
                                    ShellEntry {
                                        script,
                                        output,
                                        ok: out.success,
                                        elapsed: out.elapsed,
                                        value: out.value,
                                        engine: "mongosh",
                                    },
                                    db,
                                )
                            }
                            Err(err) => (
                                ShellEntry {
                                    script,
                                    output: err.to_string(),
                                    ok: false,
                                    elapsed: start.elapsed(),
                                    value: None,
                                    engine: "mongosh",
                                },
                                db,
                            ),
                        }
                    }
                    _ => {
                        let mut current = db;
                        let results = conn.eval_builtin(&mut current, &script).await;
                        let mut output = String::new();
                        let mut ok = true;
                        let mut value = None;
                        for evaluated in &results {
                            match &evaluated.result {
                                Ok(v) => {
                                    output.push_str(&mongo_raabit_core::shell::to_shell_string(
                                        v,
                                        mongo_raabit_core::shell::ShellStyle::Pretty(2),
                                    ));
                                    value = Some(v.clone());
                                }
                                Err(err) => {
                                    ok = false;
                                    output.push_str(&format!("Error: {err}"));
                                }
                            }
                            output.push('\n');
                        }
                        (
                            ShellEntry {
                                script,
                                output: output.trim_end().to_owned(),
                                ok,
                                elapsed: start.elapsed(),
                                value,
                                engine: "built-in",
                            },
                            current,
                        )
                    }
                }
            },
            move |app: &mut MongoRaabit, (entry, db)| {
                if let Some(tab) = app.shell_tab_mut(tab_id) {
                    tab.running = false;
                    tab.db = db;
                    tab.open.clear();
                    tab.history.push(entry);
                }
            },
        );
    }
}
