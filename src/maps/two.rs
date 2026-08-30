// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//use nalgebra::{Point, Point2};
//use slotmap::SecondaryMap;
//
//use crate::{
//    graph::{MolGraph, keys::*},
//    maps::{MolMapCore, spatial::*},
//    *,
//};
//
//#[derive(Clone, Debug)]
//pub struct MolMap2 {
//    pub(crate) core: MolGraph,
//    pub(crate) positions: Positions<2>,
//}
//
//impl MolMapCore for MolMap2 {
//    #[inline]
//    fn core(&self) -> &MolGraph {
//        &self.core
//    }
//
//    #[inline]
//    fn core_mut(&mut self) -> &mut MolGraph {
//        &mut self.core
//    }
//}
//
//impl MolMap for MolMap2 {
//    fn new() -> Self {
//        Self {
//            core: MolGraph::new(),
//            positions: Positions::new(),
//        }
//    }
//
//    fn with_capacities(
//        atoms: usize,
//        pseudoatoms: usize,
//        bonds: usize,
//        substituents: usize,
//        molecules: usize,
//    ) -> Self {
//        Self {
//            core: MolGraph::with_capacities(atoms, pseudoatoms, bonds, substituents, molecules),
//            positions: Positions::with_capacities(atoms, pseudoatoms, bonds),
//        }
//    }
//}
//
//impl SpatialMolMapCore<2> for MolMap2 {
//    fn positions(&self) -> &Positions<2> {
//        &self.positions
//    }
//
//    fn positions_mut(&mut self) -> &mut Positions<2> {
//        &mut self.positions
//    }
//}
//
//impl SpatialMolMap<2> for MolMap2 {}
//
//#[cfg(test)]
//mod tests {
//    use crate::Element;
//
//    use super::*;
//
//    //#[test]
//    //fn atom_pos() {
//    //    let mut mm = MolMap2::new();
//    //    let c1 = mm.add_atom(Element::C, Point2::new(1.0, 2.0));
//    //    let pos = mm.atom_position(c1);
//    //    let positions = all_atom_positions(mm);
//    //    let pos2 = positions.first().unwrap();
//    //    assert_eq!(pos, *pos2);
//    //}
//}
