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

mod element;
mod entities;
mod error;
mod graph;
mod maps;
mod pseudoelement;

pub mod traits;
pub mod views;

pub mod ids {
    pub use crate::entities::ids::*;
}

pub use element::Element;
pub use entities::bond::BondType;
pub use error::{MolMapError, MolMapResult};
pub use maps::{MolMap0, MolMap2, MolMap3};
pub use pseudoelement::Pseudoelement;
pub use traits::{MolMap, SpatialMolMap};
