// SPDX-FileCopyrightText: 2025 Timothy Pogue
//
// SPDX-License-Identifier: ISC

use crate::config::AppConfig;

#[derive(Clone, Default, Debug)]
pub struct State {
    pub config: AppConfig,
}
