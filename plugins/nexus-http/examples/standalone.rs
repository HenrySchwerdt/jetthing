//! Development host: runs nexus-http in its own window with the Jetthing theme.
//! Once the plugin API lands this is replaced by `jetthing --app nexus-http`.
//!
//! ```bash
//! cargo run -p nexus-http --example standalone -- [DIR]       # open DIR/.nexus
//! cargo run -p nexus-http --example standalone -- --demo      # seeded demo + local API
//! ```
//!
//! Demo options (environment): `NEXUS_LIGHT=1` light theme, `NEXUS_SEND=1` send
//! the selected request on start, `NEXUS_SELECT=<request name>`,
//! `NEXUS_REQUEST_TAB=<tab>`, `NEXUS_RESPONSE_TAB=<tab>`,
//! `NEXUS_OVERLAY=env|switcher|import|export|code`.

use eframe::egui;
use nexus_http::NexusApp;
use nexus_http::state::{NexusState, RequestTab, ResponseTab};
use nexus_http::views::overlays::Overlay;
use nexus_http_core::assertions::{Assertion, Operator, Source};
use nexus_http_core::model::*;
use serde_json::json;
use std::path::PathBuf;

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let demo = args.iter().any(|a| a == "--demo");
    let mut app = if demo {
        let port = demo_server();
        let dir = std::env::temp_dir().join(format!("nexus-demo-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let mut state = NexusState::open(&dir);
        state.replace_workspace(demo_workspace(port));
        NexusApp::with_state(state)
    } else {
        let dir = args
            .first()
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        NexusApp::open(&dir)
    };
    apply_demo_env(&mut app);
    let light = std::env::var_os("NEXUS_LIGHT").is_some();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("nexus-http")
            .with_inner_size([1440.0, 900.0]),
        renderer: if std::env::var("JETTHING_RENDERER").as_deref() == Ok("glow") {
            eframe::Renderer::Glow
        } else {
            eframe::Renderer::default()
        },
        ..Default::default()
    };
    eframe::run_native(
        "nexus-http",
        options,
        Box::new(move |cc| {
            install_fonts(&cc.egui_ctx);
            cc.egui_ctx
                .style_mut(if light { islands_light } else { islands_dark });
            Ok(Box::new(Host { app, sent: false }))
        }),
    )
}

struct Host {
    app: NexusApp,
    sent: bool,
}

impl eframe::App for Host {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.sent && std::env::var_os("NEXUS_SEND").is_some() {
            self.sent = true;
            self.app.send_selected();
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::new())
            .show(ctx, |ui| self.app.ui(ui));
    }
}

fn apply_demo_env(app: &mut NexusApp) {
    if let Ok(name) = std::env::var("NEXUS_SELECT")
        && let Some(id) = app
            .state
            .workspace
            .requests()
            .iter()
            .find(|(_, r)| r.name == name)
            .map(|(_, r)| r.id.clone())
    {
        app.state.select(id);
    }
    if let Ok(tab) = std::env::var("NEXUS_REQUEST_TAB")
        && let Some(t) = RequestTab::ALL
            .iter()
            .find(|t| t.label().eq_ignore_ascii_case(&tab))
    {
        app.state.request_tab = *t;
    }
    if let Ok(tab) = std::env::var("NEXUS_RESPONSE_TAB")
        && let Some(t) = ResponseTab::ALL
            .iter()
            .find(|t| t.label().eq_ignore_ascii_case(&tab))
    {
        app.state.response_tab = *t;
    }
    let overlay = match std::env::var("NEXUS_OVERLAY").as_deref() {
        Ok("env") => Overlay::Environments,
        Ok("switcher") => Overlay::Switcher,
        Ok("import") => Overlay::Import,
        Ok("export") => Overlay::Export,
        Ok("code") => Overlay::Code,
        _ => Overlay::None,
    };
    if overlay != Overlay::None {
        app.open_overlay(overlay);
    }
}

/// The workbench fonts (Inter, JetBrains Mono; SIL OFL) bundled with jetthing-ui.
fn install_fonts(ctx: &egui::Context) {
    use egui::{FontData, FontDefinitions, FontFamily};
    use std::sync::Arc;
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "inter".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../../crates/jetthing-ui/assets/fonts/Inter-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        "inter-semibold".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../../crates/jetthing-ui/assets/fonts/Inter-SemiBold.ttf"
        ))),
    );
    fonts.font_data.insert(
        "mono".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../../crates/jetthing-ui/assets/fonts/JetBrainsMono-Regular.ttf"
        ))),
    );
    let fallbacks = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "inter".into());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "mono".into());
    let mut semibold = vec!["inter-semibold".to_owned()];
    semibold.extend(fallbacks);
    fonts
        .families
        .insert(FontFamily::Name("semibold".into()), semibold);
    ctx.set_fonts(fonts);
}

