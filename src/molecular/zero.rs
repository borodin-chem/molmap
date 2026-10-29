// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    Map,
    atomic::{AtomCore, AtomGraph, AtomMap},
    entities::*,
    traits::{Core, Graph},
};

use super::{MolCore, MolGraph, MolMap};

/// A map in which the atomic entities and bonds are organized into substituents
/// and molecules, in a pure-graph representation.
#[derive(Clone, Debug, Default)]
pub struct MolMap0 {
    graph: MolGraph,
}

impl MolMap0 {
    pub fn new() -> Self {
        Self {
            graph: MolGraph::new(),
        }
    }

    pub fn contains<E: Entity>(&self, entity: E) -> bool {
        self.graph.contains(entity)
    }
}

impl Core for MolMap0 {
    type Graph = MolGraph;

    fn graph(&self) -> &MolGraph {
        &self.graph
    }

    fn graph_mut(&mut self) -> &mut MolGraph {
        &mut self.graph
    }
}

impl Map for MolMap0 {
    fn contains<E: Entity>(&self, entity: E) -> bool {
        self.contains(entity)
    }
}

impl AtomCore for MolMap0 {
    fn atom_graph(&self) -> &AtomGraph {
        self.graph.atom_graph()
    }

    fn atom_graph_mut(&mut self) -> &mut AtomGraph {
        self.graph.atom_graph_mut()
    }
}

impl AtomMap for MolMap0 {}

impl MolCore for MolMap0 {
    fn mol_graph(&self) -> &MolGraph {
        &self.graph
    }

    fn mol_graph_mut(&mut self) -> &mut MolGraph {
        &mut self.graph
    }
}

impl MolMap for MolMap0 {}

