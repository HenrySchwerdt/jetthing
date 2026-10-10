//! End-to-end execution tests against a local HTTP server.

use nexus_http_core::assertions::{Assertion, Operator, Source, check_all};
use nexus_http_core::auth;
use nexus_http_core::cookies::CookieJar;
use nexus_http_core::exec::ExecError;
use nexus_http_core::history::History;
use nexus_http_core::model::{
    Auth, Body, Item, Method, OAuth2Config, OAuth2Grant, Request, Workspace, WorkspaceSettings,
};
use nexus_http_core::{Client, Response, execute};
use serde_json::{Value, json};
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;

struct TestServer {
    base: String,
}

fn header(name: &str, value: &str) -> tiny_http::Header {
    tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes()).unwrap()
}

fn start() -> TestServer {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();
    let base = format!("http://127.0.0.1:{port}");
    thread::spawn(move || {
        for mut request in server.incoming_requests() {
            let mut body = String::new();
            let _ = std::io::Read::read_to_string(request.as_reader(), &mut body);
            let url = request.url().to_owned();
            let path = url.split('?').next().unwrap_or("").to_owned();
            let headers: serde_json::Map<String, Value> = request
                .headers()
                .iter()
                .map(|h| {
                    (
                        h.field.as_str().as_str().to_ascii_lowercase(),
                        Value::String(h.value.as_str().to_owned()),
                    )
                })
                .collect();
            let get = |n: &str| {
                headers
                    .get(n)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_owned()
            };
            let echo = json!({
                "method": request.method().as_str(),
                "url": url,
                "headers": headers,
                "body": body,
            });
            let json_response = |status: u16, value: &Value| {
                tiny_http::Response::from_string(value.to_string())
                    .with_status_code(status)
                    .with_header(header("Content-Type", "application/json"))
            };
            let response = match path.as_str() {
                "/redirect-302" => tiny_http::Response::from_string("")
                    .with_status_code(302)
                    .with_header(header("Location", "/echo")),
                "/redirect-307" => tiny_http::Response::from_string("")
                    .with_status_code(307)
                    .with_header(header("Location", "/echo")),
                "/loop" => tiny_http::Response::from_string("")
                    .with_status_code(302)
                    .with_header(header("Location", "/loop")),
                "/set-cookie" => json_response(200, &json!({}))
                    .with_header(header("Set-Cookie", "session=abc; Path=/; HttpOnly")),
                "/login" => json_response(200, &json!({"token": "tok-123", "user": {"id": 7}})),
                "/slow" => {
                    thread::sleep(Duration::from_millis(1500));
                    json_response(200, &json!({}))
                }
                "/big" => tiny_http::Response::from_string("x".repeat(10_000)),
                "/digest" => {
                    let auth = get("authorization");
                    if auth.starts_with("Digest ")
                        && auth.contains("username=\"ada\"")
                        && auth.contains("nc=00000001")
                    {
                        json_response(200, &json!({"ok": true}))
                    } else {
                        json_response(401, &json!({})).with_header(header(
                            "WWW-Authenticate",
                            "Digest realm=\"test\", qop=\"auth\", nonce=\"n0nce\", opaque=\"op\"",
                        ))
                    }
                }
                "/token" => {
                    let form: std::collections::BTreeMap<String, String> =
                        url::form_urlencoded::parse(body.as_bytes())
                            .into_owned()
                            .collect();
                    if form.get("grant_type").map(String::as_str) == Some("client_credentials")
                        && form.get("client_id").map(String::as_str) == Some("cid")
                    {
                        json_response(
                            200,
                            &json!({"access_token": "oauth-tok", "token_type": "bearer", "expires_in": 3600}),
                        )
                    } else {
                        json_response(
                            400,
                            &json!({"error": "invalid_client", "error_description": "bad id"}),
                        )
                    }
                }
                _ => json_response(200, &echo),
            };
            let _ = request.respond(response);
        }
    });
    TestServer { base }
}

fn client() -> Client {
    let mut c = Client::new(WorkspaceSettings::default());
    c.use_system_proxy = false;
    c
}

fn run(
    ws: &Workspace,
    request: &Request,
    jar: Option<&mut CookieJar>,
    history: &History,
) -> Result<Response, ExecError> {
    let mut ws = ws.clone();
    ws.items.push(Item::Request(request.clone()));
    execute(
        &ws,
        &request.id,
        Path::new("."),
        &client(),
        jar,
        history,
        &AtomicBool::new(false),
    )
}

