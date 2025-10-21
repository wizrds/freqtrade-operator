// SPDX-FileCopyrightText: 2025 Timothy Pogue
//
// SPDX-License-Identifier: ISC

use tower_http::{
    LatencyUnit,
    trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse},
       classify::SharedClassifier,
       classify::ServerErrorsAsFailures,
};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time::ChronoUtc;
use tracing_subscriber::{EnvFilter, Layer};

pub use tracing::{error, info, warn, debug, trace};

// This function initializes the global logger
pub fn setup_logging() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .flatten_event(true)
        .with_timer(ChronoUtc::rfc_3339())
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(false)
        .boxed();

    let env_filter = EnvFilter::try_from_env("LOG_LEVEL")
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

/// This function creates a TraceLayer with a global configuration
/// for logging HTTP requests and responses.
pub fn create_trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(
            DefaultMakeSpan::new().include_headers(false)
        )
        .on_request(
            DefaultOnRequest::new().level(Level::INFO)
        )
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Millis)
        )
 }