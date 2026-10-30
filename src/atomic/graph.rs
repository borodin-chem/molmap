// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Definition of the data structure that holds the core graph of atoms and bonds.

use slotmap::SlotMap;

use crate::{
    Element, Map, MolMapError, MolMapResult, Pseudoelement, ViewMut,
    entities::*,
    traits::{Graph, Stores, StoresMut},
};

use super::data::*;

/// A map holding fundamental atomic entities, and a network of bonds between them.
///
/// This is the basic graph at the core of all other [`Map`] types.
///
/// The following invariants are promised to hold:
///
/// 1. Bonds only exist as long as both bonding partners continue to exist.
/// 2. The two bonding partners of a bond are always different.
/// 3. Only one `Bond` entity can connect a given pair of bonding partners
///    (though that bond may be a double bond/triple bond/etc.).
#[derive(Clone, Debug, Default)]
pub struct AtomMap {
    atoms: SlotMap<AtomKey, AtomData>,
    pseudoatoms: SlotMap<PseudoatomKey, PseudoatomData>,
    bonds: SlotMap<BondKey, BondData>,
}

macro_rules! impl_stores {
    ($kind: ident, $attr: ident) => {
        impl Stores<$kind> for AtomMap {
            fn slotmap(&self) -> &SlotMap<<$kind as Keyed>::Key, <$kind as Stored>::Data> {
                &self.$attr
            }
        }

        impl StoresMut<$kind> for AtomMap {
            fn slotmap_mut(
                &mut self,
            ) -> &mut SlotMap<<$kind as Keyed>::Key, <$kind as Stored>::Data> {
                &mut self.$attr
            }
        }
    };
}
impl_stores!(Atom, atoms);
impl_stores!(Bond, bonds);
impl_stores!(Pseudoatom, pseudoatoms);

impl Graph for AtomMap {}

impl Map for AtomMap {
    type Graph = Self;

    fn into_graph(self) -> Self::Graph {
        self
    }

    fn as_graph(&self) -> &Self {
        self
    }

    fn into_atomic(self) -> AtomMap {
        self
    }

    fn as_atomic(&self) -> &AtomMap {
        self
    }

    fn contains<E: Entity>(&self, entity: E) -> bool {
        match entity.to_resolved() {
            ResolvedEntity::Atom(atom) => {
                <AtomMap as Stores<Atom>>::slotmap(self).contains_key(atom.to_key())
            }
            ResolvedEntity::Bond(bond) => {
                <AtomMap as Stores<Bond>>::slotmap(self).contains_key(bond.to_key())
            }
            ResolvedEntity::Pseudoatom(pseudoatom) => {
                <AtomMap as Stores<Pseudoatom>>::slotmap(self).contains_key(pseudoatom.to_key())
            }
            _ => false,
        }
    }
}

/// Constructor methods.
impl AtomMap {
    /// Creates a new, empty `AtomGraph`.
    pub fn new() -> Self {
        Self {
            atoms: SlotMap::with_key(),
            pseudoatoms: SlotMap::with_key(),
            bonds: SlotMap::with_key(),
        }
    }

    /// Creates a new `AtomGraph` with the specified capacities for each kind of entity.
    pub fn with_capacities(atoms: usize, pseudoatoms: usize, bonds: usize) -> Self {
        Self {
            atoms: SlotMap::with_capacity_and_key(atoms),
            pseudoatoms: SlotMap::with_capacity_and_key(pseudoatoms),
            bonds: SlotMap::with_capacity_and_key(bonds),
        }
    }
}

/// Methods for entity addition.
impl AtomMap {
    /// Adds an atom to the map.
    pub fn add_atom(&mut self, element: Element) -> Atom {
        self.atoms.insert(AtomData::new(element)).into()
    }

