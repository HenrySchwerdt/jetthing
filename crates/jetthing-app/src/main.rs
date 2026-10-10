// Release builds are GUI apps on Windows (no console window).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

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

fn run(
    renderer: eframe::Renderer,
    title: &str,
    project: Option<PathBuf>,
    files: Vec<PathBuf>,
) -> eframe::Result<()> {
    eframe::run_native(
        "Jetthing",
        native_options(renderer, title),
        Box::new(move |cc| {
            cc.egui_ctx.set_theme(eframe::egui::Theme::Dark);
            jetthing_ui::install_theme(&cc.egui_ctx);
            Ok(Box::new(jetthing_ui::IdeApp::new(project, files)))
        }),
    )
}

fn main() -> eframe::Result<()> {
    // Usage: jetthing-app [PROJECT_DIR] [FILE...]
    let mut args = std::env::args_os().skip(1).map(PathBuf::from);
    let project = args.next();
    let files: Vec<PathBuf> = args.collect();
    let title = project
        .as_deref()
        .and_then(|path| path.canonicalize().ok())
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .map(|name| format!("{name} – Jetthing"))
        .unwrap_or_else(|| "Jetthing".into());
    let forced_gl = std::env::var("JETTHING_RENDERER").is_ok_and(|value| value == "glow");
    if forced_gl {
        return run(eframe::Renderer::Glow, &title, project, files);
    }
    match run(
        eframe::Renderer::Wgpu,
        &title,
        project.clone(),
        files.clone(),
    ) {
        // No usable wgpu adapter/surface (VMs, old drivers): fall back to OpenGL.
        Err(eframe::Error::Wgpu(error)) => {
            eprintln!("jetthing: wgpu unavailable ({error}); falling back to OpenGL");
            run(eframe::Renderer::Glow, &title, project, files)
        }
        other => other,
    }
}
