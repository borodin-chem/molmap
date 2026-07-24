// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::ids::{EntityId, EntityKind, FundamentalId};

/// Errors specific to the crate.
#[derive(thiserror::Error, Debug)]
pub enum MolMapError {
    /// Returned when an ID is invalid.
    #[error("The ID was not found in the Map")]
    Id(EntityId),
    /// Returned when a fundamental is not in fact a member of a specific collection.
    #[error("The fundamental is not a member of this collection")]
    Membership(FundamentalId),
    /// General error returned when a disallowed operation is attempted.
    #[error("The operation was not allowed")]
    Disallowed(String),
    /// Occurs in the rare case that converting a `u8` to an [`EntityKind`] fails
    /// because it does not correspond to a valid discriminant.
    #[error("Not a recognized kind of entity")]
    UnknownEntityKind(u8),
    /// Returned when attempting to convert an entity kind enum to a kind
    /// that it is not a subset of.
    #[error("The entity kind is not within the subset of possible entity kinds")]
    IncorrectEntityKind(EntityKind, EntityId),
}

/// A `Result` type for situations where the crate's [`MolMapError`] might be returned.
pub type MolMapResult<T> = core::result::Result<T, MolMapError>;
