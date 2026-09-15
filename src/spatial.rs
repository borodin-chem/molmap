// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Maps that hold not only the molecular graph but also the spatial positions
//! of its entities, and methods and functions to work with them.

mod euclidean;
mod generic;

pub(crate) use generic::Vector;

pub use generic::SpatialMolMap;

/// A [`MolMap`] that holds the positions of its entities in two dimensions.
pub type MolMap2 = SpatialMolMap<2>;

/// A [`MolMap`] that holds the positions of its entities in three dimensions.
pub type MolMap3 = SpatialMolMap<3>;
