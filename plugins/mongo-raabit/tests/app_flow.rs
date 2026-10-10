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

#[test]
fn edit_insert_clone_delete_explain_and_code() {
    use mongo_raabit::app::CellEdit;
    use mongo_raabit::dialogs::{Dialog, DocumentMode};
    let (mut profile, _) = ConnectionProfile::from_uri(&uri()).unwrap();
    profile.connect_timeout_ms = Some(1500);
    let mut app = MongoRaabit::new(None);
    let session = app.connect(profile);
    app.wait_idle(Duration::from_secs(10));
    let Some(conn) = app.session(session).unwrap().connection().cloned() else {
        eprintln!(
            "skipping edit_insert_clone_delete_explain_and_code: no MongoDB server at {}",
            uri()
        );
        return;
    };
    let db = "raabit_test_ui_crud";
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let _ = conn.drop_database(db).await;
        conn.insert_documents(
            db,
            "people",
            vec![
                doc! {"_id": 1, "name": "Ada", "tier": "pro"},
                doc! {"_id": 2, "name": "Linus"},
            ],
        )
        .await
        .unwrap();
    });
    app.open_collection(session, db, "people", CollectionKind::Collection);
    app.wait_idle(Duration::from_secs(10));
    let tab = app.active_tab_id().unwrap();
    app.collection_tab_mut(tab).unwrap().selected = Some(0);

    // Edit through the document editor.
    app.edit_selected(tab);
    let Some(Dialog::Document(editor)) = &app.dialog else {
        panic!("editor not open")
    };
    assert_eq!(
        editor.mode,
        DocumentMode::Edit(mongo_raabit::core::bson::Bson::Int32(1))
    );
    app.save_document(
        tab,
        DocumentMode::Edit(1.into()),
        doc! {"_id": 1, "name": "Ada L.", "tier": "pro"},
    );
    app.wait_idle(Duration::from_secs(10));
    assert!(app.dialog.is_none(), "dialog closes after saving");
    assert_eq!(
        app.selected_document(tab).unwrap().get_str("name").unwrap(),
        "Ada L."
    );

    // A failing save keeps the editor open with the error.
    app.edit_selected(tab);
    app.save_document(
        tab,
        DocumentMode::Edit(1.into()),
        doc! {"_id": 99, "name": "x"},
    );
    app.wait_idle(Duration::from_secs(10));
    let Some(Dialog::Document(editor)) = &app.dialog else {
        panic!("editor closed on error")
    };
    assert!(
        editor
            .error
            .as_deref()
            .unwrap()
            .contains("_id cannot be changed")
    );
    app.dialog = None;

    // Inline edit keeps strings as strings and parses other types.
    app.collection_tab_mut(tab).unwrap().editing = Some(CellEdit {
        doc_index: 0,
        path: "tier".into(),
        text: "enterprise".into(),
        original: Some("pro".into()),
        focus: false,
    });
    app.commit_cell_edit(tab);
    app.wait_idle(Duration::from_secs(10));
    app.collection_tab_mut(tab).unwrap().editing = Some(CellEdit {
        doc_index: 0,
        path: "score".into(),
        text: "41.5".into(),
        original: None,
        focus: false,
    });
    app.commit_cell_edit(tab);
    app.wait_idle(Duration::from_secs(10));
    let doc = app.selected_document(tab).unwrap();
    assert_eq!(doc.get_str("tier").unwrap(), "enterprise");
    assert_eq!(doc.get_f64("score").unwrap(), 41.5);

    // Insert and clone.
    app.save_document(tab, DocumentMode::Insert, doc! {"name": "Grace"});
    app.wait_idle(Duration::from_secs(10));
    app.clone_selected(tab);
    let Some(Dialog::Document(editor)) = &app.dialog else {
        panic!("clone editor")
    };
    assert_eq!(editor.mode, DocumentMode::Insert);
    assert!(!editor.text.contains("_id"));
    let cloned = editor.parse().unwrap();
    app.save_document(tab, DocumentMode::Insert, cloned);
    app.wait_idle(Duration::from_secs(10));
    assert_eq!(app.collection_tab_mut(tab).unwrap().count, Load::Ready(4));

    // Delete with confirmation.
    app.collection_tab_mut(tab).unwrap().selected = Some(1);
    app.request_delete(tab);
    let Some(Dialog::Delete(confirm)) = &app.dialog else {
        panic!("confirm")
    };
    let ids = confirm.ids.clone();
    app.delete_documents(tab, ids);
    app.wait_idle(Duration::from_secs(10));
    assert_eq!(app.collection_tab_mut(tab).unwrap().count, Load::Ready(3));

    // Explain and query code.
    app.collection_tab_mut(tab).unwrap().query.filter = "{name: 'Ada L.'}".into();
    app.explain(tab);
    app.wait_idle(Duration::from_secs(10));
    let Some(Dialog::Explain(explain)) = &app.dialog else {
        panic!("explain")
    };
    let plan = explain.result.ready().expect("explain finished");
    assert!(plan.root.has_collection_scan());
    app.query_code(tab);
    let Some(Dialog::Code(code)) = &app.dialog else {
        panic!("code")
    };
    assert_eq!(
        code.shell,
        "db.getCollection(\"people\").find({ name: \"Ada L.\" })"
    );
    rt.block_on(async {
        conn.drop_database(db).await.unwrap();
    });
}