///// Methods for entity addition.
//impl MolMap0 {
//    /// Adds an atom to the map.
//    pub fn add_atom(&mut self, element: Element) -> Atom {
//        self.core.add_atom(element)
//    }
//
//    /// Adds an atom to the map along with the requested number of hydrogen atoms
//    /// (and single covalent bonds from the central atom to them).
//    ///
//    /// Returns the ID of the added central atom as well as a slice over the IDs
//    /// of the new bonds.
//    pub fn add_atom_with_hydrogen(&mut self, element: Element, n_hydrogen: u8) -> (Atom, &[Bond]) {
//        let centre = self.add_atom(element);
//        for _ in 0..n_hydrogen {
//            let new_h = self.add_atom(Element::H);
//            self.core
//                .add_bond(BondType::Covalent { order: 1.0 }, centre, new_h);
//        }
//        // Don't waste memory allocating a new Vec to hold the bond IDs, since
//        // they are already stored on the new central atom – return a slice instead
//        (
//            centre,
//            self.core()
//                .data(centre)
//                .expect("We just created this atom, so ID must be valid")
//                .bonds
//                .as_slice(),
//        )
//    }
//
//    /// Adds an atom to the map along with the number of additional hydrogen atoms
//    /// required to satisfy the most common valency for the central atom (and single
//    /// covalent bonds from the central atom to them).
//    ///
//    /// Returns the ID of the added central atom as well as a slice over the IDs
//    /// of the new bonds.
//    pub fn add_atom_and_saturate(&mut self, element: Element) -> (Atom, &[Bond]) {
//        let n_hydrogen = element.default_valency();
//        self.add_atom_with_hydrogen(element, n_hydrogen)
//    }
//
//    /// Adds a pseudoatom to the map.
//    pub fn add_pseudoatom(&mut self, pseudoelement: Pseudoelement) -> Pseudoatom {
//        self.core.add_pseudoatom(pseudoelement)
//    }
//
//    /// Creates a new bond between two bondable entities.
//    ///
//    /// # Errors
//    ///
//    /// Fails if either of `start` and `end` are invalid.
//    pub fn add_bond<A, B>(&mut self, bond_type: BondType, start: A, end: B) -> MolMapResult<Bond>
//    where
//        A: Bondable,
//        B: Bondable,
//    {
//        if !self.contains(start) {
//            return Err(MolMapError::InvalidId(start.as_entity()));
//        } else if !self.contains(end) {
//            return Err(MolMapError::InvalidId(end.as_entity()));
//        };
//        Ok(self.core.add_bond(bond_type, start, end))
//    }
//
//    /// Creates a new single covalent bond between two bondable entities.
//    ///
//    /// # Errors
//    ///
//    /// Fails if either of `start` and `end` are invalid.
//    pub fn add_single_bond<A, B>(&mut self, start: A, end: B) -> MolMapResult<Bond>
//    where
//        A: Bondable,
//        B: Bondable,
//    {
//        self.add_bond(BondType::Covalent { order: 1.0 }, start, end)
//    }
//
//    /// Creates a new double covalent bond between two bondable entities.
//    ///
//    /// # Errors
//    ///
//    /// Fails if either of `start` and `end` are invalid.
//    pub fn add_double_bond<A, B>(&mut self, start: A, end: B) -> MolMapResult<Bond>
//    where
//        A: Bondable,
//        B: Bondable,
//    {
//        self.add_bond(BondType::Covalent { order: 2.0 }, start, end)
//    }
//
//    /// Adds an empty substituent to the map.
//    pub fn add_substituent(&mut self) -> Substituent {
//        self.core.add_substituent()
//    }
//
//    /// Adds a substituent to the map with a single, newly-created central atom.
//    ///
//    /// Returns the IDs of the added substituent and central atom.
//    pub fn add_substituent_with_atom(&mut self, element: Element) -> (Substituent, Atom) {
//        let centre = self.add_atom(element);
//        let sub = self.core.add_substituent_with_centre(centre);
//        (sub, centre)
//    }
//
//    /// Adds a substituent to the map with a newly-created central atom and the
//    /// requested number of peripheral hydrogen atoms
//    /// (and single covalent bonds from the central atom to them).
//    ///
//    /// Returns the IDs of the added substituent, central atom, and a slice over
//    /// the IDs of the new bonds.
//    pub fn add_substituent_with_hydrogen(
//        &mut self,
//        element: Element,
//        n_hydrogen: u8,
//    ) -> (Substituent, Atom, &[Bond]) {
//        let (sub, centre) = self.add_substituent_with_atom(element);
//        for _ in 0..n_hydrogen {
//            let new_h = self.add_atom(Element::H);
//            let new_bond = self
//                .core
//                .add_bond(BondType::Covalent { order: 1.0 }, centre, new_h);
//            self.core.insert_into_substituent_unchecked(sub, new_h);
//            self.core.insert_into_substituent_unchecked(sub, new_bond);
//        }
//        (
//            sub,
//            centre,
//            self.core()
//                .data(centre)
//                .expect("We just created this atom, so ID must be valid")
//                .bonds
//                .as_slice(),
//        )
//    }
//
//    /// Adds a substituent to the map with a newly-created central atom and the
//    /// number of additional hydrogen atoms required to satisfy the most common
//    /// valency for the central atom (and single covalent bonds from the central
//    /// atom to them).
//    ///
//    /// Returns the ID of the added substituent, central atom, and a slice over
//    /// the IDs of the new bonds.
//    pub fn add_substituent_and_saturate(
//        &mut self,
//        element: Element,
//    ) -> (Substituent, Atom, &[Bond]) {
//        let n_hydrogen = element.default_valency();
//        self.add_substituent_with_hydrogen(element, n_hydrogen)
//    }
//
//    /// Adds an empty molecule to the map.
//    pub fn add_molecule(&mut self) -> Molecule {
//        self.core.add_molecule()
//    }
//}
//
//// Implement public API for deleting entities/changing their collection membership,
//// via mutable views
//
//impl<'m> ViewMut<'m, MolMap0, Atom> {
//    /// Removes the atom from the map, as well as any bonds to it.
//    pub fn delete(self) {
//        self.map.core_mut().delete_atom(self.id);
//    }
//}
//
//impl<'m> ViewMut<'m, MolMap0, Pseudoatom> {
//    /// Removes the pseudoatom from the map, as well as any bonds to it.
//    pub fn delete(self) {
//        self.map.core_mut().delete_pseudoatom(self.id);
//    }
//}
//
//impl<'m> ViewMut<'m, MolMap0, Bond> {
//    /// Removes the bond from the map (but not its bonding partners).
//    pub fn delete(self) {
//        self.map.core_mut().delete_bond(self.id);
//    }
//}
//
//impl<'m> ViewMut<'m, MolMap0, Substituent> {
//    /// Removes the substituent from the map, as well as all of its members.
//    pub fn delete(self) {
//        self.map.core_mut().delete_substituent(self.id);
//    }
//
//    /// Adds an atom, pseudoatom, or bond to the substituent.
//    ///
//    /// Returns whether the fundamental was newly inserted.
//    ///
//    /// If the fundamental is already a member of another substituent, it is removed
//    /// from it before it is inserted into this one.
//    ///
//    /// # Errors
//    ///
//    /// Returns an error if the fundamental is invalid.
//    pub fn insert(self, fundamental: impl Fundamental) -> MolMapResult<bool> {
//        if !self.map.contains(fundamental) {
//            return Err(MolMapError::InvalidId(fundamental.as_entity()));
//        };
//        Ok(self
//            .map
//            .core_mut()
//            .insert_into_substituent(self.id, fundamental))
//    }
//
//    /// Adds atoms, pseudoatoms, or bonds from an iterator to the substituent.
//    ///
//    /// If any of the fundamentals are already members of other substituents, they are
//    /// removed before they are inserted into this one.
//    ///
//    /// # Errors
//    ///
//    /// Returns an error if any of the fundamentals are invalid, in which case the
//    /// map will remain unchanged; the returned error will hold the ID of the first
//    /// invalid fundamental encountered.
//    pub fn extend<E, I>(self, fundamentals: I) -> MolMapResult<()>
//    where
//        E: Entity + Into<AnyFundamental>,
//        I: IntoIterator<Item = E>,
//        I::IntoIter: Clone,
//    {
//        let fundamentals = fundamentals.into_iter();
//        for f in fundamentals.clone() {
//            if !self.map.contains(f) {
//                return Err(MolMapError::InvalidId(f.as_entity()));
//            }
//        }
//        for f in fundamentals {
//            self.map
//                .core_mut()
//                .insert_into_substituent(self.id, f.into());
//        }
//        Ok(())
//    }
//
//    /// Removes an atom, pseudoatom, or bond from the substituent.
//    ///
//    /// Returns whether the fundamental was a member of the substituent.
//    ///
//    /// If the fundamental is an atomlike and is the centre of the substituent,
//    /// the centre is adjusted accordingly; if it is the lone centre, the
//    /// substituent becomes centreless. If it is one of two centres,
//    /// however, the centre remains `SubstituentCentre::Multiple` rather than
//    /// becoming `Single`.
//    ///
//    /// The substituent continues to exist, even if empty, as does the removed
//    /// fundamental.
//    pub fn remove(self, fundamental: impl Fundamental) -> bool {
//        self.map
//            .core_mut()
//            .remove_from_substituent(self.id, fundamental)
//    }
//
//    /// Empties the substituent by removing all its members, returning an iterator
//    /// over the IDs of the former members.
//    ///
//    /// The substituent and all removed fundamentals continue to exist.
//    ///
//    /// After this operation, the substituent will be centreless.
//    pub fn drain(self) -> impl Iterator<Item = AnyFundamental> {
//        self.map.core_mut().drain_substituent(self.id)
//    }
//
//    /// Empties the substituent by deleting all its members.
//    ///
//    /// The substituent itself continues to exist, and will be centreless.
//    pub fn clear(self) {
//        self.map.core_mut().clear_substituent(self.id);
//    }
//
//    /// Empties the substituent and then removes it from the map, returning the IDs of
//    /// the former members.
//    ///
//    /// All removed fundamentals continue to exist.
//    pub fn dissolve(self) -> impl Iterator<Item = AnyFundamental> {
//        self.map.core_mut().dissolve_substituent(self.id)
//    }
//}
//
//impl<'m> ViewMut<'m, MolMap0, Molecule> {
//    /// Removes the molecule from the map, as well as all of its members.
//    pub fn delete(self) {
//        self.map.core_mut().delete_molecule(self.id);
//    }
//
//    /// Adds an atom, pseudoatom, or bond to the molecule.
//    ///
//    /// Returns whether the fundamental was newly inserted.
//    ///
//    /// If the fundamental is already a member of another molecule, it is removed
//    /// from it before it is inserted into this one.
//    ///
//    /// # Errors
//    ///
//    /// Returns an error if the fundamental is invalid.
//    pub fn insert(self, fundamental: impl Fundamental) -> MolMapResult<bool> {
//        if !self.map.contains(fundamental) {
//            return Err(MolMapError::InvalidId(fundamental.as_entity()));
//        };
//        Ok(self
//            .map
//            .core_mut()
//            .insert_into_molecule(self.id, fundamental))
//    }
//
//    /// Adds atoms, pseudoatoms, or bonds from an iterator to the molecule.
//    ///
//    /// If any of the fundamentals are already members of other molecules, they are
//    /// removed before they are inserted into this one.
//    ///
//    /// # Errors
//    ///
//    /// Returns an error if any of the fundamentals are invalid, in which case the
//    /// map will remain unchanged; the returned error will hold the ID of the first
//    /// invalid fundamental encountered.
//    pub fn extend<E, I>(self, fundamentals: I) -> MolMapResult<()>
//    where
//        E: Entity + Into<AnyFundamental>,
//        I: IntoIterator<Item = E>,
//        I::IntoIter: Clone,
//    {
//        let fundamentals = fundamentals.into_iter();
//        for f in fundamentals.clone() {
//            if !self.map.contains(f) {
//                return Err(MolMapError::InvalidId(f.as_entity()));
//            }
//        }
//        for f in fundamentals {
//            self.map.core_mut().insert_into_molecule(self.id, f.into());
//        }
//        Ok(())
//    }
//
//    /// Removes an atom, pseudoatom, or bond from the molecule.
//    ///
//    /// Returns whether the fundamental was a member of the molecule.
//    ///
//    /// The molecule continues to exist even if empty, as does the removed
//    /// fundamental.
//    pub fn remove(self, fundamental: impl Fundamental) -> bool {
//        self.map
//            .core_mut()
//            .remove_from_molecule(self.id, fundamental)
//    }
//
//    /// Empties the molecule by removing all its members, returning an iterator
//    /// over the IDs of the former members.
//    ///
//    /// The molecule and all removed fundamentals continue to exist.
//    pub fn drain(self) -> impl Iterator<Item = AnyFundamental> {
//        self.map.core_mut().drain_molecule(self.id)
//    }
//
//    /// Empties the molecule by deleting all its members.
//    ///
//    /// The molecule itself continues to exist.
//    pub fn clear(self) {
//        self.map.core_mut().clear_molecule(self.id);
//    }
//
//    /// Empties the molecule and then removes it from the map, returning the IDs of the
//    /// former members.
//    ///
//    /// All removed fundamentals continue to exist.
//    pub fn dissolve(self) -> impl Iterator<Item = AnyFundamental> {
//        self.map.core_mut().dissolve_molecule(self.id)
//    }
//}
