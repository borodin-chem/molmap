// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    Element, MolMapError, MolMapResult, Pseudoelement,
    graph::MolGraph,
    ids::{AtomId, AtomlikeId, BondId, BondableId, MoleculeId, PseudoatomId, SubstituentId},
    pseudoelement,
    traits::{MolMap, MolMapCore},
};

/// A pure molecular graph, without spatial positions.
#[derive(Clone, Debug, Default)]
pub struct MolMap0 {
    pub(crate) core: MolGraph,
}

impl MolMapCore for MolMap0 {
    #[inline]
    fn core(&self) -> &MolGraph {
        &self.core
    }

    #[inline]
    fn core_mut(&mut self) -> &mut MolGraph {
        &mut self.core
    }
}

impl MolMap for MolMap0 {
    fn new() -> Self {
        Self {
            core: MolGraph::new(),
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
        }
    }
}

impl MolMap0 {
    // Methods to add entities

    /// Adds an atom to the map.
    pub fn add_atom(&mut self, element: Element) -> AtomId {
        self.core.add_atom(element)
    }

    /// Adds a pseudoatom to the map.
    pub fn add_pseudoatom(&mut self, pseudoelement: Pseudoelement) -> PseudoatomId {
        self.core.add_pseudoatom(pseudoelement)
    }

    /// Creates a new (single covalent) bond between two bondable entities.
    ///
    /// Fails if either of `start` and `end` are invalid.
    pub fn add_bond(&mut self, start: BondableId, end: BondableId) -> MolMapResult<BondId> {
        if !self.core.contains_bondable(start) {
            return Err(MolMapError::Id(start.into()));
        } else if !self.core.contains_bondable(end) {
            return Err(MolMapError::Id(end.into()));
        };
        Ok(self.core.add_bond(start, end))
    }

    /// Adds a substituent to the map with a single central atom.
    ///
    /// Fails if `centre` is invalid.
    pub fn add_substituent(&mut self, centre: AtomlikeId) -> MolMapResult<SubstituentId> {
        if !self.core.contains_atomlike(centre) {
            return Err(MolMapError::Id(centre.into()));
        }
        Ok(self.core.add_substituent_with_centre(centre))
    }

    /// Adds an empty molecule to the map.
    pub fn add_molecule(&mut self) -> MoleculeId {
        self.core.add_molecule()
    }
}

#[cfg(test)]
mod tests {
    use crate::Element;

    use super::*;

    /// Creates a basic map to use as the basis for various tests.
    ///
    /// The map contains:
    /// - one molecule (CH3OH)
    /// - two substituents (CH3, OH) (n.b. not yet implemented)
    /// - six atoms
    /// - five bonds
    fn meoh_map() -> MolMap0 {
        let mut mm = MolMap0::new();
        let h1 = mm.add_atom(Element::H);
        let h2 = mm.add_atom(Element::H);
        let h3 = mm.add_atom(Element::H);
        let c1 = mm.add_atom(Element::C);
        let c1h1 = mm.add_bond(c1.into(), h1.into()).unwrap();
        let c1h2 = mm.add_bond(c1.into(), h2.into()).unwrap();
        let c1h3 = mm.add_bond(c1.into(), h3.into()).unwrap();
        let o1 = mm.add_atom(Element::O);
        let h4 = mm.add_atom(Element::H);
        let o1h4 = mm.add_bond(o1.into(), h4.into()).unwrap();
        let c1o1 = mm.add_bond(c1.into(), o1.into()).unwrap();
        // TODO substituents
        mm
    }

    #[test]
    fn add_atom() {
        let mut mm = MolMap0::new();
        assert!(mm.core.atoms.is_empty());
        let h1 = mm.add_atom(Element::H);
        assert_eq!(mm.core.atoms.len(), 1);
        let c1 = mm.add_atom(Element::C);
        assert_eq!(mm.core.atoms.len(), 2);
        // Check the atoms can be accessed by their ID, and that the elements are correct
        assert_eq!(mm.core.atoms.get(h1).unwrap().element, Element::H);
        assert_eq!(mm.core.atoms.get(c1).unwrap().element, Element::C);
        // Check that the bond arrays are created empty
        assert!(mm.core.atoms.get(h1).unwrap().bonds.is_empty());
    }