#[test]
fn aggregation_sql_schema_and_index_tabs() {
    let (mut profile, _) = ConnectionProfile::from_uri(&uri()).unwrap();
    profile.connect_timeout_ms = Some(1500);
    let mut app = MongoRaabit::new(None);
    let session = app.connect(profile);
    app.wait_idle(Duration::from_secs(10));
    let Some(conn) = app.session(session).unwrap().connection().cloned() else {
        eprintln!(
            "skipping aggregation_sql_schema_and_index_tabs: no MongoDB server at {}",
            uri()
        );
        return;
    };
    let db = "raabit_test_ui_tools";
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let _ = conn.drop_database(db).await;
        conn.insert_documents(
            db,
            "sales",
            (0..30).map(|i| doc! {"region": (["north", "south", "east"][i % 3]), "amount": i as i32 * 10}).collect(),
        )
        .await
        .unwrap();
    });

    // Aggregation: preview of the selected stage and a full run.
    let pipeline = mongo_raabit::core::aggregate::Pipeline::from_text(
        "[{$match: {amount: {$gte: 100}}}, {$group: {_id: '$region', total: {$sum: '$amount'}}}, {$sort: {_id: 1}}]",
    )
    .unwrap();
    let agg = app.open_aggregation(session, db, "sales", Some(pipeline));
    app.aggregation_tab_mut(agg).unwrap().selected = 1;
    app.preview_stage(agg);
    app.wait_idle(Duration::from_secs(10));
    {
        let tab = app.aggregation_tab_mut(agg).unwrap();
        let preview = tab.preview.ready().expect("preview");
        assert_eq!(preview.input.len(), 20);
        assert_eq!(preview.output.len(), 3);
    }
    app.run_pipeline(agg);
    app.wait_idle(Duration::from_secs(10));
    let result = app
        .aggregation_tab_mut(agg)
        .unwrap()
        .result
        .ready()
        .cloned()
        .unwrap();
    assert_eq!(result.documents[0], doc! {"_id": "east", "total": 1400});

    // Invalid stage text is reported on the tab, not sent to the server.
    app.aggregation_tab_mut(agg).unwrap().pipeline.stages[0].body = "{amount: }".into();
    app.preview_stage(agg);
    assert!(
        app.aggregation_tab_mut(agg)
            .unwrap()
            .stage_error
            .as_deref()
            .unwrap()
            .starts_with("stage 1")
    );

    // SQL.
    let sql = app.open_sql(session, db, Some("sales")).unwrap();
    assert_eq!(
        app.collection_tab_mut(sql).unwrap().mode,
        mongo_raabit::app::QueryMode::Sql
    );
    app.sql_tab_mut(sql).unwrap().sql = "SELECT region, SUM(amount) AS total FROM sales GROUP BY region ORDER BY total DESC LIMIT 1".into();
    app.run_sql(sql);
    app.wait_idle(Duration::from_secs(10));
    let rows = app
        .sql_tab_mut(sql)
        .unwrap()
        .result
        .ready()
        .cloned()
        .unwrap()
        .documents;
    assert_eq!(rows, vec![doc! {"region": "east", "total": 1550}]);
    app.sql_tab_mut(sql).unwrap().sql = "DELETE FROM sales".into();
    app.run_sql(sql);
    assert!(matches!(
        app.sql_tab_mut(sql).unwrap().result,
        Load::Failed(_)
    ));

    // Schema.
    let schema = app.open_schema(session, db, "sales");
    app.wait_idle(Duration::from_secs(10));
    let report = app
        .schema_tab_mut(schema)
        .unwrap()
        .report
        .ready()
        .cloned()
        .unwrap();
    assert_eq!(report.sampled, 30);
    assert!(
        report
            .fields
            .iter()
            .any(|f| f.path == "region" && f.top_values.len() == 3)
    );

    // Indexes: create, hide, drop. They share the Structure tab with the schema.
    let idx = app.open_indexes(session, db, "sales");
    assert_eq!(idx, schema);
    assert_eq!(app.tabs.len(), 4, "welcome, query, aggregation, structure");
    app.wait_idle(Duration::from_secs(10));
    assert_eq!(
        app.index_tab_mut(idx)
            .unwrap()
            .indexes
            .ready()
            .unwrap()
            .len(),
        1
    );
    app.index_tab_mut(idx).unwrap().form.keys = "{region: 1, amount: -1}".into();
    app.index_tab_mut(idx).unwrap().form.name = "by_region".into();
    app.create_index(idx);
    app.wait_idle(Duration::from_secs(10));
    assert_eq!(
        app.index_tab_mut(idx)
            .unwrap()
            .indexes
            .ready()
            .unwrap()
            .len(),
        2
    );
    app.set_index_hidden(idx, "by_region".into(), true);
    app.wait_idle(Duration::from_secs(10));
    assert!(
        app.index_tab_mut(idx)
            .unwrap()
            .indexes
            .ready()
            .unwrap()
            .iter()
            .any(|i| i.name == "by_region" && i.hidden)
    );
    app.drop_index(idx, "by_region".into());
    app.wait_idle(Duration::from_secs(10));
    assert_eq!(
        app.index_tab_mut(idx)
            .unwrap()
            .indexes
            .ready()
            .unwrap()
            .len(),
        1
    );
    app.index_tab_mut(idx).unwrap().form.keys = "{}".into();
    app.create_index(idx);
    assert!(app.index_tab_mut(idx).unwrap().error.is_some());

    // Render every tab headlessly.
    let ctx = egui::Context::default();
    for i in 0..app.tabs.len() {
        app.active_tab = i;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| app.ui(ui));
        });
    }
    rt.block_on(async {
        conn.drop_database(db).await.unwrap();
    });
}
