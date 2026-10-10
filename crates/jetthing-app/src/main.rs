// Release builds are GUI apps on Windows (no console window).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod plugins;

use std::path::PathBuf;

enum Mode {
    Ide {
        project: Option<PathBuf>,
        files: Vec<PathBuf>,
    },
    /// `--app <plugin-id>`: only the plugin's app view.
    App(String),
}

fn native_options(renderer: eframe::Renderer, title: &str) -> eframe::NativeOptions {
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title(title)
        .with_app_id("jetthing")
        .with_inner_size([1440.0, 900.0])
        .with_min_inner_size([800.0, 500.0]);
    viewport = if cfg!(target_os = "macos") {
        // Keep native window controls in the OS title bar. The toolbar below it
        // starts at the content edge and needs no traffic-light spacer.
        viewport.with_title_shown(false)
    } else {
        // The main toolbar doubles as the title bar.
        viewport.with_decorations(false)
    };
    eframe::NativeOptions {
        viewport,
        renderer,
        ..Default::default()
    }
}

fn run(renderer: eframe::Renderer, title: &str, app: jetthing_ui::IdeApp) -> eframe::Result<()> {
    eframe::run_native(
        "Jetthing",
        native_options(renderer, title),
        Box::new(move |cc| {
            cc.egui_ctx.set_theme(eframe::egui::Theme::Dark);
            jetthing_ui::install_theme(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}

fn parse_args() -> Result<Mode, String> {
    let mut args = std::env::args_os().skip(1);
    let mut positional = Vec::new();
    while let Some(arg) = args.next() {
        if arg == "--app" {
            let id = args
                .next()
                .and_then(|id| id.into_string().ok())
                .ok_or("--app needs a plugin id, e.g. --app dev.jetthing.hello")?;
            return Ok(Mode::App(id));
        }
        if let Some(id) = arg.to_str().and_then(|arg| arg.strip_prefix("--app=")) {
            return Ok(Mode::App(id.to_owned()));
        }
        positional.push(PathBuf::from(arg));
    }
    let mut positional = positional.into_iter();
    Ok(Mode::Ide {
        project: positional.next(),
        files: positional.collect(),
    })
}

fn build_app(mode: &Mode) -> Result<jetthing_ui::IdeApp, String> {
    match mode {
        Mode::Ide { project, files } => Ok(jetthing_ui::IdeApp::with_plugins(
            project.clone(),
            files.clone(),
            plugins::bundled(),
        )),
        Mode::App(id) => jetthing_ui::IdeApp::standalone(id, plugins::bundled()),
    }
}

fn main() -> eframe::Result<()> {
    // Usage: jetthing [PROJECT_DIR] [FILE...]  |  jetthing --app <PLUGIN_ID>
    let mode = match parse_args() {
        Ok(mode) => mode,
        Err(error) => {
            eprintln!("jetthing: {error}");
            std::process::exit(2);
        }
    };
    let app = match build_app(&mode) {
        Ok(app) => app,
        Err(error) => {
            eprintln!("jetthing: {error}");
            std::process::exit(2);
        }
    };
    let project = match &mode {
        Mode::Ide { project, .. } => project.clone(),
        Mode::App(_) => None,
    };
    let title = app.app_title().unwrap_or_else(|| title_for(project));
    let forced_gl = std::env::var("JETTHING_RENDERER").is_ok_and(|value| value == "glow");
    if forced_gl {
        return run(eframe::Renderer::Glow, &title, app);
    }
    match run(eframe::Renderer::Wgpu, &title, app) {
        // No usable wgpu adapter/surface (VMs, old drivers): fall back to OpenGL.
        // The first app was consumed by the failed attempt; build a fresh one.
        Err(eframe::Error::Wgpu(error)) => {
            eprintln!("jetthing: wgpu unavailable ({error}); falling back to OpenGL");
            match build_app(&mode) {
                Ok(app) => run(eframe::Renderer::Glow, &title, app),
                Err(error) => {
                    eprintln!("jetthing: {error}");
                    std::process::exit(2);
                }
            }
        }
        other => other,
    }
}

fn title_for(project: Option<PathBuf>) -> String {
    project
        .as_deref()
        .and_then(|path| path.canonicalize().ok())
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .map(|name| format!("{name} – Jetthing"))
        .unwrap_or_else(|| "Jetthing".into())
}
