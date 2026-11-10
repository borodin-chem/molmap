// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Notes for contributors:
//
// - Use `where` clauses for methods on maps with multiple generic entities, use
//   `impl Trait` when there is a single target that happens to be generic
// - Internal code should not import from the crate root, only from a level 1
//   module, or its own module via `super` (which is the preference for things
//   from the same module)

#![warn(unreachable_pub)]

// ------------------
// Crate-internal API
// ------------------
pub(crate) mod bonding;
pub(crate) mod error;
pub(crate) mod traits;
pub(crate) mod view;

// ----------
// Public API
// ----------

// Publicly accessible modules
// ---------------------------
// General
pub mod elements;
pub mod entities;
pub mod geometry;
pub mod graph;
pub mod iter;
pub mod parse;
pub mod units;
// Layers
pub mod atomic;
pub mod molecular;
//pub mod crystalline;
//pub mod reaction;

// Top-level items
// ---------------
// Only stuff that is universal to the whole crate and isn't exposed publicly elsewhere
pub use error::{MolMapError, MolMapResult};
pub use traits::{Map, Spatial};
pub use view::{View, ViewMut};
