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

#![warn(unreachable_pub)]

// Private modules
// ---------------
mod element;
mod error;
mod pseudoelement;
mod view;

pub(crate) mod geometry;
pub(crate) mod traits;

// ----------
// Public API
// ----------

// Publicly accessible modules
// ---------------------------

// General
pub mod entities;
pub mod graph;
pub mod parse;
pub mod units;

// Layers
pub mod atomic;
//pub mod crystalline;
pub mod molecular;
//pub mod reaction;

// Top-level items
// ---------------
pub use atomic::data::BondType;
pub use atomic::{AtomMap, AtomMap2, AtomMap3};
pub use element::Element;
pub use entities::{Atom, Bond, Molecule, Pseudoatom, Substituent};
//pub use molecular::{MolMap, MolMap2, MolMap3};
pub use error::{MolMapError, MolMapResult};
pub use geometry::{Point, Point2, Point3, Vector, Vector2, Vector3};
pub use pseudoelement::Pseudoelement;
pub use traits::{Map, Spatial, ThreeDimensional, TwoDimensional};
pub use view::{View, ViewMut, Views};

// Foreign re-exports
// ------------------
// Foreign crates or things from them
// Re-exporting nalgebra makes it easier for others to use
pub use nalgebra;
