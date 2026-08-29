// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use nalgebra::{Point, Point3};
use slotmap::SecondaryMap;

use crate::{
    graph::{MolGraph, keys::*},
    maps::{MolMapCore, spatial::*},
    *,
};

#[derive(Clone, Debug)]
pub struct MolMap3 {
    pub(crate) core: MolGraph,
    pub(crate) positions: Positions<3>,
}

impl MolMapCore for MolMap3 {
    #[inline]
    fn core(&self) -> &MolGraph {
        &self.core
    }

    #[inline]
    fn core_mut(&mut self) -> &mut MolGraph {
        &mut self.core
    }
}

impl MolMap for MolMap3 {
    fn new() -> Self {
        Self {
            core: MolGraph::new(),
            positions: Positions::new(),
        }
    }

    fn with_capacities(
        atoms: usize,
        pseudoatoms: usize,
        bonds: usize,
        substituents: usize,
        molecules: usize,
    ) -> Self {
        Self {
            core: MolGraph::with_capacities(atoms, pseudoatoms, bonds, substituents, molecules),
            positions: Positions::with_capacities(atoms, pseudoatoms, bonds),
        }
    }
}

impl SpatialMolMapCore<3> for MolMap3 {
    fn positions(&self) -> &Positions<3> {
        &self.positions
    }

    fn positions_mut(&mut self) -> &mut Positions<3> {
        &mut self.positions
    }
}

impl SpatialMolMap<3> for MolMap3 {}
