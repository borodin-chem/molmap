// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use nalgebra::{Point, Point2};
use slotmap::SecondaryMap;

use crate::{AtomId, Element, MolMap, graph::MolGraph};

pub trait SpatialMolMap<const D: usize>: MolMap {
    fn atom_position(&self, id: AtomId) -> Point<f64, D>;
}

#[derive(Debug, Default)]
pub struct MolMap2 {
    pub(crate) core: MolGraph,
    pub(crate) atom_positions: SecondaryMap<AtomId, Point2<f64>>
}

impl MolMap for MolMap2 {
    fn new() -> Self {
        Self {
            core: MolGraph::new(),
            atom_positions: SecondaryMap::new(),
        }
    }

    fn with_capacity(n: usize) -> Self {
        Self {
            core: MolGraph::with_capacity(n),
            atom_positions: SecondaryMap::with_capacity(n),
        }
    }

    fn core(&self) -> &MolGraph {
        &self.core
    }

    fn core_mut(&mut self) -> &mut MolGraph {
        &mut self.core
    }
}

impl SpatialMolMap<2> for MolMap2 {
    fn atom_position(&self, id: AtomId) -> Point<f64, 2> {
        *self.atom_positions.get(id).unwrap()
    }
}

impl MolMap2 {
    pub fn add_atom(&mut self, element: Element, position: Point2<f64>) -> AtomId {
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

#[cfg(test)]
mod tests {
    use crate::Element;

    use super::*;

    #[test]
    fn atom_pos() {
        let mut mm = MolMap2::new();
        let c1 = mm.add_atom(Element::C, Point2::new(1.0, 2.0));
        let pos = mm.atom_position(c1);
        let positions = all_atom_positions(mm);
        let pos2 = positions.first().unwrap();
        assert_eq!(pos, *pos2);
    }
}
