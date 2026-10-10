//! View state of mongo-raabit and the operations that load data in the background.

use crate::jobs::Jobs;
use crate::manager::ConnectionManager;
use mongo_raabit_core::Connection;
use mongo_raabit_core::connection::{
    CollectionInfo, CollectionKind, CollectionStats, DatabaseInfo, ServerInfo,
};
use mongo_raabit_core::profile::{ConnectionProfile, ProfileStore};
use mongo_raabit_core::query::{Page, Query, QueryText};
use mongo_raabit_core::secrets::{MemorySecretStore, SecretStore};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

pub type SessionId = u64;
pub type TabId = u64;

/// Something loaded in the background.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Load<T> {
    #[default]
    Idle,
    Loading,
    Ready(T),
    Failed(String),
}

impl<T> Load<T> {
    pub fn ready(&self) -> Option<&T> {
        match self {
            Self::Ready(value) => Some(value),
            _ => None,
        }
    }

    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }
}

pub struct CollectionEntry {
    pub info: CollectionInfo,
    pub stats: Option<CollectionStats>,
}

pub struct DatabaseEntry {
    pub info: DatabaseInfo,
    pub collections: Load<Vec<CollectionEntry>>,
}

pub enum SessionState {
    Connecting,
    Connected(Connection),
    Failed(String),
}

/// An opened connection in the connection tree.
pub struct Session {
    pub id: SessionId,
    pub profile: ConnectionProfile,
    pub state: SessionState,
    pub databases: Load<Vec<DatabaseEntry>>,
}

impl Session {
    pub fn connection(&self) -> Option<&Connection> {
        match &self.state {
            SessionState::Connected(conn) => Some(conn),
            _ => None,
        }
    }

    pub fn server(&self) -> Option<&ServerInfo> {
        self.connection().map(Connection::server)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultView {
    Table,
    Tree,
    Json,
}

pub struct CollectionTab {
    pub session: SessionId,
    pub db: String,
    pub collection: String,
    pub kind: CollectionKind,
    pub query: QueryText,
    pub page: u64,
    pub page_size: u64,
    pub result: Load<Page>,
    pub count: Load<u64>,
    pub view: ResultView,
    /// Drill-down path in the table view (`address`, `items`…).
    pub drill: Vec<String>,
    /// Expanded tree rows by path key.
    pub open: HashSet<String>,
    pub selected: Option<usize>,
    /// Incremented per run; stale results are dropped.
    pub seq: u64,
    pub ran: Option<Query>,
    pub query_error: Option<String>,
}

impl CollectionTab {
    pub fn new(session: SessionId, db: &str, collection: &str, kind: CollectionKind) -> Self {
        Self {
            session,
            db: db.to_owned(),
            collection: collection.to_owned(),
            kind,
            query: QueryText::default(),
            page: 0,
            page_size: 50,
            result: Load::Idle,
            count: Load::Idle,
            view: ResultView::Table,
            drill: Vec::new(),
            open: HashSet::new(),
            selected: None,
            seq: 0,
            ran: None,
            query_error: None,
        }
    }
}

pub enum TabKind {
    Welcome,
    Collection(Box<CollectionTab>),
}

pub struct Tab {
    pub id: TabId,
    pub kind: TabKind,
}

/// Tree node identity, used for selection and expansion.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeKey {
    Session(SessionId),
    Database(SessionId, String),
    Collection(SessionId, String, String),
}

pub struct MongoRaabit {
    pub jobs: Jobs<MongoRaabit>,
    pub secrets: Arc<dyn SecretStore>,
    pub store: ProfileStore,
    pub store_path: Option<PathBuf>,
    pub sessions: Vec<Session>,
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub manager: Option<ConnectionManager>,
    pub expanded: HashSet<NodeKey>,
    pub selected: Option<NodeKey>,
    pub tree_filter: String,
    pub status: String,
    next_id: u64,
}

impl Default for MongoRaabit {
    fn default() -> Self {
        Self::new(ProfileStore::default_path())
    }
}

impl MongoRaabit {
    /// Create the app, loading saved connections from `store_path` (none = in memory).
    pub fn new(store_path: Option<PathBuf>) -> Self {
        let (store, status) = match store_path.as_deref().map(ProfileStore::load) {
            Some(Ok(store)) => (store, String::new()),
            Some(Err(err)) => (
                ProfileStore::default(),
                format!("Could not load connections: {err}"),
            ),
            None => (ProfileStore::default(), String::new()),
        };
        Self {
            jobs: Jobs::new(),
            secrets: Arc::new(MemorySecretStore::default()),
            store,
            store_path,
            sessions: Vec::new(),
            tabs: vec![Tab {
                id: 0,
                kind: TabKind::Welcome,
            }],
            active_tab: 0,
            manager: None,
            expanded: HashSet::new(),
            selected: None,
            tree_filter: String::new(),
            status,
            next_id: 1,
        }
    }

