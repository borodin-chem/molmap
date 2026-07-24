// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use nalgebra::{Point, Point2};
use slotmap::SecondaryMap;

use crate::{Element, graph::MolGraph, ids::AtomId, traits::MolMap};

/// A [`MolMap`] that also holds the spatial positions (with dimensionality `D`)
/// of its entities.
pub trait SpatialMolMap<const D: usize>: MolMap {
    fn atom_position(&self, id: AtomId) -> Point<f64, D>;
}
