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
    Element, MolMap,
    graph::MolGraph,
    ids::*,
    traits::{MolMapCore, SpatialMolMap},
};

#[derive(Clone, Debug, Default)]
pub struct MolMap3 {
    pub(crate) core: MolGraph,
    pub(crate) atom_positions: SecondaryMap<AtomId, Point3<f64>>,
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
            atom_positions: SecondaryMap::new(),
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
            atom_positions: SecondaryMap::with_capacity(atoms),
        }
    }
}

impl SpatialMolMap<3> for MolMap3 {
    fn atom_position(&self, id: AtomId) -> Point<f64, 3> {
        *self.atom_positions.get(id).unwrap()
    }
}

impl MolMap3 {
    pub fn add_atom(&mut self, element: Element, position: Point3<f64>) -> AtomId {
        let new_id = self.core.add_atom(element);
        self.atom_positions.insert(new_id, position);
        new_id
    }
}

fn all_atom_positions<const D: usize, M: SpatialMolMap<{ D }>>(map: M) -> Vec<Point<f64, { D }>> {
    let mut result = Vec::new();
    for id in map.atom_ids() {
        result.push(map.atom_position(id));
    }
    result
}