    /// Adds an atom to the map along with the requested number of hydrogen atoms
    /// (and single covalent bonds from the central atom to them).
    ///
    /// Returns the ID of the added central atom as well as a slice over the IDs
    /// of the new bonds.
    pub fn add_atom_with_hydrogen(&mut self, element: Element, n_hydrogen: u8) -> (Atom, &[Bond]) {
        let centre = self.add_atom(element);
        for _ in 0..n_hydrogen {
            let new_h = self.add_atom(Element::H);
            self.add_bond_unchecked(BondType::Covalent { order: 1.0 }, centre, new_h);
        }
        // Don't waste memory allocating a new Vec to hold the bond IDs, since
        // they are already stored on the new central atom – return a slice instead
        (
            centre,
            self.data(centre)
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
    pub fn add_atom_and_saturate(&mut self, element: Element) -> (Atom, &[Bond]) {
        let n_hydrogen = element.default_valency();
        self.add_atom_with_hydrogen(element, n_hydrogen)
    }

    /// Adds a pseudoatom to the map.
    pub fn add_pseudoatom(&mut self, pseudoelement: Pseudoelement) -> Pseudoatom {
        self.pseudoatoms
            .insert(PseudoatomData::new(pseudoelement))
            .into()
    }

    /// Creates a new bond between two bondable entities without validating them first.
    ///
    /// # Panics
    ///
    /// Panics if either of `start` and `end` are invalid.
    pub fn add_bond_unchecked<A, B>(&mut self, bond_type: BondType, start: A, end: B) -> Bond
    where
        A: Bondable,
        B: Bondable,
    {
        let bond: Bond = self
            .bonds
            .insert(BondData::new(
                bond_type,
                start.as_bondable(),
                end.as_bondable(),
            ))
            .into();
        for partner in [start.as_bondable(), end.as_bondable()] {
            match partner.resolve() {
                ResolvedBondable::Atom(id) => {
                    self.atoms.get_mut(id.into()).unwrap().bonds.push(bond)
                }
                ResolvedBondable::Pseudoatom(id) => self
                    .pseudoatoms
                    .get_mut(id.into())
                    .unwrap()
                    .bonds
                    .push(bond),
            }
        }
        bond
    }

    /// Creates a new bond between two bondable entities.
    ///
    /// # Errors
    ///
    /// Fails if either of `start` and `end` are invalid.
    pub fn add_bond<A, B>(&mut self, bond_type: BondType, start: A, end: B) -> MolMapResult<Bond>
    where
        A: Bondable,
        B: Bondable,
    {
        if !self.contains(start) {
            return Err(MolMapError::InvalidId(start.as_entity()));
        } else if !self.contains(end) {
            return Err(MolMapError::InvalidId(end.as_entity()));
        };
        Ok(self.add_bond_unchecked(bond_type, start, end))
    }

    /// Creates a new single covalent bond between two bondable entities.
    ///
    /// # Errors
    ///
    /// Fails if either of `start` and `end` are invalid.
    pub fn add_single_bond<A, B>(&mut self, start: A, end: B) -> MolMapResult<Bond>
    where
        A: Bondable,
        B: Bondable,
    {
        self.add_bond(BondType::Covalent { order: 1.0 }, start, end)
    }

    /// Creates a new double covalent bond between two bondable entities.
    ///
    /// # Errors
    ///
    /// Fails if either of `start` and `end` are invalid.
    pub fn add_double_bond<A, B>(&mut self, start: A, end: B) -> MolMapResult<Bond>
    where
        A: Bondable,
        B: Bondable,
    {
        self.add_bond(BondType::Covalent { order: 2.0 }, start, end)
    }
}

/// Methods for entity removal.
impl AtomMap {
    /// Removes an atom from the map, as well as any bonds to it.
    ///
    /// Returns whether the atom was present in the map.
    ///
    /// This is infallible – if the atom is not in the map, nothing changes.
    pub(crate) fn delete_atom(&mut self, atom: Atom) -> bool {
        if !self.contains(atom) {
            return false;
        }
        // Make sure we always remove bonds first
        let bonds = self.data(atom).unwrap().bonds.clone();
        for bond in bonds {
            self.delete_bond(bond);
        }
        // Now we can safely remove the atom itself without leaving dangling bonds
        self.atoms.remove(atom.into()).is_some() // Should always be `true`
    }

    /// Removes a pseudoatom from the map, as well as any bonds to it.
    ///
    /// Returns whether the pseudoatom was present in the map.
    ///
    /// This is infallible – if the pseudoatom is not in the map, nothing changes.
    pub(crate) fn delete_pseudoatom(&mut self, pseudoatom: Pseudoatom) -> bool {
        if !self.contains(pseudoatom) {
            return false;
        }
        // Make sure we always remove bonds first
        let bonds = self.data(pseudoatom).unwrap().bonds.clone();
        for bond in bonds {
            self.delete_bond(bond);
        }
        // Now we can safely remove the pseudoatom itself without leaving dangling bonds
        self.pseudoatoms.remove(pseudoatom.into()).is_some()
    }

    /// Removes a bond from the map (but not its bonding partners).
    ///
    /// Returns whether the bond was present in the map.
    ///
    /// If the bond is not in the map, nothing changes.
    ///
    /// # Panics
    ///
    /// Panics if either of the bond's bonding partners does not exist (which
    /// should never be the case – bonds are last in, first out).
    pub(crate) fn delete_bond(&mut self, bond: Bond) -> bool {
        if let Some(bond_data) = self.bonds.remove(bond.into()) {
            for bonding_partner in [bond_data.start, bond_data.end] {
                match bonding_partner.resolve() {
                    ResolvedBondable::Atom(atom) => {
                        let atom_data = self
                            .data_mut(atom)
                            .expect("Bonds are always removed before their bonding partners");
                        let pos = atom_data.bonds.iter().position(|x| *x == bond).expect(
                            "Bond should be listed in the bonding partner's bonds until deletion",
                        );
                        atom_data.bonds.remove(pos);
                    }
                    ResolvedBondable::Pseudoatom(pseudoatom) => {
                        let pseudoatom_data = self
                            .data_mut(pseudoatom)
                            .expect("Bonds are always removed before their bonding partners");
                        let pos = pseudoatom_data.bonds.iter().position(|x| *x == bond).expect(
                                "Bond should be listed in the bonding partner's bonds until deletion",
                            );
                        pseudoatom_data.bonds.remove(pos);
                    }
                }
            }
            true
        } else {
            false
        }
    }
}

/// View methods specific to `AtomMap`.
mod views {
    use super::*;

    impl<'m> ViewMut<'m, AtomMap, Atom> {
        /// Removes the atom from the map, as well as any bonds to it.
        pub fn delete(self) {
            self.map.delete_atom(self.id);
        }
    }

    impl<'m> ViewMut<'m, AtomMap, Pseudoatom> {
        /// Removes the pseudoatom from the map, as well as any bonds to it.
        pub fn delete(self) {
            self.map.delete_pseudoatom(self.id);
        }
    }

    impl<'m> ViewMut<'m, AtomMap, Bond> {
        /// Removes the bond from the map (but not its bonding partners).
        pub fn delete(self) {
            self.map.delete_bond(self.id);
        }
    }
}

#[cfg(test)]
#[allow(unused)]
pub(crate) mod tests {
    use super::*;

    /// Creates a basic graph to use as the basis for various tests.
    pub(crate) fn meoh_graph() -> AtomMap {
        let mut g = AtomMap::new();
        let h1 = g.add_atom(Element::H);
        let h2 = g.add_atom(Element::H);
        let h3 = g.add_atom(Element::H);
        let c1 = g.add_atom(Element::C);
        let c1h1 = g.add_bond_unchecked(BondType::Covalent { order: 1.0 }, c1, h1);
        let c1h2 = g.add_bond_unchecked(BondType::Covalent { order: 1.0 }, c1, h2);
        let c1h3 = g.add_bond_unchecked(BondType::Covalent { order: 1.0 }, c1, h3);
        let o1 = g.add_atom(Element::O);
        let h4 = g.add_atom(Element::H);
        let o1h4 = g.add_bond_unchecked(BondType::Covalent { order: 1.0 }, o1, h4);
        let c1o1 = g.add_bond_unchecked(BondType::Covalent { order: 1.0 }, c1, o1);
        g
    }

    #[test]
    fn slotmap() {
        let g = AtomMap::new();
        // Make sure that the generic slotmap method returns a reference to the appropriate slotmap
        assert!(std::ptr::eq(
            <AtomMap as Stores<Atom>>::slotmap(&g),
            &g.atoms
        ));
        assert!(std::ptr::eq(
            <AtomMap as Stores<Bond>>::slotmap(&g),
            &g.bonds
        ));
        assert!(std::ptr::eq(
            <AtomMap as Stores<Pseudoatom>>::slotmap(&g),
            &g.pseudoatoms
        ));
    }

    #[test]
    fn data() {
        let mut g = AtomMap::new();
        let h1 = g.add_atom(Element::H);
        let ph = g.add_pseudoatom(Pseudoelement::Ph);
        // Make sure that the generic data method returns a reference to the appropriate entity data struct
        let h1_data = g.data(h1).unwrap();
        assert!(std::ptr::eq(h1_data, g.atoms.get(h1.into()).unwrap()));
        assert_eq!(h1_data.element, Element::H);
        assert_eq!(h1_data.bonds, &[]);
        assert!(std::ptr::eq(
            g.data(ph).unwrap(),
            g.pseudoatoms.get(ph.into()).unwrap()
        ));
    }

    #[test]
    fn contains() {
        let mut g = AtomMap::new();
        let h1 = g.add_atom(Element::H);
        assert!(g.contains(h1));
        let ph = g.add_pseudoatom(Pseudoelement::Ph);
        assert!(g.contains(ph));
        g.delete_atom(h1);
        assert!(!g.contains(h1));
        assert!(g.contains(ph));
        g.delete_pseudoatom(ph);
        assert!(!g.contains(ph));
    }

    #[test]
    fn add_atom() {
        let mut g = AtomMap::new();
        assert!(g.atoms.is_empty());
        let h1 = g.add_atom(Element::H);
        assert_eq!(g.atoms.len(), 1);
        let c1 = g.add_atom(Element::C);
        assert_eq!(g.atoms.len(), 2);
        // Check the atoms can be accessed by their ID, and that the elements are correct
        assert_eq!(g.atoms.get(h1.into()).unwrap().element, Element::H);
        assert_eq!(g.atoms.get(c1.into()).unwrap().element, Element::C);
        // Check that the bond arrays are created empty
        assert!(g.atoms.get(h1.into()).unwrap().bonds.is_empty());
    }

    #[test]
    fn add_pseudoatom() {
        let mut g = AtomMap::new();
        assert!(g.pseudoatoms.is_empty());
        let ph = g.add_pseudoatom(Pseudoelement::Ph);
        assert_eq!(g.pseudoatoms.len(), 1);
    }

    #[test]
    fn delete_atom() {
        let mut g = AtomMap::new();
        let h1 = g.add_atom(Element::H);
        let c1 = g.add_atom(Element::C);
        assert_eq!(g.atoms.len(), 2);
        assert!(g.delete_atom(h1));
        assert_eq!(g.atoms.len(), 1);
        // Make sure the correct one got deleted
        assert_eq!(g.atoms.keys().next().unwrap(), c1.to_key());
        // Trying to delete the same atom again shouldn't panic but should return false
        assert!(!g.delete_atom(h1));
    }

    #[test]
    fn delete_pseudoatom() {
        let mut g = AtomMap::new();
        let et = g.add_pseudoatom(Pseudoelement::Et);
        assert_eq!(g.pseudoatoms.len(), 1);
        g.delete_pseudoatom(et);
        assert!(g.pseudoatoms.is_empty());
    }

    #[test]
    fn add_bond_between_atoms() {
        let mut g = AtomMap::new();
        assert!(g.bonds.is_empty());
        let h1 = g.add_atom(Element::H);
        let h2 = g.add_atom(Element::H);
        let b1 = g.add_bond_unchecked(BondType::Covalent { order: 1.0 }, h1, h2);
        assert_eq!(g.bonds.len(), 1);
        assert!(g.contains(b1));
        assert!(g.atoms.get(h1.into()).unwrap().bonds.contains(&b1));
        assert!(g.atoms.get(h2.into()).unwrap().bonds.contains(&b1));
        assert_eq!(g.bonds.get(b1.into()).unwrap().start, h1.into());
        assert_eq!(g.bonds.get(b1.into()).unwrap().end, h2.into());
    }

    #[test]
    fn delete_bond_between_atoms() {
        let mut g = AtomMap::new();
        let h1 = g.add_atom(Element::H);
        let h2 = g.add_atom(Element::H);
        let b1 = g.add_bond_unchecked(BondType::Covalent { order: 1.0 }, h1, h2);
        for h in [h1, h2] {
            assert!(g.atoms.get(h.into()).unwrap().bonds.contains(&b1));
        }
        assert_eq!(g.bonds.get(b1.into()).unwrap().start, h1.as_bondable());
        assert_eq!(g.bonds.get(b1.into()).unwrap().end, h2.as_bondable());
        // Now delete the bond and check the effects
        g.delete_bond(b1);
        // Bond should obviously be gone
        assert!(!g.contains(b1));
        // Atoms should remain, however
        for h in [h1, h2] {
            assert!(g.contains(h));
            // Neither atom should have any bonds now
            assert!(g.atoms.get(h.into()).unwrap().bonds.is_empty());
        }
    }

    #[test]
    fn deleting_bonding_partner_deletes_bond() {
        let mut g = AtomMap::new();
        let h1 = g.add_atom(Element::H);
        let h2 = g.add_atom(Element::H);
        let b1 = g.add_bond_unchecked(BondType::Covalent { order: 1.0 }, h1, h2);
        // Delete one of the bonding atoms and check the effects
        g.delete_atom(h1);
        // Atom should obviously be gone
        assert!(!g.contains(h1));
        // Bond should have been removed too
        assert!(!g.contains(b1));
        // The other atom should still remain, but without any bonds now
        assert!(g.contains(h2));
        assert!(g.atoms.get(h2.into()).unwrap().bonds.is_empty());
    }
}
