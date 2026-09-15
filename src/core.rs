// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Core types for the crate.
//!
//! This module is private to the crate, but various things are re-exported from
//! it at the top level.

mod categories;
mod element;
mod entities;
mod error;
mod id;
mod molmap;
mod pseudoelement;
mod view;

// Everything that is regularly needed crate-wide is re-exported

// Re-exports that are pub to enable re-export at the crate level
pub use categories::*;
pub use element::Element;
pub use entities::*;
pub use error::{MolMapError, MolMapResult};
pub use molmap::{MolMap, MolMapCore};
pub use pseudoelement::Pseudoelement;
pub use view::*;
