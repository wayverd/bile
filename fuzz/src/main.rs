use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use http_body_util::BodyExt as _;
use tower::util::ServiceExt as _;

fn main() {
    afl::fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            if let Ok(uri) = http::Uri::try_from(s) {
                let Ok(rt) = tokio::runtime::Runtime::new() else {
                    println!("failed to create tokio runtime");
                    return;
                };

                rt.block_on(async {
                    let Ok(req) = Request::builder().uri(uri).body(Body::empty()) else {
                        println!("failed to create http request");
                        return;
                    };

                    let bile =
                        bile::Bile::init(bile::config::Config::default().finalize().unwrap());

                    let Ok(res) = bile.routes().oneshot(req).await else {
                        println!("failed to send http request");
                        return;
                    };

                    let status = res.status();

                    let body = body::to_bytes(res.into_body(), usize::MAX)
                        .await
                        .ok()
                        .and_then(|bytes| String::from_utf8(bytes.to_vec()).ok())
                        .unwrap_or_else(|| "BODY PARSE ERROR".to_string());

                    assert_ne!(status, StatusCode::INTERNAL_SERVER_ERROR, "{}", body);
                });
            }
        }
    });
}
