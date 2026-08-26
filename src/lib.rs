// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(unused)]

// Private modules
// ---------------
mod definition;
mod element;
mod graph;
mod id;
mod maps;
mod pseudoelement;

// ----------
// Public API
// ----------

// Publicly accessible modules
// ---------------------------
pub mod categories;
pub mod entities;
pub mod error;
pub mod view;

// Top-level items
// ---------------
pub use element::Element;
pub use graph::entities::BondType;
pub use maps::{MolMap, SpatialMolMap}; // Traits
pub use maps::{MolMap0, MolMap2, MolMap3}; // Common map types
pub use pseudoelement::Pseudoelement;

// Selected re-exports from public modules
// ---------------------------------------
pub use entities::{Atom, Bond, Molecule, Pseudoatom, Substituent}; // All the basic kinds of entity

// Foreign re-exports
// ------------------
// Foreign crates or things from them
// Re-exporting nalgebra makes it easier for others to use
pub use nalgebra;