/// Islands Dark colors (facts from the IntelliJ Community theme, as used by jetthing-ui).
fn islands_dark(style: &mut egui::Style) {
    let c = |rgb: u32| egui::Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8);
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(8.0, 3.0);
    style.spacing.interact_size.y = 24.0;
    style.interaction.selectable_labels = false;
    let v = &mut style.visuals;
    *v = egui::Visuals::dark();
    v.override_text_color = Some(c(0xD1D3D9));
    v.panel_fill = c(0x26282C);
    v.code_bg_color = c(0x191A1C);
    v.window_fill = c(0x26282C);
    v.extreme_bg_color = c(0x191A1C);
    v.selection.bg_fill = c(0x2A4371);
    v.window_stroke = egui::Stroke::new(1.0_f32, c(0x33353B));
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, c(0x33353B));
    v.widgets.inactive.bg_fill = c(0x191A1C);
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, c(0x40434A));
    v.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
    v.widgets.hovered.weak_bg_fill = egui::Color32::from_white_alpha(0x17);
    v.widgets.active.bg_stroke = egui::Stroke::new(2.0_f32, c(0x3871E1));
}

/// Approximation of the Islands Light palette until the host theme engine
/// provides it.
fn islands_light(style: &mut egui::Style) {
    let c = |rgb: u32| egui::Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8);
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(8.0, 3.0);
    style.spacing.interact_size.y = 24.0;
    style.interaction.selectable_labels = false;
    let v = &mut style.visuals;
    *v = egui::Visuals::light();
    v.override_text_color = Some(c(0x1E1F22));
    v.panel_fill = c(0xF2F3F5);
    v.code_bg_color = c(0xFFFFFF);
    v.window_fill = c(0xFFFFFF);
    v.extreme_bg_color = c(0xFFFFFF);
    v.selection.bg_fill = c(0xD4E2FF);
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, c(0xDFE1E5));
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, c(0xC9CCD6));
    v.widgets.hovered.weak_bg_fill = c(0xEDEEF1);
    v.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
    v.window_stroke = egui::Stroke::new(1.0_f32, c(0xDFE1E5));
}

fn demo_workspace(port: u16) -> Workspace {
    let mut ws = Workspace::new("Acme Store API");
    ws.base_environment = ws
        .base_environment
        .clone()
        .with("base_url", format!("http://127.0.0.1:{port}"))
        .with("api_version", "v1");
    let mut local = Environment::new("Local")
        .with("user", "ada@acme.test")
        .with("password", "demo");
    local.color = Some("#4E9D6C".into());
    local.secrets.push("password".into());
    let mut staging = Environment::new("Staging")
        .with("base_url", "https://staging.acme.test")
        .with("user", "qa@acme.test")
        .with("password", "");
    staging.color = Some("#D59637".into());
    ws.active_environment = Some(local.id.clone());
    ws.environments = vec![local, staging];

    let mut login = Request::new(
        "Login",
        Method::Post,
        "{{ base_url }}/{{ api_version }}/auth/login",
    )
    .with_body(Body::Json {
        text: "{\n  \"email\": \"{{ user }}\",\n  \"password\": \"{{ password }}\"\n}".into(),
    });
    login
        .assertions
        .push(Assertion::new(Source::Status, Operator::Equals, "200"));
    login.assertions.push(Assertion::new(
        Source::JsonPath("$.token".into()),
        Operator::Exists,
        "",
    ));
    let login_id = login.id.clone();
    let me = Request::new(
        "Current user",
        Method::Get,
        "{{ base_url }}/{{ api_version }}/me",
    )
    .with_auth(Auth::Bearer {
        token: format!("{{% response 'body', '{login_id}', '$.token' %}}"),
        prefix: "Bearer".into(),
    });
    let mut auth = Folder::new("Auth");
    auth.children = vec![Item::Request(login), Item::Request(me)];

    let mut products = Folder::new("Products");
    products
        .headers
        .push(KeyValue::new("Accept", "application/json"));
    let mut list = Request::new(
        "List products",
        Method::Get,
        "{{ base_url }}/{{ api_version }}/products",
    )
    .param("category", "keyboards")
    .param("limit", "20")
    .header("X-Request-Id", "{% uuid %}");
    list.params.push(KeyValue {
        enabled: false,
        ..KeyValue::new("sort", "-price")
    });
    list.assertions
        .push(Assertion::new(Source::Status, Operator::Equals, "200"));
    list.assertions.push(Assertion::new(
        Source::JsonPath("$.items[0].sku".into()),
        Operator::Equals,
        "KB-001",
    ));
    list.assertions.push(Assertion::new(
        Source::ResponseTimeMs,
        Operator::LessThan,
        "500",
    ));
    let get = Request::new(
        "Get product",
        Method::Get,
        "{{ base_url }}/{{ api_version }}/products/KB-001",
    );
    let create = Request::new("Create product", Method::Post, "{{ base_url }}/{{ api_version }}/products")
        .with_body(Body::Json {
            text: "{\n  \"sku\": \"KB-042\",\n  \"name\": \"Split Keyboard\",\n  \"price\": 189.0,\n  \"tags\": [\"ergonomic\", \"wireless\"],\n  \"in_stock\": true\n}".into(),
        })
        .with_auth(Auth::Inherit);
    let update = Request::new(
        "Update price",
        Method::Patch,
        "{{ base_url }}/{{ api_version }}/products/KB-042",
    )
    .with_body(Body::Json {
        text: "{ \"price\": 169.0 }".into(),
    });
    let delete = Request::new(
        "Delete product",
        Method::Delete,
        "{{ base_url }}/{{ api_version }}/products/KB-042",
    );
    products.children = [list, get, create, update, delete]
        .into_iter()
        .map(Item::Request)
        .collect();

    let mut orders = Folder::new("Orders");
    orders.children.push(Item::Request(
        Request::new(
            "Upload invoice",
            Method::Post,
            "{{ base_url }}/{{ api_version }}/orders/1001/invoice",
        )
        .with_body(Body::Multipart {
            fields: vec![
                MultipartField {
                    name: "note".into(),
                    value: MultipartValue::Text { value: "Q3".into() },
                    enabled: true,
                },
                MultipartField {
                    name: "file".into(),
                    value: MultipartValue::File {
                        path: "invoice.pdf".into(),
                        content_type: None,
                    },
                    enabled: true,
                },
            ],
        }),
    ));
    orders.children.push(Item::Request(
        Request::new(
            "Search orders",
            Method::Get,
            "{{ base_url }}/{{ api_version }}/orders",
        )
        .param("status", "open"),
    ));

    let graphql = Request::new("Inventory (GraphQL)", Method::Post, "{{ base_url }}/graphql").with_body(Body::GraphQl {
        query: "query Inventory($first: Int!) {\n  products(first: $first) {\n    sku\n    name\n    stock\n  }\n}".into(),
        variables: "{ \"first\": 3 }".into(),
        operation_name: None,
    });
    let health = Request::new("Health check", Method::Get, "{{ base_url }}/health");
    ws.items = vec![
        Item::Folder(auth),
        Item::Folder(products),
        Item::Folder(orders),
        Item::Request(graphql),
        Item::Request(health),
    ];
    ws
}

