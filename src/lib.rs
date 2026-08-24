// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(unused)]

// Re-export nalgebra to make it easier for others to use
pub use nalgebra;

mod definition;
mod element;
mod error;
mod graph;
mod id;
mod maps;
mod pseudoelement;
mod traits;
mod view;

pub mod categories;
pub mod entities;

pub(crate) use categories::*;
pub use element::Element;
pub(crate) use entities::Keyed;
pub use entities::bond::BondType;
pub use entities::{Atom, Bond, Molecule, Pseudoatom, Substituent};
pub use entities::{Entity, EntityKind};
pub use error::{MolMapError, MolMapResult};
pub use maps::{MolMap0, MolMap2, MolMap3};
pub use pseudoelement::Pseudoelement;
pub use traits::{MolMap, SpatialMolMap};
pub use view::{View, ViewMut};
