// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::iter::FusedIterator;

use slotmap::{SlotMap, basic::Keys};

use crate::{
    Element, Pseudoelement,
    categories::*,
    entities::*,
    graph::{entities::*, keys::*},
    id::Id,
};

/// An arena-like data structure to represent a set of chemical entities,
/// their properties, and the relationships between them, as a molecular graph.
///
/// A `MolGraph` forms the core of all `MolMap` types, but the type is not meant
/// for external use. Its `pub` visibility is necessary to match [`MolMapCore`].
///
/// [`MolMap0`] is the `MolMap` type that provides a molecular graph for users.
///
/// In general, the methods of `MolGraph` should be small in scope and efficient
/// so that the higher maps can combine them to create a nice public API.
/// The methods should also do as little checking and validation as possible, with
/// panicking preferred - the higher maps are then responsible for careful usage.
#[derive(Clone, Debug, Default)]
pub struct MolGraph {
    pub(super) atoms: SlotMap<AtomKey, AtomData>,
    pub(super) pseudoatoms: SlotMap<PseudoatomKey, PseudoatomData>,
    pub(super) bonds: SlotMap<BondKey, BondData>,
    pub(super) substituents: SlotMap<SubstituentKey, SubstituentData>,
    pub(super) molecules: SlotMap<MoleculeKey, MoleculeData>,
}

/// Constructor methods.
impl MolGraph {
    /// Creates a new, empty `MolGraph`.
    pub(crate) fn new() -> Self {
        Self {
            atoms: SlotMap::with_key(),
            pseudoatoms: SlotMap::with_key(),
            bonds: SlotMap::with_key(),
            substituents: SlotMap::with_key(),
            molecules: SlotMap::with_key(),
        }
    }

    /// Creates a new `MolGraph` with the specified capacities for each kind of entity.
    pub(crate) fn with_capacities(
        atoms: usize,
        pseudoatoms: usize,
        bonds: usize,
        substituents: usize,
        molecules: usize,
    ) -> Self {
        Self {
            atoms: SlotMap::with_capacity_and_key(atoms),
            pseudoatoms: SlotMap::with_capacity_and_key(pseudoatoms),
            bonds: SlotMap::with_capacity_and_key(bonds),
            substituents: SlotMap::with_capacity_and_key(substituents),
            molecules: SlotMap::with_capacity_and_key(molecules),
        }
    }
}

/// Methods generic over all stored kinds of entity, that do the same regardless of kind.
impl MolGraph {
    /// Returns a reference to the `SlotMap` that holds the entity.
    #[inline]
    pub(crate) fn slotmap<E: Entity + Keyed>(&self) -> &SlotMap<E::KEY, E::DATA> {
        E::get_slotmap(self)
    }

    /// Returns a mutable reference to the `SlotMap` that holds the entity.
    #[inline]
    pub(crate) fn slotmap_mut<E: Entity + Keyed>(&mut self) -> &mut SlotMap<E::KEY, E::DATA> {
        E::get_slotmap_mut(self)
    }

    /// Returns a reference to the entity's data struct, or `None` if `entity` is invalid.
    #[inline]
    pub(crate) fn get_data<E: Entity + Keyed>(&self, entity: E) -> Option<&E::DATA> {
        self.slotmap::<E>().get(entity.to_key())
    }

    /// Returns a mutable reference to the entity's data struct, or `None` if `entity` is invalid.
    #[inline]
    pub(crate) fn get_data_mut<E: Entity + Keyed>(&mut self, entity: E) -> Option<&mut E::DATA> {
        self.slotmap_mut::<E>().get_mut(entity.to_key())
    }

    /// Returns a reference to the entity's data struct.
    ///
    /// # Panics
    ///
    /// Panics if `entity` is invalid.
    #[inline]
    pub(crate) fn data<E: Entity + Keyed>(&self, entity: E) -> &E::DATA {
        self.get_data(entity).unwrap()
    }

    /// Returns a mutable reference to the entity's data struct.
    ///
    /// # Panics
    ///
    /// Panics if `entity` is invalid.
    #[inline]
    pub(crate) fn data_mut<E: Entity + Keyed>(&mut self, entity: E) -> &mut E::DATA {
        self.get_data_mut(entity).unwrap()
    }

    /// Checks if the map currently contains the given entity.
    pub(crate) fn contains<E: Entity + Keyed>(&self, entity: E) -> bool {
        self.slotmap::<E>().contains_key(entity.to_key())
    }

