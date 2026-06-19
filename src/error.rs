// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::ids::{EntityId, FundamentalId};

/// Errors specific to the crate.
#[derive(thiserror::Error, Debug)]
pub enum MolMapError {
    #[error("The Id was not found in the Map")]
    Id(EntityId),
    #[error("The fundamental is not a member of this collection")]
    Membership(FundamentalId),
    #[error("The operation was not allowed")]
    Disallowed(String),
}

/// A `Result` type for situations where the crate's [`MolMapError`] might be returned.
pub type MolMapResult<T> = core::result::Result<T, MolMapError>;
