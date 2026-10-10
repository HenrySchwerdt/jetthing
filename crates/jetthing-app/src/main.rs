use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let project = std::env::args_os().nth(1).map(PathBuf::from);
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([800.0, 500.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "Jetthing",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_theme(eframe::egui::Theme::Dark);
            Ok(Box::new(jetthing_ui::IdeApp::new(project)))
        }),
    )
}