    /// Returns an iterator over all of a given kind of entity in the map.
    pub(crate) fn iter<E: Entity + Keyed>(
        &'_ self,
    ) -> impl Iterator<Item = E> + ExactSizeIterator + FusedIterator {
        self.slotmap::<E>().keys().map(|k| E::from_key(k))
    }
}

/// Methods for entity addition.
impl MolGraph {
    /// Adds an atom to the map.
    pub(crate) fn add_atom(&mut self, element: Element) -> Atom {
        self.atoms.insert(AtomData::new(element)).into()
    }

    /// Adds a pseudoatom to the map.
    pub(crate) fn add_pseudoatom(&mut self, pseudoelement: Pseudoelement) -> Pseudoatom {
        self.pseudoatoms
            .insert(PseudoatomData::new(pseudoelement))
            .into()
    }

    /// Creates a new (single covalent) bond between two bondable entities.
    ///
    /// # Panics
    ///
    /// Panics if either of `start` and `end` are invalid.
    pub(crate) fn add_bond(&mut self, start: impl Bondable, end: impl Bondable) -> Bond {
        let bond: Bond = self
            .bonds
            .insert(BondData::new(
                BondType::Covalent,
                1.0,
                start.as_bondable(),
                end.as_bondable(),
            ))
            .into();
        for partner in [start.as_bondable(), end.as_bondable()] {
            match partner.as_tagged_bondable() {
                TaggedBondable::Atom(id) => self.atoms.get_mut(id.into()).unwrap().bonds.push(bond),
                TaggedBondable::Pseudoatom(id) => self
                    .pseudoatoms
                    .get_mut(id.into())
                    .unwrap()
                    .bonds
                    .push(bond),
            }
        }
        bond
    }

    /// Adds an empty substituent to the map.
    ///
    /// If the atomlike that is going to be the substituent's centre already
    /// exists, prefer `add_substituent_with_centre`.
    pub(crate) fn add_substituent(&mut self) -> Substituent {
        self.substituents
            .insert(SubstituentData {
                centre: SubstituentCentre::None,
                members: Vec::new(),
            })
            .into()
    }

    /// Adds a substituent to the map with the given atomlike as its centre.
    ///
    /// Note that this method will not fail, even if `centre` is an invalid ID.
    pub(crate) fn add_substituent_with_centre(&mut self, centre: impl Atomlike) -> Substituent {
        self.substituents
            .insert(SubstituentData::new(
                centre.as_atomlike(),
                &[centre.as_atomlike().into()],
            ))
            .into()
    }

    /// Adds an empty molecule to the map.
    pub(crate) fn add_molecule(&mut self) -> Molecule {
        self.molecules.insert(MoleculeData::new()).into()
    }
}

