// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Notes for contributors:
//
// - For collections of IDs of multiple kinds of entity of a category `Category`,
//   use the corresponding `AnyCategory` type rather than `dyn Category`
// - For function parameters that take an entity ID, and the entity may be one
//   of multiple kinds of entity of a category `Category`, take a generic
//   parameter for convenience, and use `impl Category` syntax rather than a
//   named generic parameter
// - For generic function parameters that facilitate a wider range of input
//   types via simple conversions e.g. `Into`, `IntoIterator`, also prefer the
//   anonymous `impl Trait` style
// - Exceptions to the above are situations where the function's return type
//   should be linked to one of the input types
// - Use named generic parameters for all other cases
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
pub mod algorithms;
pub mod elements;
pub mod entities;
pub mod geometry;
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
pub use error::Error;
pub use traits::{Map, Spatial};
pub use view::{View, ViewMut};
