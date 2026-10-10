//! Drives the view state against a local mongod (skipped when none is reachable).

use mongo_raabit::MongoRaabit;
use mongo_raabit::app::{Load, SessionState, TabKind};
use mongo_raabit::core::bson::doc;
use mongo_raabit::core::connection::CollectionKind;
use mongo_raabit::core::profile::ConnectionProfile;
use std::time::Duration;

fn uri() -> String {
    std::env::var("MONGO_RAABIT_TEST_URI").unwrap_or_else(|_| "mongodb://127.0.0.1:27017".into())
}

#[test]
fn connect_browse_query_and_page() {
    let (mut profile, _) = ConnectionProfile::from_uri(&uri()).unwrap();
    profile.connect_timeout_ms = Some(1500);
    let mut app = MongoRaabit::new(None);
    let session = app.connect(profile);
    app.wait_idle(Duration::from_secs(10));
    let state = &app.session(session).unwrap().state;
    if let SessionState::Failed(err) = state {
        eprintln!(
            "skipping connect_browse_query_and_page: no MongoDB server at {} ({err})",
            uri()
        );
        return;
    }
    // Seed a private database through the session's connection.
    let conn = app.session(session).unwrap().connection().unwrap().clone();
    let db = "raabit_test_ui_flow";
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let _ = conn.drop_database(db).await;
        conn.insert_documents(
            db,
            "items",
            (0..120)
                .map(|i| doc! {"i": i, "even": i % 2 == 0})
                .collect(),
        )
        .await
        .unwrap();
    });
    app.load_databases(session);
    app.wait_idle(Duration::from_secs(10));
    assert!(
        app.session(session)
            .unwrap()
            .databases
            .ready()
            .unwrap()
            .iter()
            .any(|d| d.info.name == db)
    );
    app.load_collections(session, db);
    app.wait_idle(Duration::from_secs(10));

    app.open_collection(session, db, "items", CollectionKind::Collection);
    app.wait_idle(Duration::from_secs(10));
    let tab_id = app.active_tab_id().unwrap();
    {
        let tab = app.collection_tab_mut(tab_id).unwrap();
        assert_eq!(tab.count, Load::Ready(120));
        let page = tab.result.ready().unwrap();
        assert_eq!(page.documents.len(), 50);
        assert!(page.has_more);
        tab.query.filter = "{even: true}".into();
        tab.query.sort = "{i: -1}".into();
    }
    app.run_query(tab_id, true);
    app.wait_idle(Duration::from_secs(10));
    {
        let tab = app.collection_tab_mut(tab_id).unwrap();
        assert_eq!(tab.count, Load::Ready(60));
        assert_eq!(
            tab.result.ready().unwrap().documents[0]
                .get_i32("i")
                .unwrap(),
            118
        );
        tab.page = 1;
    }
    app.run_query(tab_id, false);
    app.wait_idle(Duration::from_secs(10));
    {
        let tab = app.collection_tab_mut(tab_id).unwrap();
        let page = tab.result.ready().unwrap();
        assert_eq!(page.documents.len(), 10);
        assert!(!page.has_more);
        assert_eq!(tab.count, Load::Ready(60));
        tab.query.filter = "{even: }".into();
    }
    app.run_query(tab_id, true);
    assert!(
        app.collection_tab_mut(tab_id)
            .unwrap()
            .query_error
            .is_some()
    );

    // Opening the same collection again focuses the existing tab.
    let tabs = app.tabs.len();
    app.open_collection(session, db, "items", CollectionKind::Collection);
    assert_eq!(app.tabs.len(), tabs);

    // Disconnect closes the session's tabs.
    app.disconnect(session);
    assert!(app.tabs.iter().all(|t| matches!(t.kind, TabKind::Welcome)));
    rt.block_on(async {
        conn.drop_database(db).await.unwrap();
    });
}

#[test]
fn renders_every_view_headlessly() {
    let mut app = MongoRaabit::new(None);
    app.open_manager();
    let ctx = egui::Context::default();
    for _ in 0..2 {
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| app.ui(ui));
        });
    }
    assert!(app.manager.is_some());
}