    pub fn next_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    /// Apply finished background work. Call once per frame.
    pub fn poll(&mut self) {
        for apply in self.jobs.take() {
            apply(self);
        }
    }

    /// Block until background work settles (tests, scripted screenshots).
    pub fn wait_idle(&mut self, timeout: std::time::Duration) {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            self.poll();
            if !self.jobs.busy() || std::time::Instant::now() > deadline {
                self.poll();
                if !self.jobs.busy() || std::time::Instant::now() > deadline {
                    return;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    pub fn save_store(&mut self) {
        if let Some(path) = &self.store_path
            && let Err(err) = self.store.save(path)
        {
            self.status = format!("Could not save connections: {err}");
        }
    }

    pub fn session(&self, id: SessionId) -> Option<&Session> {
        self.sessions.iter().find(|s| s.id == id)
    }

    pub fn session_mut(&mut self, id: SessionId) -> Option<&mut Session> {
        self.sessions.iter_mut().find(|s| s.id == id)
    }

    pub fn open_manager(&mut self) {
        if self.manager.is_none() {
            self.manager = Some(ConnectionManager::new(&self.store));
        }
    }

    // ----- connections -------------------------------------------------------------------

    /// Open a session for `profile` and load its databases.
    pub fn connect(&mut self, profile: ConnectionProfile) -> SessionId {
        let id = self.next_id();
        self.status = format!("Connecting to {}…", profile.name);
        self.sessions.push(Session {
            id,
            profile: profile.clone(),
            state: SessionState::Connecting,
            databases: Load::Idle,
        });
        self.expanded.insert(NodeKey::Session(id));
        self.selected = Some(NodeKey::Session(id));
        let secrets = self.secrets.clone();
        self.jobs.spawn(
            async move { Connection::connect(&profile, secrets.as_ref()).await },
            move |app: &mut MongoRaabit, result| {
                let Some(session) = app.session_mut(id) else {
                    return;
                };
                let name = session.profile.name.clone();
                let connected = match result {
                    Ok(conn) => {
                        let status = format!(
                            "Connected to {name} — MongoDB {} ({})",
                            conn.server().version,
                            conn.server().topology
                        );
                        session.state = SessionState::Connected(conn);
                        Some(status)
                    }
                    Err(err) => {
                        session.state = SessionState::Failed(err.to_string());
                        None
                    }
                };
                match connected {
                    Some(status) => {
                        app.status = status;
                        app.load_databases(id);
                    }
                    None => app.status = format!("Connection to {name} failed"),
                }
            },
        );
        id
    }

    pub fn disconnect(&mut self, id: SessionId) {
        if let Some(index) = self.sessions.iter().position(|s| s.id == id) {
            let session = self.sessions.remove(index);
            self.tabs.retain(|tab| match &tab.kind {
                TabKind::Collection(c) => c.session != id,
                TabKind::Welcome => true,
            });
            self.active_tab = self.active_tab.min(self.tabs.len().saturating_sub(1));
            self.status = format!("Disconnected from {}", session.profile.name);
            if let SessionState::Connected(conn) = session.state {
                self.jobs.spawn(conn.disconnect(), |_, ()| {});
            }
        }
    }

    pub fn load_databases(&mut self, id: SessionId) {
        let Some(session) = self.session_mut(id) else {
            return;
        };
        let Some(conn) = session.connection().cloned() else {
            return;
        };
        session.databases = Load::Loading;
        self.jobs.spawn(
            async move { conn.databases().await },
            move |app: &mut MongoRaabit, result| {
                let Some(session) = app.session_mut(id) else {
                    return;
                };
                session.databases = match result {
                    Ok(list) => Load::Ready(
                        list.into_iter()
                            .map(|info| DatabaseEntry {
                                info,
                                collections: Load::Idle,
                            })
                            .collect(),
                    ),
                    Err(err) => Load::Failed(err.to_string()),
                };
            },
        );
    }

    pub fn load_collections(&mut self, id: SessionId, db: &str) {
        let Some(session) = self.session_mut(id) else {
            return;
        };
        let Some(conn) = session.connection().cloned() else {
            return;
        };
        let Load::Ready(databases) = &mut session.databases else {
            return;
        };
        let Some(entry) = databases.iter_mut().find(|d| d.info.name == db) else {
            return;
        };
        entry.collections = Load::Loading;
        let db = db.to_owned();
        let db_for_apply = db.clone();
        self.jobs.spawn(
            async move {
                let collections = conn.collections(&db).await?;
                let mut out = Vec::new();
                for info in collections {
                    let stats = if matches!(
                        info.kind,
                        CollectionKind::Collection | CollectionKind::TimeSeries
                    ) {
                        conn.collection_stats(&db, &info.name).await.ok()
                    } else {
                        None
                    };
                    out.push(CollectionEntry { info, stats });
                }
                Ok::<_, mongo_raabit_core::RaabitError>(out)
            },
            move |app: &mut MongoRaabit, result| {
                let Some(session) = app.session_mut(id) else {
                    return;
                };
                if let Load::Ready(databases) = &mut session.databases
                    && let Some(entry) = databases.iter_mut().find(|d| d.info.name == db_for_apply)
                {
                    entry.collections = match result {
                        Ok(list) => Load::Ready(list),
                        Err(err) => Load::Failed(err.to_string()),
                    };
                }
            },
        );
    }

    // ----- tabs --------------------------------------------------------------------------

    pub fn active_tab_id(&self) -> Option<TabId> {
        self.tabs.get(self.active_tab).map(|t| t.id)
    }

    pub fn tab_mut(&mut self, id: TabId) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|t| t.id == id)
    }

    pub fn collection_tab_mut(&mut self, id: TabId) -> Option<&mut CollectionTab> {
        match &mut self.tab_mut(id)?.kind {
            TabKind::Collection(tab) => Some(tab),
            _ => None,
        }
    }

    pub fn close_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.tabs.remove(index);
            if self.active_tab >= index && self.active_tab > 0 {
                self.active_tab -= 1;
            }
        }
    }