fn echo(response: &Response) -> Value {
    serde_json::from_slice(&response.body).unwrap()
}

fn workspace(server: &TestServer) -> Workspace {
    let mut ws = Workspace::new("Test");
    ws.base_environment = ws
        .base_environment
        .clone()
        .with("base", server.base.clone());
    ws
}

#[test]
fn sends_method_query_headers_and_json_body() {
    let server = start();
    let ws = workspace(&server);
    let req = Request::new("Create", Method::Post, "{{base}}/echo")
        .param("page", "2")
        .header("X-Trace", "{% base64 'encode', 'id' %}")
        .with_body(Body::Json {
            text: r#"{"name":"Ada"}"#.into(),
        });
    let response = run(&ws, &req, None, &History::default()).unwrap();
    assert_eq!(response.status, 200);
    let e = echo(&response);
    assert_eq!(e["method"], "POST");
    assert_eq!(e["url"], "/echo?page=2");
    assert_eq!(e["headers"]["x-trace"], "aWQ=");
    assert_eq!(e["headers"]["content-type"], "application/json");
    assert!(
        e["headers"]["user-agent"]
            .as_str()
            .unwrap()
            .starts_with("nexus-http/")
    );
    assert_eq!(e["body"], r#"{"name":"Ada"}"#);
    assert!(response.timings.total_ms >= response.timings.wait_ms);
    assert!(
        response
            .timeline
            .iter()
            .any(|t| t.text == "POST /echo?page=2 HTTP/1.1")
    );
}

#[test]
fn follows_redirects_with_method_rules() {
    let server = start();
    let ws = workspace(&server);
    let body = Body::Text {
        text: "payload".into(),
        content_type: None,
    };
    let r302 = run(
        &ws,
        &Request::new("a", Method::Post, "{{base}}/redirect-302").with_body(body.clone()),
        None,
        &History::default(),
    )
    .unwrap();
    assert_eq!(r302.redirects, 1);
    assert_eq!(echo(&r302)["method"], "GET");
    assert_eq!(echo(&r302)["body"], "");
    let r307 = run(
        &ws,
        &Request::new("b", Method::Post, "{{base}}/redirect-307").with_body(body),
        None,
        &History::default(),
    )
    .unwrap();
    assert_eq!(echo(&r307)["method"], "POST");
    assert_eq!(echo(&r307)["body"], "payload");

    let mut no_follow = Request::new("c", Method::Get, "{{base}}/redirect-302");
    no_follow.settings.follow_redirects = Some(false);
    assert_eq!(
        run(&ws, &no_follow, None, &History::default())
            .unwrap()
            .status,
        302
    );

    let err = run(
        &ws,
        &Request::new("d", Method::Get, "{{base}}/loop"),
        None,
        &History::default(),
    )
    .unwrap_err();
    assert!(matches!(err, ExecError::TooManyRedirects(10)), "{err}");
}

#[test]
fn stores_and_sends_cookies() {
    let server = start();
    let ws = workspace(&server);
    let mut jar = CookieJar::default();
    run(
        &ws,
        &Request::new("set", Method::Get, "{{base}}/set-cookie"),
        Some(&mut jar),
        &History::default(),
    )
    .unwrap();
    assert_eq!(jar.cookies.len(), 1);
    let r = run(
        &ws,
        &Request::new("get", Method::Get, "{{base}}/echo"),
        Some(&mut jar),
        &History::default(),
    )
    .unwrap();
    assert_eq!(echo(&r)["headers"]["cookie"], "session=abc");

    let mut off = Request::new("off", Method::Get, "{{base}}/echo");
    off.settings.disable_cookies = true;
    let r = run(&ws, &off, Some(&mut jar), &History::default()).unwrap();
    assert!(echo(&r)["headers"].get("cookie").is_none());
}

#[test]
fn chains_responses_into_later_requests() {
    let server = start();
    let ws = workspace(&server);
    let login = Request::new("login", Method::Post, "{{base}}/login");
    let mut history = History::default();
    let response = run(&ws, &login, None, &history).unwrap();
    history.push(login.id.clone(), response);
    let me = Request::new("me", Method::Get, "{{base}}/echo").with_auth(Auth::Bearer {
        token: format!("{{% response 'body', '{}', '$.token' %}}", login.id),
        prefix: "Bearer".into(),
    });
    let r = run(&ws, &me, None, &history).unwrap();
    assert_eq!(echo(&r)["headers"]["authorization"], "Bearer tok-123");

    let results = check_all(
        &[
            Assertion::new(Source::Status, Operator::Equals, "200"),
            Assertion::new(Source::JsonPath("$.method".into()), Operator::Equals, "GET"),
        ],
        &r,
    );
    assert!(results.iter().all(|r| r.passed), "{results:?}");
}

#[test]
fn answers_digest_challenge_and_applies_basic() {
    let server = start();
    let ws = workspace(&server);
    let digest = Request::new("d", Method::Get, "{{base}}/digest").with_auth(Auth::Digest {
        username: "ada".into(),
        password: "pw".into(),
    });
    let r = run(&ws, &digest, None, &History::default()).unwrap();
    assert_eq!(r.status, 200, "{}", r.body_text());

    let basic = Request::new("b", Method::Get, "{{base}}/echo").with_auth(Auth::Basic {
        username: "u".into(),
        password: "p".into(),
    });
    let r = run(&ws, &basic, None, &History::default()).unwrap();
    let header = echo(&r)["headers"]["authorization"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(header, auth::basic_header("u", "p"));
    assert!(header.starts_with("Basic "));
}

#[test]
fn signs_aws_requests() {
    let server = start();
    let ws = workspace(&server);
    let req = Request::new("s3", Method::Get, "{{base}}/echo?b=2&a=1").with_auth(Auth::AwsSigV4 {
        access_key_id: "AKID".into(),
        secret_access_key: "secret".into(),
        session_token: "sess".into(),
        region: "eu-central-1".into(),
        service: "execute-api".into(),
    });
    let e = echo(&run(&ws, &req, None, &History::default()).unwrap());
    let auth = e["headers"]["authorization"].as_str().unwrap();
    assert!(
        auth.starts_with("AWS4-HMAC-SHA256 Credential=AKID/"),
        "{auth}"
    );
    assert!(auth.contains("/eu-central-1/execute-api/aws4_request"));
    assert!(auth.contains("SignedHeaders=host;x-amz-date;x-amz-security-token"));
    assert_eq!(e["headers"]["x-amz-security-token"], "sess");
}

#[test]
fn fetches_oauth2_client_credentials_token() {
    let server = start();
    let config = OAuth2Config {
        grant: OAuth2Grant::ClientCredentials,
        access_token_url: format!("{}/token", server.base),
        client_id: "cid".into(),
        client_secret: "sec".into(),
        ..Default::default()
    };
    let request = auth::token_request(&config, None, None).unwrap();
    let token = client()
        .fetch_oauth_token(&request, &AtomicBool::new(false))
        .unwrap();
    assert_eq!(token.access_token, "oauth-tok");
    assert!(token.expires_at.is_some());

    let bad = OAuth2Config {
        client_id: "nope".into(),
        ..config
    };
    let err = client()
        .fetch_oauth_token(
            &auth::token_request(&bad, None, None).unwrap(),
            &AtomicBool::new(false),
        )
        .unwrap_err();
    assert!(err.to_string().contains("invalid_client"), "{err}");
}

#[test]
fn reports_timeouts_connection_errors_and_truncation() {
    let server = start();
    let ws = workspace(&server);
    let mut slow = Request::new("slow", Method::Get, "{{base}}/slow");
    slow.settings.timeout_ms = Some(200);
    let err = run(&ws, &slow, None, &History::default()).unwrap_err();
    assert!(matches!(err, ExecError::Timeout(200)), "{err}");

    // A port that was just freed refuses connections.
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let refused = Request::new("x", Method::Get, format!("http://127.0.0.1:{port}/"));
    let err = run(&ws, &refused, None, &History::default()).unwrap_err();
    assert!(matches!(err, ExecError::Connect { .. }), "{err}");

    let mut small = ws.clone();
    small.settings.max_response_bytes = 1000;
    let r = run(
        &small,
        &Request::new("big", Method::Get, "{{base}}/big"),
        None,
        &History::default(),
    )
    .unwrap();
    assert!(r.truncated);
    assert_eq!(r.body.len(), 1000);
}

#[test]
fn cancels_before_sending() {
    let server = start();
    let mut ws = workspace(&server);
    let req = Request::new("c", Method::Get, "{{base}}/echo");
    ws.items.push(Item::Request(req.clone()));
    let err = execute(
        &ws,
        &req.id,
        Path::new("."),
        &client(),
        None,
        &History::default(),
        &AtomicBool::new(true),
    )
    .unwrap_err();
    assert!(matches!(err, ExecError::Cancelled));
}
