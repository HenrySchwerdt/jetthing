// Release builds are GUI apps on Windows (no console window).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use jetthing_ui::diagnostics;
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
            let mut app = jetthing_ui::IdeApp::new(project, files);
            app.start_control_from_env(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}

/// Turns winit's panic about a missing system library (e.g. `Library
/// libxkbcommon-x11.so could not be loaded.`) into install instructions.
fn missing_library_hint(message: &str) -> Option<String> {
    let rest = message.split("Library ").nth(1)?;
    let (library, _) = rest.split_once(" could not be loaded")?;
    let library = library.trim();
    // Debian/Ubuntu and Fedora package names for the libraries winit loads at runtime.
    let packages = [
        ("libxkbcommon-x11", "libxkbcommon-x11-0", "libxkbcommon-x11"),
        ("libxkbcommon", "libxkbcommon0", "libxkbcommon"),
        (
            "libwayland-client",
            "libwayland-client0",
            "libwayland-client",
        ),
        (
            "libwayland-cursor",
            "libwayland-cursor0",
            "libwayland-cursor",
        ),
        ("libX11-xcb", "libx11-xcb1", "libX11-xcb"),
        ("libX11", "libx11-6", "libX11"),
        ("libXcursor", "libxcursor1", "libXcursor"),
        ("libXi", "libxi6", "libXi"),
        ("libXrandr", "libxrandr2", "libXrandr"),
        ("libxcb", "libxcb1", "libxcb"),
    ];
    let install = packages
        .iter()
        .find(|(prefix, _, _)| library.starts_with(&format!("{prefix}.")))
        .map(|(_, apt, dnf)| {
            format!(
                "\n  Debian/Ubuntu: sudo apt install {apt}\n  Fedora:        sudo dnf install {dnf}"
            )
        })
        .unwrap_or_default();
    Some(format!(
        "jetthing: cannot start: the system library {library} is missing.\nInstall it with your package manager and start Jetthing again.{install}"
    ))
}

/// Missing desktop libraries make winit panic during startup; report them
/// as an actionable error instead of a backtrace. Any other panic leaves a
/// crash report in the log directory, offered to the user on the next start.
fn install_startup_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = info
            .payload()
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| info.payload().downcast_ref::<&str>().copied())
            .unwrap_or_default();
        match missing_library_hint(message) {
            Some(hint) => {
                diagnostics::log(diagnostics::Level::Error, &hint);
                eprintln!("{hint}");
                std::process::exit(1);
            }
            None => {
                let location = info.location().map(ToString::to_string);
                match diagnostics::write_crash_report(message, location.as_deref()) {
                    Ok(path) => eprintln!("jetthing: crash report written to {}", path.display()),
                    Err(error) => eprintln!("jetthing: could not write a crash report: {error}"),
                }
                default_hook(info)
            }
        }
    }));
}

const USAGE: &str = "usage: jetthing [PROJECT_DIR] [FILE...]

Opens PROJECT_DIR (or the welcome screen) and FILEs relative to it.
  -V, --version   print version and build information
  -h, --help      print this help
Environment: JETTHING_RENDERER=glow, JETTHING_NO_UPDATE_CHECK,
JETTHING_CONTROL_SOCKET. See README.md.";

/// Release builds on Windows have no console of their own; print `--version`
/// into the terminal that started them.
#[cfg(windows)]
fn attach_parent_console() {
    unsafe extern "system" {
        fn AttachConsole(process_id: u32) -> i32;
    }
    const ATTACH_PARENT_PROCESS: u32 = u32::MAX;
    // SAFETY: no pointers are passed; failure only means there is no console.
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

#[cfg(not(windows))]
fn attach_parent_console() {}

fn main() -> eframe::Result<()> {
    let first = std::env::args_os().nth(1);
    if let Some(text) = jetthing_ui::build_info::cli_flag_output(
        "jetthing",
        USAGE,
        first.as_deref().and_then(|arg| arg.to_str()),
    ) {
        attach_parent_console();
        println!("{text}");
        return Ok(());
    }
    if let Err(error) = diagnostics::init() {
        eprintln!("jetthing: logging disabled: {error}");
    }
    install_startup_panic_hook();
    // Debug builds only: exercise the crash-report path end to end.
    if cfg!(debug_assertions) && std::env::var_os("JETTHING_DEBUG_PANIC").is_some() {
        panic!("JETTHING_DEBUG_PANIC requested a test crash");
    }
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
            let message = format!("wgpu unavailable ({error}); falling back to OpenGL");
            diagnostics::log(diagnostics::Level::Warn, &message);
            eprintln!("jetthing: {message}");
            run(eframe::Renderer::Glow, &title, project, files)
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_xkbcommon_gets_install_instructions() {
        let hint =
            missing_library_hint("Library libxkbcommon-x11.so could not be loaded.").unwrap();
        assert!(hint.contains("libxkbcommon-x11.so is missing"), "{hint}");
        assert!(hint.contains("apt install libxkbcommon-x11-0"), "{hint}");
        assert!(hint.contains("dnf install libxkbcommon-x11"), "{hint}");
        let generic = missing_library_hint("Library libfoo.so.3 could not be loaded").unwrap();
        assert!(generic.contains("libfoo.so.3 is missing"));
        assert!(!generic.contains("apt install"));
        assert_eq!(missing_library_hint("index out of bounds"), None);
    }
}
