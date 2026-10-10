use super::*;
use jetthing_plugin_api::json;
use jetthing_plugin_api::testing::RecordingHost;
use std::fs;

fn example_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins/hello-wasm")
}

fn manifest(permissions: &str) -> PluginManifest {
    let text = fs::read_to_string(example_dir().join("jetthing-plugin.toml")).unwrap();
    PluginManifest::parse(&text.replace("permissions = [\"fs.project\"]", permissions)).unwrap()
}

fn load(permissions: &str) -> WasmPlugin {
    let loader = WasmLoader::new().unwrap();
    WasmPlugin::load(&loader.engine, manifest(permissions), &example_dir()).unwrap()
}

fn host_with_project(project: &Path, permissions: &[Permission]) -> RecordingHost {
    let mut host = RecordingHost::new("dev.jetthing.hello-wasm").with_permissions(permissions);
    host.project = Some(project.to_path_buf());
    host
}

#[test]
fn component_declares_contributions_and_runs_commands() {
    let mut plugin = load("permissions = []");
    let contributions = plugin.contributions();
    assert!(
        contributions.problems().is_empty(),
        "{:?}",
        contributions.problems()
    );
    assert_eq!(contributions.commands.len(), 3);
    assert_eq!(
        contributions.tool_windows[0].icon_text.as_deref(),
        Some("W")
    );
    let mut host = RecordingHost::new("dev.jetthing.hello-wasm");
    plugin.activate(&mut host).unwrap();
    assert_eq!(host.statuses, ["Hello WASM activated"]);
    host.settings.insert("greeting".into(), json!("Hi"));
    let result = plugin
        .execute("hello_wasm.greet", json!({"name": "Ada"}), &mut host)
        .unwrap();
    assert_eq!(result["message"], "Hi, Ada!");
    assert_eq!(host.statuses.last().unwrap(), "Hi, Ada!");
    let error = plugin
        .execute("hello_wasm.greet", json!({}), &mut host)
        .unwrap_err()
        .to_string();
    assert!(error.contains("expected string field"), "{error}");
    assert!(matches!(
        plugin.execute("nope.nope", Json::Null, &mut host),
        Err(CommandError::Unknown(_))
    ));
    assert_eq!(plugin.status_widget("count").unwrap().text, "WASM × 1");
}

#[test]
fn project_files_need_the_fs_project_permission() {
    let project =
        std::env::temp_dir().join(format!("jetthing-wasm-project-{}", std::process::id()));
    let _ = fs::remove_dir_all(&project);
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join("Cargo.toml"), "").unwrap();
    fs::write(project.join("README.md"), "").unwrap();

    let mut allowed = load("permissions = [\"fs.project\"]");
    let mut host = host_with_project(&project, &[Permission::ProjectFiles]);
    allowed.activate(&mut host).unwrap();
    let files = allowed
        .execute("hello_wasm.listProject", Json::Null, &mut host)
        .unwrap();
    assert_eq!(files["files"], json!(["Cargo.toml", "README.md"]));
    assert_eq!(host.tool_windows, ["hello_wasm"]);

    let mut denied = load("permissions = []");
    let mut host = host_with_project(&project, &[]);
    denied.activate(&mut host).unwrap();
    let files = denied
        .execute("hello_wasm.listProject", Json::Null, &mut host)
        .unwrap();
    let first = files["files"][0].as_str().unwrap();
    assert!(first.contains("/project unavailable"), "{first}");
    fs::remove_dir_all(&project).unwrap();
}

#[test]
fn runaway_calls_run_out_of_fuel_and_the_plugin_recovers() {
    let mut plugin = load("permissions = []");
    // `hello_wasm.spin` is not contributed; declare it so execute calls the guest.
    plugin
        .contributions
        .commands
        .push(jetthing_plugin_api::CommandContribution::new(
            "hello_wasm.spin",
            "Spin",
        ));
    let mut host = RecordingHost::new("dev.jetthing.hello-wasm");
    let error = plugin
        .execute("hello_wasm.spin", Json::Null, &mut host)
        .unwrap_err()
        .to_string();
    assert!(error.contains("out of fuel"), "{error}");
    // A fresh instance serves the next call.
    let result = plugin
        .execute("hello_wasm.sayHello", Json::Null, &mut host)
        .unwrap();
    assert_eq!(result["message"], "Hello, WASM!");
}

#[test]
fn declarative_views_render_and_events_reach_the_guest() {
    let mut plugin = load("permissions = []");
    let mut host = RecordingHost::new("dev.jetthing.hello-wasm");
    let ctx = egui::Context::default();
    let view = ViewContext {
        view: "main".into(),
        placement: ViewPlacement::ToolWindow {
            id: "hello_wasm".into(),
        },
    };
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            plugin.render_view(&view, &mut ViewFrame::new(ui), &mut host);
        });
    });
    assert_eq!(plugin.view_error, None);
    // Events go through on-view-event.
    let event =
        serde_json::to_string(&jetthing_plugin_api::ViewEvent::Clicked { id: "hello".into() })
            .unwrap();
    plugin
        .call(CALL_FUEL, Some(&mut host), |p, store| {
            p.call_on_view_event(store, "main", &event)
        })
        .unwrap();
    assert_eq!(host.statuses.last().unwrap(), "Hello, WASM!");
}

#[test]
fn invalid_components_fail_with_actionable_errors() {
    let dir = std::env::temp_dir().join(format!("jetthing-wasm-bad-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("hello_wasm.wasm"), b"\0asm not really").unwrap();
    let loader = WasmLoader::new().unwrap();
    let error = WasmPlugin::load(&loader.engine, manifest("permissions = []"), &dir)
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("not a valid component"), "{error}");
    fs::remove_file(dir.join("hello_wasm.wasm")).unwrap();
    let error = WasmPlugin::load(&loader.engine, manifest("permissions = []"), &dir)
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("hello_wasm.wasm"), "{error}");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn loader_registers_wasm_plugins_with_the_registry() {
    use jetthing_plugins::{PluginLoader, PluginRegistry, PluginStatus};
    use std::rc::Rc;
    let root = std::env::temp_dir().join(format!("jetthing-wasm-registry-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let target = root.join("hello-wasm");
    fs::create_dir_all(&target).unwrap();
    for file in ["jetthing-plugin.toml", "hello_wasm.wasm"] {
        fs::copy(example_dir().join(file), target.join(file)).unwrap();
    }
    let loader = WasmLoader::new().unwrap();
    assert_eq!(loader.kind(), PluginKind::Wasm);
    let mut registry = PluginRegistry::default();
    registry.add_loader(Rc::new(loader));
    registry.discover(&root);
    assert_eq!(
        registry.get("dev.jetthing.hello-wasm").unwrap().status,
        PluginStatus::Enabled
    );
    registry.ensure_instance("dev.jetthing.hello-wasm").unwrap();
    fs::remove_dir_all(&root).unwrap();
}
