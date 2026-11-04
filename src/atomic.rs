// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Maps holding fundamental atomic entities connected by a network of bonds.

mod graph;
mod spatial;
mod views;

pub(crate) mod data;
pub(crate) use spatial::SpatialAtomicCommon;

pub use graph::AtomMap;
pub use spatial::{AtomMap2, AtomMap3, SpatialAtomMap};

use crate::Map;

/// A map that holds fundamental atomic entities, and a network of bonds between them.
///
/// This applies to all maps, and so all map types in the crate implement this trait.
pub trait Atomic: Map {
    /// Returns the the core network of atoms and bonds without any spatial positions.
    fn into_atomic(self) -> AtomMap;

    /// Provides read-only access to the core network of atoms and bonds.
    fn as_atomic(&self) -> &AtomMap;
}