    /// Focus the tab of `db.collection` or open a new one, then run its query.
    pub fn open_collection(
        &mut self,
        session: SessionId,
        db: &str,
        collection: &str,
        kind: CollectionKind,
    ) {
        let existing = self.tabs.iter().position(|t| {
            matches!(&t.kind, TabKind::Collection(c) if c.session == session && c.db == db && c.collection == collection)
        });
        if let Some(index) = existing {
            self.active_tab = index;
            return;
        }
        let id = self.next_id();
        self.tabs.push(Tab {
            id,
            kind: TabKind::Collection(Box::new(CollectionTab::new(session, db, collection, kind))),
        });
        self.active_tab = self.tabs.len() - 1;
        self.run_query(id, true);
    }

    /// Run (or re-page) the query of a collection tab. `recount` refreshes the total.
    pub fn run_query(&mut self, tab_id: TabId, recount: bool) {
        let Some(tab) = self.collection_tab_mut(tab_id) else {
            return;
        };
        let query = match tab.query.parse() {
            Ok(query) => query,
            Err(err) => {
                tab.query_error = Some(err.to_string());
                return;
            }
        };
        tab.query_error = None;
        tab.seq += 1;
        let seq = tab.seq;
        let (session_id, db, coll, page, page_size) = (
            tab.session,
            tab.db.clone(),
            tab.collection.clone(),
            tab.page,
            tab.page_size,
        );
        if recount || tab.ran.as_ref() != Some(&query) {
            tab.count = Load::Loading;
        }
        let recount = tab.count.is_loading();
        tab.result = Load::Loading;
        tab.ran = Some(query.clone());
        let Some(conn) = self
            .session(session_id)
            .and_then(Session::connection)
            .cloned()
        else {
            if let Some(tab) = self.collection_tab_mut(tab_id) {
                tab.result = Load::Failed("not connected".into());
            }
            return;
        };
        {
            let conn = conn.clone();
            let (db, coll, query) = (db.clone(), coll.clone(), query.clone());
            self.jobs.spawn(
                async move { conn.find_page(&db, &coll, &query, page, page_size).await },
                move |app: &mut MongoRaabit, result| {
                    if let Some(tab) = app.collection_tab_mut(tab_id)
                        && tab.seq == seq
                    {
                        tab.result = match result {
                            Ok(page) => Load::Ready(page),
                            Err(err) => Load::Failed(err.to_string()),
                        };
                    }
                },
            );
        }
        if recount {
            self.jobs.spawn(
                async move { conn.count(&db, &coll, &query).await },
                move |app: &mut MongoRaabit, result| {
                    if let Some(tab) = app.collection_tab_mut(tab_id)
                        && tab.seq == seq
                    {
                        tab.count = match result {
                            Ok(n) => Load::Ready(n),
                            Err(err) => Load::Failed(err.to_string()),
                        };
                    }
                },
            );
        }
    }
}
