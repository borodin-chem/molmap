// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Definition of the data structure that holds the core molecular graph.

use slotmap::SlotMap;

use crate::{
    Element, Pseudoelement,
    entities::*,
    error::{MolMapError, MolMapResult},
    graph::{data::*, keys::*},
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
/// The methods should do relatively little checking and validation, with the
/// higher maps responsible for careful usage. In particular, all IDs should be
/// assumed to be valid.
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
    pub(crate) fn slotmap<E: Kind>(&self) -> &SlotMap<E::KEY, E::DATA> {
        E::get_slotmap(self)
    }

    /// Returns a mutable reference to the `SlotMap` that holds the entity.
    #[inline]
    pub(crate) fn slotmap_mut<E: Kind>(&mut self) -> &mut SlotMap<E::KEY, E::DATA> {
        E::get_slotmap_mut(self)
    }

    /// Returns a reference to the entity's data struct, or `None` if `entity` is invalid.
    #[inline]
    pub(crate) fn data<E: Kind>(&self, entity: E) -> Option<&E::DATA> {
        self.slotmap::<E>().get(entity.to_key())
    }

    /// Returns a mutable reference to the entity's data struct, or `None` if `entity` is invalid.
    #[inline]
    pub(crate) fn data_mut<E: Kind>(&mut self, entity: E) -> Option<&mut E::DATA> {
        self.slotmap_mut::<E>().get_mut(entity.to_key())
    }

    /// Checks if the map currently contains the given entity.
    pub(crate) fn contains<E: Kind>(&self, entity: E) -> bool {
        self.slotmap::<E>().contains_key(entity.to_key())
    }

    /// Returns an iterator over all the keys of a given kind of entity in the map.
    pub(crate) fn keys<E: Kind>(&'_ self) -> slotmap::basic::Keys<'_, E::KEY, E::DATA> {
        self.slotmap::<E>().keys()
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

    /// Creates a new bond between two bondable entities.
    ///
    /// # Panics
    ///
    /// Panics if either of `start` and `end` are invalid.
    pub(crate) fn add_bond(
        &mut self,
        bond_type: BondType,
        start: impl Bondable,
        end: impl Bondable,
    ) -> Bond {
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
        let bonds = self.data(atom).unwrap().bonds.clone();
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
        let bonds = self.data(pseudoatom).unwrap().bonds.clone();
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
        let members = self.data(substituent).unwrap().members.clone();
        for member in members {
            match member.resolve() {
                ResolvedFundamental::Atom(atom) => {
                    self.delete_atom(atom);
                }
                ResolvedFundamental::Pseudoatom(pseudoatom) => {
                    self.delete_pseudoatom(pseudoatom);
                }
                ResolvedFundamental::Bond(bond) => {
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
        let members = self.data(molecule).unwrap().members.clone();
        for member in members {
            match member.resolve() {
                ResolvedFundamental::Atom(id) => {
                    self.delete_atom(id);
                }
                ResolvedFundamental::Pseudoatom(id) => {
                    self.delete_pseudoatom(id);
                }
                ResolvedFundamental::Bond(id) => {
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
    /// If the fundamental is already a member of another substituent, it is removed
    /// from it before it is inserted into this one.
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
        if let Some(parent) = self.parent_substituent(fundamental) {
            if parent == substituent {
                // Already a member of this substituent
                return false;
            } else {
                self.remove_from_substituent(parent, fundamental);
            }
        }
        self.insert_into_substituent_unchecked(substituent, fundamental)
    }

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
    pub(crate) fn insert_into_substituent_unchecked(
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
    /// If the fundamental is already a member of another molecule, it is removed
    /// from it before it is inserted into this one.
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
        if let Some(parent) = self.parent_molecule(fundamental) {
            if parent == molecule {
                // Already a member of this molecule
                return false;
            } else {
                self.remove_from_molecule(parent, fundamental);
            }
        }
        self.insert_into_molecule_unchecked(molecule, fundamental)
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
    pub(crate) fn insert_into_molecule_unchecked(
        &mut self,
        molecule: Molecule,
        fundamental: impl Fundamental,
    ) -> bool {
        let mol = self.molecules.get_mut(molecule.into()).unwrap();
        mol.members.insert(fundamental.as_fundamental())
    }

    /// Adds atoms, pseudoatoms, or bonds from an iterator to a substituent.
    ///
    /// This method should only ever be used with fundamentals that do not already
    /// belong to a substituent.
    ///
    /// # Panics
    ///
    /// Panics if `substituent` is invalid, but is unaffected if any of the fundamental
    /// IDs are invalid.
    #[allow(unused)]
    pub(crate) fn extend_substituent_unchecked<I, E>(
        &mut self,
        substituent: Substituent,
        fundamentals: I,
    ) where
        I: IntoIterator<Item = E>,
        E: Fundamental,
    {
        let sub = self.substituents.get_mut(substituent.into()).unwrap();
        sub.members
            .extend(fundamentals.into_iter().map(|e| e.as_fundamental()))
    }

    /// Adds atoms, pseudoatoms, or bonds from an iterator to a molecule.
    ///
    /// This method should only ever be used with fundamentals that do not already
    /// belong to a molecule.
    ///
    /// # Panics
    ///
    /// Panics if `molecule` is invalid, but is unaffected if any of the fundamental
    /// IDs are invalid.
    #[allow(unused)]
    pub(crate) fn extend_molecule_unchecked<I, E>(&mut self, molecule: Molecule, fundamentals: I)
    where
        I: IntoIterator<Item = E>,
        E: Fundamental,
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
                if let Some(atomlike) = match Fundamental::to_resolved(fundamental) {
                    ResolvedFundamental::Bond(_) => None,
                    ResolvedFundamental::Atom(atom) => Some(atom.into()),
                    ResolvedFundamental::Pseudoatom(pseudoatom) => Some(pseudoatom.into()),
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
    ) -> impl Iterator<Item = AnyFundamental> {
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
    ) -> impl Iterator<Item = AnyFundamental> {
        let mol = self
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
        let former_members: Vec<AnyFundamental> = std::mem::take(&mut sub.members);
        // It's fine to delete in any order as if something isn't in the map any more
        // (e.g. because it's a bond and one of its bonding partners was already deleted
        // and thus it too was already deleted) then nothing changes when the deletion
        // is attempted
        for member in former_members {
            match member.resolve() {
                ResolvedFundamental::Atom(atom) => self.delete_atom(atom),
                ResolvedFundamental::Pseudoatom(pseudoatom) => self.delete_pseudoatom(pseudoatom),
                ResolvedFundamental::Bond(bond) => self.delete_bond(bond),
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
        let mol = self
            .molecules
            .get_mut(molecule.into())
            .expect("Caller is required to ensure that the Molecule is valid");
        let former_members: Vec<AnyFundamental> = mol.members.drain().collect();
        // It's fine to delete in any order as if something isn't in the map any more
        // (e.g. because it's a bond and one of its bonding partners was already deleted
        // and thus it too was already deleted) then nothing changes when the deletion
        // is attempted
        for member in former_members {
            match member.resolve() {
                ResolvedFundamental::Atom(atom) => self.delete_atom(atom),
                ResolvedFundamental::Pseudoatom(pseudoatom) => self.delete_pseudoatom(pseudoatom),
                ResolvedFundamental::Bond(bond) => self.delete_bond(bond),
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
    ) -> impl Iterator<Item = AnyFundamental> {
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
    ) -> impl Iterator<Item = AnyFundamental> {
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

impl MolGraph {
    /// Attempts to change the centre of the substituent to the one requested.
    ///
    /// # Errors
    ///
    /// Fails if the requested centre is not already a member of the substituent,
    /// or if there are already bonds to the current centre(s).
    ///
    /// # Panics
    ///
    /// Panics if `substituent` is invalid.
    pub(crate) fn set_substituent_centre(
        &mut self,
        substituent: Substituent,
        new_centre: impl Atomlike,
    ) -> MolMapResult<()> {
        // First confirm that the new centre is actually a member of the substituent
        let sub_data = self
            .data(substituent)
            .expect("Caller is required to ensure that the molecule is valid");
        if !sub_data
            .members
            .contains(&new_centre.as_atomlike().as_fundamental())
        {
            return Err(MolMapError::Membership(
                new_centre.as_atomlike().as_fundamental(),
            ));
        }
        // A closure that determines if an atom or pseudoatom has bonds already
        let atomlike_has_bonds = |id: AnyAtomlike| -> bool {
            let bonds = match id.resolve() {
                ResolvedAtomlike::Atom(atom) => {
                    &self
                        .data(atom)
                        .expect("Wouldn't be listed as the centre if it had been removed")
                        .bonds
                }
                ResolvedAtomlike::Pseudoatom(pseudoatom) => {
                    &self
                        .data(pseudoatom)
                        .expect("Wouldn't be listed as the centre if it had been removed")
                        .bonds
                }
            };
            !bonds.is_empty()
        };
        // Check that there aren't already bonds to the current centre
        let already_bonded = match &sub_data.centre {
            SubstituentCentre::None => false,
            SubstituentCentre::Single(atomlike_id) => atomlike_has_bonds(*atomlike_id),
            SubstituentCentre::Multiple(atomlike_ids) => {
                atomlike_ids.iter().copied().any(atomlike_has_bonds)
            }
        };
        if already_bonded {
            Err(MolMapError::Disallowed(String::from(
                "Substituent already has at least one bond to its centre(s)",
            )))
        } else {
            self.data_mut(substituent)
                .expect("Already validated")
                .centre = SubstituentCentre::Single(new_centre.as_atomlike());
            Ok(())
        }
    }

    /// Makes the requested atomlike a centre of the substituent, in addition to any
    /// already existing centres.
    ///
    /// # Errors
    ///
    /// Fails if the requested centre is not already a member of the substituent.
    ///
    /// # Panics
    ///
    /// Panics if `substituent` is invalid.
    pub(crate) fn add_substituent_centre(
        &mut self,
        substituent: Substituent,
        new_centre: impl Atomlike,
    ) -> MolMapResult<()> {
        // First confirm that the new centre is actually a member of the substituent
        let sub_data = self
            .data_mut(substituent)
            .expect("Caller is required to ensure that the molecule is valid");
        if !sub_data
            .members
            .contains(&new_centre.as_atomlike().as_fundamental())
        {
            return Err(MolMapError::Membership(
                new_centre.as_atomlike().as_fundamental(),
            ));
        }
        match &mut sub_data.centre {
            SubstituentCentre::None => {
                self.data_mut(substituent)
                    .expect("Already validated")
                    .centre = SubstituentCentre::Single(new_centre.as_atomlike())
            }
            SubstituentCentre::Single(existing_centre) => {
                self.data_mut(substituent)
                    .expect("Already validated")
                    .centre = SubstituentCentre::Multiple(Box::new(vec![
                    *existing_centre,
                    new_centre.as_atomlike(),
                ]))
            }
            SubstituentCentre::Multiple(existing_centres) => {
                existing_centres.push(new_centre.as_atomlike())
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(unused)]
pub(crate) mod tests {
    use super::*;

    /// Creates a basic graph to use as the basis for various tests.
    ///
    /// The graph contains:
    /// - one molecule (CH3OH)
    /// - two substituents (CH3, OH)
    /// - six atoms
    /// - five bonds
    pub(crate) fn meoh_graph() -> MolGraph {
        let mut g = MolGraph::new();
        let h1 = g.add_atom(Element::H);
        let h2 = g.add_atom(Element::H);
        let h3 = g.add_atom(Element::H);
        let c1 = g.add_atom(Element::C);
        let c1h1 = g.add_bond(BondType::Covalent { order: 1.0 }, c1, h1);
        let c1h2 = g.add_bond(BondType::Covalent { order: 1.0 }, c1, h2);
        let c1h3 = g.add_bond(BondType::Covalent { order: 1.0 }, c1, h3);
        let methyl = g.add_substituent_with_centre(c1);
        g.extend_substituent_unchecked(methyl, [h1, h2, h3]);
        g.extend_substituent_unchecked(methyl, [c1h1, c1h2, c1h3]);
        let o1 = g.add_atom(Element::O);
        let h4 = g.add_atom(Element::H);
        let o1h4 = g.add_bond(BondType::Covalent { order: 1.0 }, o1, h4);
        let hydroxy = g.add_substituent_with_centre(o1);
        g.insert_into_substituent_unchecked(hydroxy, h4);
        g.insert_into_substituent_unchecked(hydroxy, o1h4);
        let c1o1 = g.add_bond(BondType::Covalent { order: 1.0 }, c1, o1);
        let mol = g.add_molecule();
        g.extend_molecule_unchecked(mol, g.data(methyl).unwrap().members.clone());
        g.extend_molecule_unchecked(mol, g.data(hydroxy).unwrap().members.clone());
        // Inter-substituent bond isn't member of either, but needs to be added too
        g.insert_into_molecule_unchecked(mol, c1o1);
        g
    }

    #[test]
    fn add_atom() {
        let mut g = MolGraph::new();
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
        let mut g = MolGraph::new();
        assert!(g.pseudoatoms.is_empty());
        let ph = g.add_pseudoatom(Pseudoelement::Ph);
        assert_eq!(g.pseudoatoms.len(), 1);
    }

    #[test]
    fn delete_atom() {
        let mut g = MolGraph::new();
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
        let mut g = MolGraph::new();
        let et = g.add_pseudoatom(Pseudoelement::Et);
        assert_eq!(g.pseudoatoms.len(), 1);
        g.delete_pseudoatom(et);
        assert!(g.pseudoatoms.is_empty());
    }

    #[test]
    fn slotmap() {
        let g = MolGraph::new();
        // Make sure that the generic slotmap method returns a reference to the appropriate slotmap
        assert!(std::ptr::eq(g.slotmap::<Atom>(), &g.atoms));
        assert!(std::ptr::eq(g.slotmap::<Bond>(), &g.bonds));
        assert!(std::ptr::eq(g.slotmap::<Molecule>(), &g.molecules));
    }

    #[test]
    fn data() {
        let mut g = MolGraph::new();
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
        let mut g = MolGraph::new();
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
    fn add_bond_between_atoms() {
        let mut g = MolGraph::new();
        assert!(g.bonds.is_empty());
        let h1 = g.add_atom(Element::H);
        let h2 = g.add_atom(Element::H);
        let b1 = g.add_bond(BondType::Covalent { order: 1.0 }, h1, h2);
        assert_eq!(g.bonds.len(), 1);
        assert!(g.contains(b1));
        assert!(g.atoms.get(h1.into()).unwrap().bonds.contains(&b1));
        assert!(g.atoms.get(h2.into()).unwrap().bonds.contains(&b1));
        assert_eq!(g.bonds.get(b1.into()).unwrap().start, h1.into());
        assert_eq!(g.bonds.get(b1.into()).unwrap().end, h2.into());
    }

    #[test]
    fn delete_bond_between_atoms() {
        let mut g = MolGraph::new();
        let h1 = g.add_atom(Element::H);
        let h2 = g.add_atom(Element::H);
        let b1 = g.add_bond(BondType::Covalent { order: 1.0 }, h1, h2);
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
        let mut g = MolGraph::new();
        let h1 = g.add_atom(Element::H);
        let h2 = g.add_atom(Element::H);
        let b1 = g.add_bond(BondType::Covalent { order: 1.0 }, h1, h2);
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

    #[test]
    fn delete_substituent() {
        let mut g = meoh_graph();
        let mut subs = g.substituents.iter();
        let (mut key0, data0) = subs.next().unwrap(); // Come in insertion order
        let (mut key1, data1) = subs.next().unwrap();
        let (methyl, hydroxy): (Substituent, Substituent) = match data0.members.len() {
            7 => (key0.into(), key1.into()),
            3 => (key1.into(), key0.into()),
            _ => panic!(),
        };
        let c1: Atom = match g.data(methyl).unwrap().centre {
            SubstituentCentre::Single(any_atomlike) => any_atomlike.try_into().unwrap(),
            _ => panic!(),
        };
        let o1: Atom = match g.data(hydroxy).unwrap().centre {
            SubstituentCentre::Single(any_atomlike) => any_atomlike.try_into().unwrap(),
            _ => panic!(),
        };
        assert_eq!(g.data(c1).unwrap().element, Element::C);
        assert_eq!(g.data(o1).unwrap().element, Element::O);
        // Carbon should have four bonds
        assert_eq!(g.data(c1).unwrap().bonds.len(), 4);
        // Delete the hydroxy group
        assert!(g.delete_substituent(hydroxy));
        // Everything that was in the hydroxy group should now be gone
        assert_eq!(g.substituents.len(), 1);
        assert!(!g.contains(hydroxy));
        assert_eq!(g.atoms.len(), 4); // Two fewer
        assert!(!g.contains(o1));
        // C–O bond should be gone (and O–H of course)
        assert_eq!(g.data(c1).unwrap().bonds.len(), 3);
        // Carbon/methyl should still be present
        assert!(g.contains(c1));
        assert!(g.contains(methyl));
        // Molecule shouldn't have gone anywhere
        assert_eq!(g.molecules.len(), 1);
    }

    #[test]
    fn delete_molecule() {
        let mut g = meoh_graph();
        let methanol: Molecule = g.molecules.keys().next().unwrap().into(); // Contains exactly one molecule
        g.delete_molecule(methanol);
        // Everything was in the molecule, so everything should now be gone
        assert!(g.atoms.is_empty());
        assert!(g.bonds.is_empty());
        assert!(g.molecules.is_empty());
        // Substituents will still be present but should be empty
        assert_eq!(g.substituents.len(), 2);
        for (sub, sub_data) in g.substituents {
            assert!(sub_data.members.is_empty());
        }
    }

    #[test]
    fn insert_into_sub() {
        let mut g = MolGraph::new();
        let h1 = g.add_atom(Element::H);
        let h2 = g.add_atom(Element::H);
        let s1 = g.add_substituent();
        assert!(g.data(s1).unwrap().members.is_empty());
        assert!(g.insert_into_substituent(s1, h1));
        assert_eq!(g.data(s1).unwrap().members, [h1.as_fundamental()]);
        // Repeated insertion has no effect, returns false if already present
        assert!(!g.insert_into_substituent(s1, h1));
        assert_eq!(g.data(s1).unwrap().members, [h1.as_fundamental()]);
        let s2 = g.add_substituent();
        // Inserting an atom that is already in a substituent into another moves
        // it when the normal (as opposed to unchecked) method is used
        g.insert_into_substituent(s2, h1);
        assert!(g.data(s1).unwrap().members.is_empty());
        assert_eq!(g.data(s2).unwrap().members, [h1.as_fundamental()]);
        g.insert_into_substituent(s2, h2);
        assert_eq!(
            g.data(s2).unwrap().members,
            [h1.as_fundamental(), h2.as_fundamental()]
        );
    }

    #[test]
    fn insert_into_mol() {
        let mut g = MolGraph::new();
        let h1 = g.add_atom(Element::H);
        let h2 = g.add_atom(Element::H);
        let m1 = g.add_molecule();
        assert!(g.data(m1).unwrap().members.is_empty());
        g.insert_into_molecule(m1, h1);
        assert_eq!(g.data(m1).unwrap().members.len(), 1);
        let m2 = g.add_molecule();
        // Inserting an atom that is already in a molecule into another moves
        // it when the normal (as opposed to unchecked) method is used
        g.insert_into_molecule(m2, h1);
        assert!(g.data(m1).unwrap().members.is_empty());
        assert_eq!(g.data(m2).unwrap().members.len(), 1);
        g.insert_into_molecule(m2, h2);
        assert_eq!(g.data(m2).unwrap().members.len(), 2);
    }

    #[test]
    fn remove_from_sub() {
        let mut g = meoh_graph();
        let c1 = g
            .atoms
            .iter()
            .find_map(|(k, d)| (d.element == Element::C).then_some(Atom::from_key(k)))
            .unwrap();
        let o1 = g
            .atoms
            .iter()
            .find_map(|(k, d)| (d.element == Element::O).then_some(Atom::from_key(k)))
            .unwrap();
        let methyl = g.parent_substituent(c1).unwrap();
        let mut h1: Option<Atom> = None;
        for &fund in g.data(methyl).unwrap().members.iter() {
            match fund.resolve() {
                ResolvedFundamental::Atom(atom) if atom == c1 => continue,
                ResolvedFundamental::Atom(atom) => {
                    h1 = Some(atom);
                    break;
                }
                ResolvedFundamental::Bond(_) => continue,
                ResolvedFundamental::Pseudoatom(_) => unreachable!(),
            }
        }
        let h1 = h1.unwrap();
        assert_eq!(
            g.data(methyl).unwrap().centre,
            SubstituentCentre::Single(c1.as_atomlike())
        );
        // Removing an atom that isn't a member of the substituent should have no effect
        assert!(!g.remove_from_substituent(methyl, o1));
        assert!(g.contains(o1));
        // Remove a hydrogen
        assert!(g.remove_from_substituent(methyl, h1));
        // Gone from 4 atoms + 3 bonds to 3 + 3
        // (bond isn't deleted, because atom still exists, it's just not in the sub now)
        assert_eq!(g.data(methyl).unwrap().members.len(), 6);
        // Has no effect on the substituent's centre
        assert_eq!(
            g.data(methyl).unwrap().centre,
            SubstituentCentre::Single(c1.as_atomlike())
        );
        // Removing the central carbon, however...
        assert!(g.remove_from_substituent(methyl, c1));
        // Only two hydrogens left but all 3 bonds remain
        assert_eq!(g.data(methyl).unwrap().members.len(), 5);
        // Substituent is now centreless
        assert_eq!(g.data(methyl).unwrap().centre, SubstituentCentre::None);
    }

    #[test]
    fn drain_mol() {
        let mut g = meoh_graph();
        let mol: Molecule = g.molecules.keys().next().unwrap().into();
        let former_members = g.drain_molecule(mol);
        // 6 atoms, 5 bonds
        assert_eq!(former_members.count(), 11);
        // All members should still be present
        assert_eq!(g.atoms.len(), 6);
        assert_eq!(g.bonds.len(), 5);
        // Molecule should remain too, but should now be empty
        assert!(g.contains(mol));
        assert!(g.data(mol).unwrap().members.is_empty());
    }

    #[test]
    fn clear_mol() {
        let mut g = meoh_graph();
        let mol: Molecule = g.molecules.keys().next().unwrap().into();
        g.clear_molecule(mol);
        // All members should be gone
        assert!(g.atoms.is_empty());
        assert!(g.bonds.is_empty());
        // Just the molecule container should remain
        assert!(g.contains(mol));
    }

    #[test]
    fn dissolve_mol() {
        let mut g = meoh_graph();
        let mol: Molecule = g.molecules.keys().next().unwrap().into();
        let former_members = g.dissolve_molecule(mol);
        // 6 atoms, 5 bonds
        assert_eq!(former_members.count(), 11);
        // All members should still be present
        assert_eq!(g.atoms.len(), 6);
        assert_eq!(g.bonds.len(), 5);
        // Just the molecule container should be gone
        assert!(!g.contains(mol));
    }

    #[test]
    fn parent_substituent() {
        let g = meoh_graph();
        let c1 = g
            .atoms
            .iter()
            .find_map(|(k, d)| (d.element == Element::C).then_some(Atom::from_key(k)))
            .unwrap();
        let methyl = g.parent_substituent(c1).unwrap();
        assert_eq!(
            g.data(methyl).unwrap().centre,
            SubstituentCentre::Single(c1.into())
        );
        assert_eq!(g.data(methyl).unwrap().members.len(), 7);
    }

    #[test]
    fn parent_molecule() {
        let g = meoh_graph();
        let c1 = g
            .atoms
            .iter()
            .find_map(|(k, d)| (d.element == Element::C).then_some(Atom::from_key(k)))
            .unwrap();
        assert_eq!(
            g.parent_molecule(c1).unwrap(),
            g.molecules.keys().next().unwrap().into(), // Only one molecule
        );
    }

    #[test]
    fn set_substituent_centre() {
        let mut g = MolGraph::new();
        let c1 = g.add_atom(Element::C);
        let n1 = g.add_atom(Element::N);
        assert_ne!(c1, n1);
        let sub = g.add_substituent_with_centre(c1);
        g.insert_into_substituent(sub, n1);
        // Now we have a CN substituent that bonds at C i.e. a nitrile/cyano group
        // First, sanity checks
        // Both C and N atom should be members
        assert_eq!(
            g.data(sub).unwrap().members,
            [c1.as_fundamental(), n1.as_fundamental()]
        );
        // C should be the centre
        assert_eq!(
            g.data(sub).unwrap().centre,
            SubstituentCentre::Single(c1.as_atomlike()),
        );
        // Now, convert to isonitrile by making the N the centre
        g.set_substituent_centre(sub, n1).unwrap();
        // N should be the centre
        assert_eq!(
            g.data(sub).unwrap().centre,
            SubstituentCentre::Single(n1.as_atomlike()),
        );
        // Should fail if the requested centre isn't a member
        let c2 = g.add_atom(Element::C);
        assert!(g.set_substituent_centre(sub, c2).is_err());
        // Should fail if the current centre has a bond
        let c2n1 = g.add_bond(BondType::Covalent { order: 1.0 }, c2, n1);
        assert!(g.set_substituent_centre(sub, c1).is_err());
    }

    #[test]
    fn add_substituent_centre() {
        let mut g = MolGraph::new();
        let c1 = g.add_atom(Element::C);
        let o1 = g.add_atom(Element::O); // The double-bonded oxygen (designated arbitrarily)
        let o2 = g.add_atom(Element::O);
        let sub = g.add_substituent_with_centre(c1);
        g.insert_into_substituent(sub, o1);
        g.insert_into_substituent(sub, o2);
        // Now we have a COO substituent that bonds at C i.e. carboxyl
        // First, sanity checks
        // All atoms should be members
        assert_eq!(
            g.data(sub).unwrap().members,
            [c1.into(), o1.into(), o2.into()]
        );
        // C should be the centre
        assert_eq!(
            g.data(sub).unwrap().centre,
            SubstituentCentre::Single(c1.into()),
        );
        // Now, let the substituent bond at both carbon and oxygen i.e. as R–COO–R
        g.add_substituent_centre(sub, o2).unwrap();
        // Both C and the second O should be centres
        assert_eq!(
            g.data(sub).unwrap().centre,
            SubstituentCentre::Multiple(Box::new(vec![c1.into(), o2.into()])),
        );
        // Should fail if the requested centre isn't a member
        let c2 = g.add_atom(Element::C);
        assert!(g.set_substituent_centre(sub, c2).is_err());
    }
}
