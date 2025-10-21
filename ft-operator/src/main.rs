// SPDX-FileCopyrightText: 2025 Timothy Pogue
//
// SPDX-License-Identifier: ISC

mod cli;

use std::sync::Arc;
use futures::StreamExt;
use std::process;
use clap::Parser;
use clap::CommandFactory;
use rustls::crypto::aws_lc_rs;

use ft_operator_common::config::AppConfigBuilder;
use ft_operator_common::state::State;
use ft_operator_common::telemetry::{error, info, setup_logging};
use ft_operator_controller::controller::{context::Context, utils::{error_policy, create_k8s_client}, bot::BotController};
use ft_operator_controller::crd::{v1alpha1::bot::Bot as V1Alpha1Bot, utils as crd_utils};
use ft_operator_webhook::server::{create_router, create_tls_config, serve};

use crate::cli::{CliArgs, Commands};

#[tokio::main]
async fn main() {
    // Install the default aws_lc_rs crypto provider
    let _ = aws_lc_rs::default_provider().install_default();

    let args = CliArgs::parse();

    setup_logging();

    match &args.cmd {
        Some(Commands::Crds) => crd_utils::generate_crds(),
        Some(Commands::Webhook) => {
            info!(
                event = "Starting",
                version = env!("CARGO_PKG_VERSION"),
            );

            // Load configuration
            let config = AppConfigBuilder::default()
                .with_env()
                .build()
                .unwrap_or_else(|e| {
                    error!(
                        event = "Error",
                        error = %e,
                    );
                    process::exit(1);
                });

            // Create necessary resources
            let state = Arc::new(State { config: config.clone() });

            let addr = format!("{}:{}", config.webhook.host, config.webhook.port);
            let tls_config = create_tls_config(config.webhook.tls.cert_file.to_string(), config.webhook.tls.key_file.to_string()).await;
            let router = create_router(state.clone());

            // Run Webhook server
            info!(event = "Listening", address = addr.as_str());
            serve(addr, router, tls_config).await.unwrap_or_else(|e| {
                error!(
                    event = "Error",
                    error = %e,
                );
                process::exit(1);
            });
        },
        Some(Commands::Controller) => {
            info!(
                event = "Starting",
                version = env!("CARGO_PKG_VERSION"),
            );

            // Load configuration
            let config = AppConfigBuilder::default()
                .with_env()
                .build()
                .unwrap_or_else(|e| {
                    error!(
                        event = "Error",
                        error = %e,
                    );
                    process::exit(1);
                });

            // Create necessary resources
            let state = Arc::new(State { config: config.clone() });
            let client = create_k8s_client().await.unwrap_or_else(|e| {
                error!(
                    event = "Error",
                    error = %e,
                );
                process::exit(1);
            });
            let controller_ctx = Arc::new(Context::new(client).with_state(state.clone()));

            // Create CRD controllers
            let v1alpha1_bot_controller = BotController::create_controller::<V1Alpha1Bot>(controller_ctx.clone()).await;

            // Run CRD controllers
            info!(event = "ControllerStarted", kind = "Bot", version = "v1alpha1");
            let v1alpha1_bot_handle = tokio::spawn(async move {
                v1alpha1_bot_controller.run(BotController::reconcile::<V1Alpha1Bot>, error_policy::<V1Alpha1Bot>, controller_ctx.clone())
                    .for_each(|r| async move {
                        match r {
                            Ok(_) => info!(event = "Reconciled", kind = "Bot", version = "v1alpha1"),
                            Err(e) => error!(event = "ReconcileError", error = %e),
                        }
                    })
                    .await
            });

            // Wait for all controllers to finish
            match tokio::try_join!(v1alpha1_bot_handle) {
                Ok(_) => info!(event = "Stopped"),
                Err(e) => error!(event = "Error", error = %e),
            }
        },
        None => {
            let mut cmd = CliArgs::command();
            cmd.print_help().unwrap();
            process::exit(1);
        },
    }
}