/// Methods for entity removal.
impl MolGraph {
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
        let bonds = self.data(atom).bonds.clone();
        for bond in bonds {
            self.delete_bond(bond);
        }
        // Remove from any collections
        if let Some(sub) = self.parent_substituent(atom) {
            self.remove_from_substituent(sub, atom);
        }
        if let Some(mol) = self.parent_molecule(atom) {
            self.remove_from_molecule(mol, atom);
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
        let bonds = self.data(pseudoatom).bonds.clone();
        for bond in bonds {
            self.delete_bond(bond);
        }
        // Remove from any collections
        if let Some(sub) = self.parent_substituent(pseudoatom) {
            self.remove_from_substituent(sub, pseudoatom);
        }
        if let Some(mol) = self.parent_molecule(pseudoatom) {
            self.remove_from_molecule(mol, pseudoatom);
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
                match bonding_partner.as_tagged_bondable() {
                    TaggedBondable::Atom(atom) => {
                        let mut atom_data = self
                            .get_data_mut(atom)
                            .expect("Bonds are always removed before their bonding partners");
                        let pos = atom_data.bonds.iter().position(|x| *x == bond).expect(
                            "Bond should be listed in the bonding partner's bonds until deletion",
                        );
                        atom_data.bonds.remove(pos);
                    }
                    TaggedBondable::Pseudoatom(pseudoatom) => {
                        let mut pseudoatom_data = self
                            .get_data_mut(pseudoatom)
                            .expect("Bonds are always removed before their bonding partners");
                        let pos = pseudoatom_data.bonds.iter().position(|x| *x == bond).expect(
                            "Bond should be listed in the bonding partner's bonds until deletion",
                        );
                        pseudoatom_data.bonds.remove(pos);
                    }
                }
            }
            // Remove from any collections
            if let Some(sub) = self.parent_substituent(bond) {
                self.remove_from_substituent(sub, bond);
            }
            if let Some(mol) = self.parent_molecule(bond) {
                self.remove_from_molecule(mol, bond);
            }
            true
        } else {
            false
        }
    }

    /// Removes a substituent from the map, as well as all of its members.
    ///
    /// Returns whether the substituent was present in the map.
    ///
    /// This is infallible – if the substituent is not in the map, nothing changes.
    pub(crate) fn delete_substituent(&mut self, substituent: Substituent) -> bool {
        if !self.contains(substituent) {
            return false;
        };
        let members = self.data(substituent).members.clone();
        for member in members {
            match member.as_tagged_fundamental() {
                TaggedFundamental::Atom(atom) => {
                    self.delete_atom(atom);
                }
                TaggedFundamental::Pseudoatom(pseudoatom) => {
                    self.delete_pseudoatom(pseudoatom);
                }
                TaggedFundamental::Bond(bond) => {
                    self.delete_bond(bond);
                }
            }
        }
        self.substituents.remove(substituent.into()).is_some()
    }

    /// Removes a molecule from the map, as well as all of its members.
    ///
    /// Returns whether the molecule was present in the map.
    ///
    /// This is infallible – if the molecule is not in the map, nothing changes.
    pub(crate) fn delete_molecule(&mut self, molecule: Molecule) -> bool {
        if !self.contains(molecule) {
            return false;
        };
        let members = self.data(molecule).members.clone();
        for member in members {
            match member.as_tagged_fundamental() {
                TaggedFundamental::Atom(id) => {
                    self.delete_atom(id);
                }
                TaggedFundamental::Pseudoatom(id) => {
                    self.delete_pseudoatom(id);
                }
                TaggedFundamental::Bond(id) => {
                    self.delete_bond(id);
                }
            }
        }
        self.molecules.remove(molecule.into()).is_some()
    }
}

/// Methods to change collection membership.
///
/// Some of these methods are identical for all current collection types, but are not
/// implemented using macros due to the likelihood that they will need to diverge in
/// future.
impl MolGraph {
    /// Adds an atom, pseudoatom, or bond to a substituent.
    ///
    /// Returns whether the fundamental was newly inserted.
    ///
    /// This method should only ever be used with fundamentals that do not already
    /// belong to a substituent.
    ///
    /// # Panics
    ///
    /// Panics if `substituent` is invalid, but is unaffected if `fundamental` is
    /// invalid.
    pub(crate) fn insert_into_substituent(
        &mut self,
        substituent: Substituent,
        fundamental: impl Fundamental,
    ) -> bool {
        let sub = self.substituents.get_mut(substituent.into()).unwrap();
        let fund = fundamental.as_fundamental();
        // members is just a Vec, so have to manually make sure we don't end up with
        // any ID in the substituent twice
        if !sub.members.contains(&fund) {
            sub.members.push(fund);
            true
        } else {
            false
        }
    }

    /// Adds an atom, pseudoatom, or bond to a molecule.
    ///
    /// Returns whether the fundamental was newly inserted.
    ///
    /// This method should only ever be used with fundamentals that do not already
    /// belong to a molecule.
    ///
    /// # Panics
    ///
    /// Panics if `molecule` is invalid, but is unaffected if `fundamental` is
    /// invalid.
    pub(crate) fn insert_into_molecule(
        &mut self,
        molecule: Molecule,
        fundamental: impl Fundamental,
    ) -> bool {
        let mol = self.molecules.get_mut(molecule.into()).unwrap();
        mol.members.insert(fundamental.as_fundamental())
    }

    /// Adds atoms, pseudoatoms, or bonds from an iterator to a substituent.
    ///
    /// # Panics
    ///
    /// Panics if `substituent` is invalid, but is unaffected if any of the fundamental
    /// IDs are invalid.
    pub(crate) fn extend_substituent<E, I>(&mut self, substituent: Substituent, fundamentals: I)
    where
        E: Fundamental,
        I: IntoIterator<Item = E>,
    {
        let sub = self.substituents.get_mut(substituent.into()).unwrap();
        // Can't just call extend, because we don't allow the IDs to be added twice
        for fundamental in fundamentals {
            let fund = fundamental.as_fundamental();
            if !sub.members.contains(&fund) {
                sub.members.push(fund);
            }
        }
    }

