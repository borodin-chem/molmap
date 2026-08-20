// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{graph::MolGraph, traits::MolMapCore, *};

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
    pub fn add_atom(&mut self, element: Element) -> Id<Atom> {
        self.core.add_atom(element)
    }

    /// Adds an atom to the map along with the requested number of hydrogen atoms
    /// (and single covalent bonds from the central atom to them).
    ///
    /// Returns the ID of the added central atom as well as a slice over the IDs
    /// of the new bonds.
    pub fn add_atom_with_hydrogen(
        &mut self,
        element: Element,
        n_hydrogen: u8,
    ) -> (Id<Atom>, &[Id<Bond>]) {
        let centre = self.add_atom(element);
        for i in 0..n_hydrogen {
            let new_h = self.add_atom(Element::H);
            self.core.add_bond(centre.into(), new_h.into());
        }
        // Don't waste memory allocating a new Vec to hold the bond IDs, since
        // they are already stored on the new central atom – return a slice instead
        (
            centre,
            self.core()
                .atoms
                .get(centre)
                .expect("We just created this atom, so ID must be valid")
                .bonds
                .as_slice(),
        )
    }

    /// Adds an atom to the map along with the number of additional hydrogen atoms
    /// required to satisfy the most common valency for the central atom (and single
    /// covalent bonds from the central atom to them).
    ///
    /// Returns the ID of the added central atom as well as a slice over the IDs
    /// of the new bonds.
    pub fn add_atom_and_saturate(&mut self, element: Element) -> (Id<Atom>, &[Id<Bond>]) {
        let n_hydrogen = element.default_valency();
        self.add_atom_with_hydrogen(element, n_hydrogen)
    }

    /// Adds a pseudoatom to the map.
    pub fn add_pseudoatom(&mut self, pseudoelement: Pseudoelement) -> Id<Pseudoatom> {
        self.core.add_pseudoatom(pseudoelement)
    }

    /// Creates a new (single covalent) bond between two bondable entities.
    ///
    /// # Errors
    ///
    /// Fails if either of `start` and `end` are invalid.
    pub fn add_bond(&mut self, start: Id<Bondable>, end: Id<Bondable>) -> MolMapResult<Id<Bond>> {
        if !self.core.contains_bondable(start) {
            return Err(MolMapError::Id(start.into()));
        } else if !self.core.contains_bondable(end) {
            return Err(MolMapError::Id(end.into()));
        };
        Ok(self.core.add_bond(start, end))
    }

    /// Adds an empty substituent to the map.
    pub fn add_substituent(&mut self) -> Id<Substituent> {
        self.core.add_substituent()
    }

    /// Adds a substituent to the map with a single, newly-created central atom.
    ///
    /// Returns the IDs of the added substituent and central atom.
    pub fn add_substituent_with_atom(&mut self, element: Element) -> (Id<Substituent>, Id<Atom>) {
        let centre = self.add_atom(element);
        let sub = self.core.add_substituent_with_centre(centre.into());
        (sub, centre)
    }

    /// Adds a substituent to the map with a newly-created central atom and the
    /// requested number of peripheral hydrogen atoms
    /// (and single covalent bonds from the central atom to them).
    ///
    /// Returns the IDs of the added substituent, central atom, and a slice over
    /// the IDs of the new bonds.
    pub fn add_substituent_with_hydrogen(
        &mut self,
        element: Element,
        n_hydrogen: u8,
    ) -> (Id<Substituent>, Id<Atom>, &[Id<Bond>]) {
        let (sub, centre) = self.add_substituent_with_atom(element);
        for i in 0..n_hydrogen {
            let new_h = self.add_atom(Element::H);
            let new_bond = self.core.add_bond(centre.into(), new_h.into());
            self.core.insert_into_substituent(sub, new_h.into());
            self.core.insert_into_substituent(sub, new_bond.into());
        }
        (
            sub,
            centre,
            self.core()
                .atoms
                .get(centre)
                .expect("We just created this atom, so ID must be valid")
                .bonds
                .as_slice(),
        )
    }

    /// Adds a substituent to the map with a newly-created central atom and the
    /// number of additional hydrogen atoms required to satisfy the most common
    /// valency for the central atom (and single covalent bonds from the central
    /// atom to them).
    ///
    /// Returns the ID of the added substituent, central atom, and a slice over
    /// the IDs of the new bonds.
    pub fn add_substituent_and_saturate(
        &mut self,
        element: Element,
    ) -> (Id<Substituent>, Id<Atom>, &[Id<Bond>]) {
        let n_hydrogen = element.default_valency();
        self.add_substituent_with_hydrogen(element, n_hydrogen)
    }

    /// Adds an empty molecule to the map.
    pub fn add_molecule(&mut self) -> Id<Molecule> {
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

    //#[test]
    //fn add_substituent() {
    //    let mut mm = MolMap0::new();
    //    let h1 = mm.add_atom(Element::H);
    //    let h2 = mm.add_atom(Element::H);
    //    let h3 = mm.add_atom(Element::H);
    //    let h4 = mm.add_atom(Element::H);
    //    let c1 = mm.add_atom(Element::C);
    //    let sub = mm.add_substituent();
    //}
}
