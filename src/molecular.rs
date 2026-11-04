// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Maps in which the atomic entities and bonds are organized into substituents
//! and molecules.

mod graph;
mod spatial;
mod views;

pub(crate) mod data;
#[expect(unused)]
pub(crate) use spatial::SpatialMolecularCommon;

pub use graph::MolMap;
pub use spatial::{MolMap2, MolMap3, SpatialMolMap};

use crate::Map;

/// A map in which the atomic entities and bonds are organized into substituents
/// and molecules.
pub trait Molecular: Map {
    /// Returns the molecular graph without any spatial positions.
    fn into_molecular(self) -> MolMap;

    /// Provides read-only access to the molecular graph.
    fn as_molecular(&self) -> &MolMap;
}