    //#[test]
    //fn add_pseudoatom() {
    //    let mut mm = MolMap0::new();
    //    assert!(mm.core.pseudoatoms.is_empty());
    //    let r1 = mm.add_pseudoatom("R");
    //    assert_eq!(mm.core.pseudoatoms.len(), 1);
    //    // Check the pseudoatom can be accessed by its ID, and that the symbol is correct
    //    assert_eq!(mm.core.pseudoatoms.get(r1).unwrap().symbol, "R");
    //    // Check that the bond arrays are created empty
    //    assert!(mm.core.pseudoatoms.get(r1).unwrap().bonds.is_empty());
    //}

    //#[test]
    //fn remove_atom() {
    //    let mut mm = MolMap0::new();
    //    let h1 = mm.add_atom(Element::H);
    //    let c1 = mm.add_atom(Element::C);
    //    assert_eq!(mm.core.atoms.len(), 2);
    //    mm.remove_atom(h1);
    //    assert_eq!(mm.core.atoms.len(), 1);
    //}

    //#[test]
    //fn remove_pseudoatom() {
    //    let mut mm = MolMap0::new();
    //    let r1 = mm.add_pseudoatom("R");
    //    assert_eq!(mm.core.pseudoatoms.len(), 1);
    //    mm.remove_pseudoatom(r1);
    //    assert!(mm.core.pseudoatoms.is_empty());
    //}

    #[test]
    fn add_bond_between_atoms() {
        let mut mm = MolMap0::new();
        assert!(mm.core.bonds.is_empty());
        let h1 = mm.add_atom(Element::H);
        let h2 = mm.add_atom(Element::H);
        let b1 = mm.add_bond(h1.into(), h2.into()).unwrap();
        assert!(mm.core.bonds.contains_key(b1));
        assert!(mm.core.atoms.get(h1).unwrap().bonds.contains(&b1));
        assert!(mm.core.atoms.get(h2).unwrap().bonds.contains(&b1));
        assert_eq!(mm.core.bonds.get(b1).unwrap().start, h1.into());
        assert_eq!(mm.core.bonds.get(b1).unwrap().end, h2.into());
    }

    //#[test]
    //fn remove_bond_between_atoms() {
    //    let mut mm = MolMap0::new();
    //    let h1 = mm.add_atom(Element::H);
    //    let h2 = mm.add_atom(Element::H);
    //    let b1 = mm.add_bond(h1.into(), h2.into()).unwrap();
    //    assert!(mm.core.bonds.contains_key(b1));
    //    assert!(mm.core.atoms.get(h1).unwrap().bonds.contains(&b1));
    //    assert!(mm.core.atoms.get(h2).unwrap().bonds.contains(&b1));
    //    assert_eq!(mm.core.bonds.get(b1).unwrap().start, h1.into());
    //    assert_eq!(mm.core.bonds.get(b1).unwrap().end, h2.into());
    //    mm.remove_bond(b1);
    //    assert!(!mm.core.bonds.contains_key(b1));
    //    assert!(!mm.core.atoms.get(h1).unwrap().bonds.contains(&b1));
    //    assert!(!mm.core.atoms.get(h2).unwrap().bonds.contains(&b1));
    //}

    #[test]
    fn add_substituent() {
        let mut mm = MolMap0::new();
        let h1 = mm.add_atom(Element::H);
        let h2 = mm.add_atom(Element::H);
        let h3 = mm.add_atom(Element::H);
        let h4 = mm.add_atom(Element::H);
        let c1 = mm.add_atom(Element::C);
        let sub = mm.add_substituent(c1.into()).unwrap();
    }
}
