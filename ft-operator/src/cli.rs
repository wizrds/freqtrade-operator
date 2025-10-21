// SPDX-FileCopyrightText: 2025 Timothy Pogue
//
// SPDX-License-Identifier: ISC

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[
    clap(
        name = "freqtrade-operator",
        version,
        author,
        about = "Operator for managing Freqtrade instances"
    )
]
pub struct CliArgs {
    #[clap(subcommand)]
    pub cmd: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[
        clap(
            name = "crds",
            about = "Generate Custom Resource Definitions (CRDs) for the operator"
        )
    ]
    Crds,
    #[
        clap(
            name = "controller",
            about = "Run the controller"
        )
    ]
    Controller,
    #[
        clap(
            name = "webhook",
            about = "Run the admission webhook server",
        )
    ]
    Webhook,
}