    /// Adds atoms, pseudoatoms, or bonds from an iterator to a molecule.
    ///
    /// # Panics
    ///
    /// Panics if `molecule` is invalid, but is unaffected if any of the fundamental
    /// IDs are invalid.
    pub(crate) fn extend_molecule<E, I>(&mut self, molecule: Molecule, fundamentals: I)
    where
        E: Fundamental,
        I: IntoIterator<Item = E>,
    {
        let mol = self.molecules.get_mut(molecule.into()).unwrap();
        mol.members
            .extend(fundamentals.into_iter().map(|e| e.as_fundamental()))
    }

    /// Removes an atom, pseudoatom, or bond from a substituent.
    ///
    /// Returns whether the fundamental was a member of the substituent.
    ///
    /// If the fundamental is an atomlike and is the centre of the substituent,
    /// the centre is adjusted accordingly; if it is the lone centre, the
    /// substituent becomes centreless. If it is one of two centres,
    /// however, the centre remains `SubstituentCentre::Multiple` rather than
    /// becoming `Single`.
    ///
    /// The substituent continues to exist, even if empty, as does the removed
    /// fundamental.
    ///
    /// # Panics
    ///
    /// Panics if `substituent` is invalid.
    pub(crate) fn remove_from_substituent(
        &mut self,
        substituent: Substituent,
        fundamental: impl Fundamental,
    ) -> bool {
        let sub = self.substituents.get_mut(substituent.into()).unwrap();
        let fund = fundamental.as_fundamental();
        if let Some(index) = sub.members.iter().position(|x| *x == fund) {
            sub.members.swap_remove(index);
        } else {
            return false;
        }
        // If fundamental is an atomlike, we might be removing the centre of the
        // substituent (or one of them).
        // Is so, adjust the centres of the substituent accordingly
        match &mut sub.centre {
            SubstituentCentre::None => (),
            SubstituentCentre::Single(atomlike) => {
                if atomlike.into_inner() == fundamental.into_inner() {
                    // No longer has a centre
                    sub.centre = SubstituentCentre::None
                }
            }
            SubstituentCentre::Multiple(atomlikes) => {
                if let Some(atomlike) = match fundamental.as_tagged_fundamental() {
                    TaggedFundamental::Bond(_) => None,
                    TaggedFundamental::Atom(atom) => Some(atom.into()),
                    TaggedFundamental::Pseudoatom(pseudoatom) => Some(pseudoatom.into()),
                } && let Some(index) = atomlikes.iter().position(|x| *x == atomlike)
                {
                    // We want to preserve order
                    atomlikes.remove(index);
                }
            }
        }
        true
    }

    /// Removes an atom, pseudoatom, or bond from a molecule.
    ///
    /// Returns whether the fundamental was a member of the molecule.
    ///
    /// The molecule continues to exist even if empty, as does the removed
    /// fundamental.
    ///
    /// # Panics
    ///
    /// Panics if `molecule` is invalid.
    pub(crate) fn remove_from_molecule(
        &mut self,
        molecule: Molecule,
        fundamental: impl Fundamental,
    ) -> bool {
        let mol = self.molecules.get_mut(molecule.into()).unwrap();
        mol.members.remove(&fundamental.as_fundamental())
    }

    /// Empties a substituent by removing all its members, returning an iterator over
    /// the IDs of the former members.
    ///
    /// The substituent and all removed fundamentals continue to exist.
    ///
    /// After this operation, the substituent will be centreless.
    ///
    /// # Panics
    ///
    /// Panics if the substituent is not in the map.
    pub(crate) fn drain_substituent(
        &mut self,
        substituent: Substituent,
    ) -> impl Iterator<Item = impl Fundamental> {
        let sub = self
            .substituents
            .get_mut(substituent.into())
            .expect("Caller is required to ensure that the substituent is valid");
        sub.centre = SubstituentCentre::None;
        sub.members.drain(..)
    }

