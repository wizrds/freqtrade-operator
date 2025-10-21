// SPDX-FileCopyrightText: 2025 Timothy Pogue
//
// SPDX-License-Identifier: ISC

use std::{sync::Arc, time::Duration, net::SocketAddr};
use axum::{
    routing::get,
    Router,
    Extension,
};
use axum::response::{Json, IntoResponse};
use axum_server::tls_rustls::RustlsConfig;
use serde::Serialize;
use tokio::signal;

use ft_operator_common::constant::APP_NAME;
use ft_operator_common::state::State;
use ft_operator_common::telemetry::create_trace_layer;

use crate::router::v1::admission;

#[derive(Serialize)]
struct RootResponse {
    name: &'static str,
    version: &'static str,
}

pub fn create_router(app_state: Arc<State>) -> Router {
    Router::new()
        .nest("/admission", admission::router())
        .layer(Extension(app_state))
        .layer(create_trace_layer())
        // Root endpoint after the tracing layer to ensure
        // that the root endpoint is not traced
        .route("/", get(|| async {
            let response = RootResponse {
                name: APP_NAME,
                version: env!("CARGO_PKG_VERSION"),
            };
            Json(response).into_response()
        }))
}

pub async fn create_tls_config(cert_file: String, key_file: String) -> RustlsConfig {
    RustlsConfig::from_pem_file(cert_file, key_file)
        .await
        .expect("Failed to create TLS config")
}

pub async fn serve(addr: String, router: Router, tls_config: RustlsConfig) -> std::io::Result<()> {
    axum_server::bind_rustls(addr.parse::<SocketAddr>().expect("Invalid address"), tls_config)
        .serve(router.into_make_service())
        .await
}

pub async fn shutdown_signal(handle: axum_server::Handle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => (),
        _ = terminate => (),
    }

    handle.graceful_shutdown(Some(Duration::from_secs(10)));
}