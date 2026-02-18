use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt as _;
use tower::util::ServiceExt as _;

fn main() {
    bile::set_config(bile::config::Config::default().finalize().unwrap());

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

                    let Ok(res) = bile::routes().oneshot(req).await else {
                        println!("failed to send http request");
                        return;
                    };

                    assert_ne!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
                });
            }
        }
    });
}