    /// Empties a molecule by removing all its members, returning an iterator over
    /// the IDs of the former members.
    ///
    /// The molecule and all removed fundamentals continue to exist.
    ///
    /// # Panics
    ///
    /// Panics if the molecule is not in the map.
    pub(crate) fn drain_molecule(
        &mut self,
        molecule: Molecule,
    ) -> impl Iterator<Item = impl Fundamental> {
        let mut mol = self
            .molecules
            .get_mut(molecule.into())
            .expect("Caller is required to ensure that the molecule is valid");
        mol.members.drain()
    }

    /// Empties a substituent by deleting all its members.
    ///
    /// The substituent itself continues to exist, and will be centreless.
    ///
    /// # Panics
    ///
    /// Panics if the substituent is not in the map.
    pub(crate) fn clear_substituent(&mut self, substituent: Substituent) {
        let sub = self
            .substituents
            .get_mut(substituent.into())
            .expect("Caller is required to ensure that the substituent is valid");
        sub.centre = SubstituentCentre::None;
        let former_members: Vec<AnyFundamental> = sub.members.drain(..).collect();
        // It's fine to delete in any order as if something isn't in the map any more
        // (e.g. because it's a bond and one of its bonding partners was already deleted
        // and thus it too was already deleted) then nothing changes when the deletion
        // is attempted
        for member in former_members {
            match member.as_tagged_fundamental() {
                TaggedFundamental::Atom(atom) => self.delete_atom(atom),
                TaggedFundamental::Pseudoatom(pseudoatom) => self.delete_pseudoatom(pseudoatom),
                TaggedFundamental::Bond(bond) => self.delete_bond(bond),
            };
        }
    }

    /// Empties a molecule by deleting all its members.
    ///
    /// The molecule itself continues to exist.
    ///
    /// # Panics
    ///
    /// Panics if the molecule is not in the map.
    pub(crate) fn clear_molecule(&mut self, molecule: Molecule) {
        let mut mol = self
            .molecules
            .get_mut(molecule.into())
            .expect("Caller is required to ensure that the Molecule is valid");
        let former_members: Vec<AnyFundamental> = mol.members.drain().collect();
        // It's fine to delete in any order as if something isn't in the map any more
        // (e.g. because it's a bond and one of its bonding partners was already deleted
        // and thus it too was already deleted) then nothing changes when the deletion
        // is attempted
        for member in former_members {
            match member.as_tagged_fundamental() {
                TaggedFundamental::Atom(atom) => self.delete_atom(atom),
                TaggedFundamental::Pseudoatom(pseudoatom) => self.delete_pseudoatom(pseudoatom),
                TaggedFundamental::Bond(bond) => self.delete_bond(bond),
            };
        }
    }

    /// Empties a substituent and then removes it from the map, returning the IDs of the
    /// former members.
    ///
    /// All removed fundamentals continue to exist.
    ///
    /// # Panics
    ///
    /// Panics if the substituent is not in the map.
    pub(crate) fn dissolve_substituent(
        &mut self,
        substituent: Substituent,
    ) -> impl Iterator<Item = impl Fundamental> {
        let sub = self
            .substituents
            .remove(substituent.into())
            .expect("Caller is required to ensure that the Molecule is valid");
        sub.members.into_iter()
    }

    /// Empties a molecule and then removes it from the map, returning the IDs of the
    /// former members.
    ///
    /// All removed fundamentals continue to exist.
    ///
    /// # Panics
    ///
    /// Panics if the molecule is not in the map.
    pub(crate) fn dissolve_molecule(
        &mut self,
        molecule: Molecule,
    ) -> impl Iterator<Item = impl Fundamental> {
        let mol = self
            .molecules
            .remove(molecule.into())
            .expect("Caller is required to ensure that the Molecule is valid");
        mol.members.into_iter()
    }
}

/// Methods to query or ascertain membership.
impl MolGraph {
    /// Determines the substituent that contains the atom, pseudoatom, or bond, if any.
    pub(crate) fn parent_substituent(&self, fundamental: impl Fundamental) -> Option<Substituent> {
        for (sub, sub_data) in self.substituents.iter() {
            if sub_data.members.contains(&fundamental.as_fundamental()) {
                return Some(sub.into());
            }
        }
        None
    }

    /// Determines the molecule that contains the atom, pseudoatom, or bond, if any.
    pub(crate) fn parent_molecule(&self, fundamental: impl Fundamental) -> Option<Molecule> {
        for (mol, mol_data) in self.molecules.iter() {
            if mol_data.members.contains(&fundamental.as_fundamental()) {
                return Some(mol.into());
            }
        }
        None
    }
}
