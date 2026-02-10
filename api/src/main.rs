use axum::{Extension, Router, middleware, routing::post};
use std::{
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::net::TcpListener;

use axum::{body::Body, http::Request, middleware::Next, response::Response};

use zkcg_common::state::ProtocolState;
#[cfg(not(feature = "zk-vm"))]
use zkcg_verifier::backend_stub::StubBackend;
#[cfg(feature = "zk-vm")]
use zkcg_verifier::backend_zkvm::ZkVmBackend;
use zkcg_verifier::engine::VerifierEngine;

use api::handler::{
    AppState, compliance_evaluate_handler, demo_prove_handler, demo_verify_handler, prove,
    submit_proof,
};

mod rate_limit;
use rate_limit::RateLimiter;

async fn log_requests(req: Request<Body>, next: Next) -> Response {
    println!("[REQUEST] {} {}", req.method(), req.uri().path());
    let res = next.run(req).await;
    println!("[RESPONSE] status={}", res.status());
    res
}

#[tokio::main]
async fn main() {
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap();

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("[BOOT] starting ZKCG API");

    let engine = VerifierEngine::new(ProtocolState::genesis(), {
        #[cfg(feature = "zk-vm")]
        {
            Box::new(ZkVmBackend)
        }
        #[cfg(not(feature = "zk-vm"))]
        {
            Box::new(StubBackend::default())
        }
    });

    let app_state = AppState {
        engine: Arc::new(Mutex::new(engine)),
    };

    let prove_limiter = Arc::new(RateLimiter::new(5, Duration::from_secs(60)));
    let verify_limiter = Arc::new(RateLimiter::new(30, Duration::from_secs(60)));

    let demo_routes = Router::new()
        .route(
            "/demo/prove",
            post(demo_prove_handler)
                .route_layer(middleware::from_fn(RateLimiter::middleware))
                .route_layer(Extension(prove_limiter)),
        )
        .route(
            "/demo/verify",
            post(demo_verify_handler)
                .route_layer(middleware::from_fn(RateLimiter::middleware))
                .route_layer(Extension(verify_limiter.clone())),
        );

    let mut app = Router::new().merge(demo_routes).layer(Extension(app_state));

    if env::var("ZKCG_ENABLE_PROTOCOL").is_ok() {
        println!("[CONFIG] protocol endpoints ENABLED");
        app = app
            .route("/v1/submit-proof", post(submit_proof))
            .route("/v1/prove", post(prove))
            .route(
                "/v1/compliance/evaluate",
                post(compliance_evaluate_handler)
                    .route_layer(middleware::from_fn(RateLimiter::middleware))
                    .route_layer(Extension(verify_limiter)),
            );
    } else {
        println!("[CONFIG] protocol endpoints DISABLED");
    }

    let app = app.layer(middleware::from_fn(log_requests));

    println!("[LISTENING] {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}