/// A small local API so the demo shows real responses.
fn demo_server() -> u16 {
    let server = tiny_http::Server::http("127.0.0.1:0").expect("bind demo server");
    let port = server.server_addr().to_ip().map(|a| a.port()).unwrap_or(0);
    std::thread::spawn(move || {
        for mut request in server.incoming_requests() {
            let mut body = String::new();
            let _ = std::io::Read::read_to_string(request.as_reader(), &mut body);
            let path = request.url().split('?').next().unwrap_or("").to_owned();
            let (status, value) = match (request.method().as_str(), path.as_str()) {
                ("POST", "/v1/auth/login") => (
                    200,
                    json!({"token": "demo-session-token", "expires_in": 3600, "user": {"id": 7, "email": "ada@acme.test"}}),
                ),
                ("GET", "/v1/me") => (
                    200,
                    json!({"id": 7, "email": "ada@acme.test", "roles": ["admin", "buyer"]}),
                ),
                ("GET", "/v1/products") => (
                    200,
                    json!({
                        "items": [
                            {"sku": "KB-001", "name": "Mechanical Keyboard", "price": 129.0, "currency": "EUR", "in_stock": true, "tags": ["tactile", "rgb"]},
                            {"sku": "KB-007", "name": "Low-profile Keyboard", "price": 99.5, "currency": "EUR", "in_stock": false, "tags": ["slim"]},
                            {"sku": "KB-013", "name": "Travel Keyboard", "price": 74.0, "currency": "EUR", "in_stock": true, "tags": []}
                        ],
                        "page": {"limit": 20, "next": null, "total": 3}
                    }),
                ),
                ("GET", p) if p.starts_with("/v1/products/") => (
                    200,
                    json!({"sku": "KB-001", "name": "Mechanical Keyboard", "price": 129.0}),
                ),
                ("POST", "/v1/products") => (201, serde_json::from_str(&body).unwrap_or(json!({}))),
                ("PATCH", _) => (200, json!({"sku": "KB-042", "price": 169.0})),
                ("DELETE", _) => (204, json!(null)),
                ("POST", "/graphql") => (
                    200,
                    json!({"data": {"products": [{"sku": "KB-001", "name": "Mechanical Keyboard", "stock": 12}]}}),
                ),
                ("GET", "/health") => (200, json!({"status": "ok"})),
                _ => (404, json!({"error": "not_found", "path": path})),
            };
            let text = if value.is_null() {
                String::new()
            } else {
                value.to_string()
            };
            let response = tiny_http::Response::from_string(text)
                .with_status_code(status)
                .with_header(
                    tiny_http::Header::from_bytes(
                        &b"Content-Type"[..],
                        &b"application/json; charset=utf-8"[..],
                    )
                    .expect("static header"),
                )
                .with_header(
                    tiny_http::Header::from_bytes(
                        &b"Set-Cookie"[..],
                        &b"session=7f3a9c; Path=/; HttpOnly"[..],
                    )
                    .expect("static header"),
                )
                .with_header(
                    tiny_http::Header::from_bytes(&b"X-Request-Id"[..], &b"c0ffee-42"[..])
                        .expect("static header"),
                );
            let _ = request.respond(response);
        }
    });
    port
}
