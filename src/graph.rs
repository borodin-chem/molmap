// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use slotmap::SlotMap;

use crate::{
    BondType, Element, Pseudoelement,
    entities::{substituent::SubstituentCentre, *},
    ids::*,
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
    pub(crate) atoms: SlotMap<AtomId, Atom>,
    pub(crate) pseudoatoms: SlotMap<PseudoatomId, Pseudoatom>,
    pub(crate) bonds: SlotMap<BondId, Bond>,
    pub(crate) substituents: SlotMap<SubstituentId, Substituent>,
    pub(crate) molecules: SlotMap<MoleculeId, Molecule>,
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

/// Methods for querying entity IDs.
impl MolGraph {
    /// Returns an iterator over all the IDs of all atoms in the map.
    pub(crate) fn atom_ids(&'_ self) -> impl Iterator<Item = AtomId> + '_ {
        self.atoms.keys()
    }

    /// Returns an iterator over all the IDs of all pseudoatoms in the map.
    pub(crate) fn pseudoatom_ids(&'_ self) -> impl Iterator<Item = PseudoatomId> + '_ {
        self.pseudoatoms.keys()
    }

    /// Returns an iterator over all the IDs of all bonds in the map.
    pub(crate) fn bond_ids(&'_ self) -> impl Iterator<Item = BondId> + '_ {
        self.bonds.keys()
    }

    /// Returns an iterator over all the IDs of all substituents in the map.
    pub(crate) fn substituent_ids(&'_ self) -> impl Iterator<Item = SubstituentId> + '_ {
        self.substituents.keys()
    }

    /// Returns an iterator over all the IDs of all molecules in the map.
    pub(crate) fn molecule_ids(&'_ self) -> impl Iterator<Item = MoleculeId> + '_ {
        self.molecules.keys()
    }

    /// Checks if the given ID is valid.
    pub(crate) fn contains_atom(&self, id: AtomId) -> bool {
        self.atoms.contains_key(id)
    }

    /// Checks if the given ID is valid.
    pub(crate) fn contains_pseudoatom(&self, id: PseudoatomId) -> bool {
        self.pseudoatoms.contains_key(id)
    }

    /// Checks if the given ID is valid.
    pub(crate) fn contains_bond(&self, id: BondId) -> bool {
        self.bonds.contains_key(id)
    }

    /// Checks if the given ID is valid.
    pub(crate) fn contains_substituent(&self, id: SubstituentId) -> bool {
        self.substituents.contains_key(id)
    }

    /// Checks if the given ID is valid.
    pub(crate) fn contains_molecule(&self, id: MoleculeId) -> bool {
        self.molecules.contains_key(id)
    }

    /// Checks if the map currently contains the atomlike with the given ID.
    pub(crate) fn contains_atomlike(&self, atomlike: AtomlikeId) -> bool {
        match atomlike.to_tagged() {
            TaggedAtomlike::Atom(id) => self.contains_atom(id),
            TaggedAtomlike::Pseudoatom(id) => self.contains_pseudoatom(id),
        }
    }

    /// Checks if the map currently contains the fundamental with the given ID.
    pub(crate) fn contains_fundamental(&self, fundamental: FundamentalId) -> bool {
        match fundamental.to_tagged() {
            TaggedFundamental::Atom(id) => self.contains_atom(id),
            TaggedFundamental::Pseudoatom(id) => self.contains_pseudoatom(id),
            TaggedFundamental::Bond(id) => self.contains_bond(id),
        }
    }

    /// Checks if the map currently contains the bondable with the given ID.
    pub(crate) fn contains_bondable(&self, bondable: BondableId) -> bool {
        match bondable.to_tagged() {
            TaggedBondable::Atom(id) => self.contains_atom(id),
            TaggedBondable::Pseudoatom(id) => self.contains_pseudoatom(id),
        }
    }

    /// Checks if the map currently contains the collection with the given ID.
    pub(crate) fn contains_collection(&self, collection: CollectionId) -> bool {
        match collection.to_tagged() {
            TaggedCollection::Substituent(id) => self.contains_substituent(id),
            TaggedCollection::Molecule(id) => self.contains_molecule(id),
        }
    }

    /// Checks if the map currently contains the entity with the given ID.
    pub(crate) fn contains(&self, entity: EntityId) -> bool {
        match entity.to_tagged() {
            TaggedEntity::Atom(id) => self.contains_atom(id),
            TaggedEntity::Pseudoatom(id) => self.contains_pseudoatom(id),
            TaggedEntity::Bond(id) => self.contains_bond(id),
            TaggedEntity::Substituent(id) => self.contains_substituent(id),
            TaggedEntity::Molecule(id) => self.contains_molecule(id),
        }
    }
}

/// Methods for entity addition and deletion.
impl MolGraph {
    /// Adds an atom to the map.
    pub(crate) fn add_atom(&mut self, element: Element) -> AtomId {
        self.atoms.insert(Atom::new(element))
    }

    /// Adds a pseudoatom to the map.
    pub(crate) fn add_pseudoatom(&mut self, pseudoelement: Pseudoelement) -> PseudoatomId {
        self.pseudoatoms.insert(Pseudoatom::new(pseudoelement))
    }

    /// Creates a new (single covalent) bond between two bondable entities.
    ///
    /// # Panics
    ///
    /// Panics if either of `start` and `end` are invalid.
    pub(crate) fn add_bond(&mut self, start: BondableId, end: BondableId) -> BondId {
        let bond_id = self
            .bonds
            .insert(Bond::new(BondType::Covalent, 1.0, start, end));
        for partner in [start, end] {
            match partner.to_tagged() {
                TaggedBondable::Atom(id) => self.atoms.get_mut(id).unwrap().bonds.push(bond_id),
                TaggedBondable::Pseudoatom(id) => {
                    self.pseudoatoms.get_mut(id).unwrap().bonds.push(bond_id)
                }
            }
        }
        bond_id
    }

    /// Adds an empty substituent to the map.
    ///
    /// If the atomlike that is going to be the substituent's centre already
    /// exists, prefer `add_substituent_with_centre()`.
    pub(crate) fn add_substituent(&mut self) -> SubstituentId {
        self.substituents.insert(Substituent {
            centre: SubstituentCentre::None,
            members: Vec::new(),
        })
    }

    /// Adds a substituent to the map with the given atomlike as its centre.
    ///
    /// Note that this method will not fail, even if `centre` is an invalid ID.
    pub(crate) fn add_substituent_with_centre(&mut self, centre: AtomlikeId) -> SubstituentId {
        self.substituents
            .insert(Substituent::new(centre, &[centre.into()]))
    }

    /// Adds an empty molecule to the map.
    pub(crate) fn add_molecule(&mut self) -> MoleculeId {
        self.molecules.insert(Molecule::new())
    }

    /// Removes an atom from the map, as well as any bonds to it.
    ///
    /// Returns whether the atom was present in the map.
    ///
    /// This is infallible – if the atom is not in the map, nothing changes.
    pub(crate) fn delete_atom(&mut self, id: AtomId) -> bool {
        if !self.contains_atom(id) {
            return false;
        }
        // Make sure we always remove bonds first
        let bonds = self.atoms.get(id).unwrap().bonds.clone();
        for bond_id in bonds {
            self.delete_bond(bond_id);
        }
        // Remove from any collections
        if let Some(frag_id) = self.parent_substituent(id.into()) {
            self.remove_from_substituent(frag_id, id.into());
        }
        if let Some(mol_id) = self.parent_molecule(id.into()) {
            self.remove_from_molecule(mol_id, id.into());
        }
        // Now we can safely remove the atom itself without leaving dangling bonds
        self.atoms.remove(id).is_some() // Should always be `true`
    }

    /// Removes a pseudoatom from the map, as well as any bonds to it.
    ///
    /// Returns whether the pseudoatom was present in the map.
    ///
    /// This is infallible – if the pseudoatom is not in the map, nothing changes.
    pub(crate) fn delete_pseudoatom(&mut self, id: PseudoatomId) -> bool {
        if !self.contains_pseudoatom(id) {
            return false;
        }
        // Make sure we always remove bonds first
        let bonds = self.pseudoatoms.get(id).unwrap().bonds.clone();
        for bond_id in bonds {
            self.delete_bond(bond_id);
        }
        // Remove from any collections
        if let Some(frag_id) = self.parent_substituent(id.into()) {
            self.remove_from_substituent(frag_id, id.into());
        }
        if let Some(mol_id) = self.parent_molecule(id.into()) {
            self.remove_from_molecule(mol_id, id.into());
        }
        // Now we can safely remove the pseudoatom itself without leaving dangling bonds
        self.pseudoatoms.remove(id).is_some()
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
    pub(crate) fn delete_bond(&mut self, id: BondId) -> bool {
        if let Some(bond) = self.bonds.remove(id) {
            for bonding_partner in [bond.start, bond.end] {
                match bonding_partner.to_tagged() {
                    TaggedBondable::Atom(atom_id) => {
                        let mut atom = self
                            .atoms
                            .get_mut(atom_id)
                            .expect("Bonds are always removed before their bonding partners");
                        let pos = atom.bonds.iter().position(|x| *x == id).expect(
                            "Bond should be listed in the bonding partner's bonds until deletion",
                        );
                        atom.bonds.remove(pos);
                    }
                    TaggedBondable::Pseudoatom(pseudoatom_id) => {
                        let mut pseudoatom = self
                            .pseudoatoms
                            .get_mut(pseudoatom_id)
                            .expect("Bonds are always removed before their bonding partners");
                        let pos = pseudoatom.bonds.iter().position(|x| *x == id).expect(
                            "Bond should be listed in the bonding partner's bonds until deletion",
                        );
                        pseudoatom.bonds.remove(pos);
                    }
                }
            }
            // Remove from any collections
            if let Some(frag_id) = self.parent_substituent(id.into()) {
                self.remove_from_substituent(frag_id, id.into());
            }
            if let Some(mol_id) = self.parent_molecule(id.into()) {
                self.remove_from_molecule(mol_id, id.into());
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
    pub(crate) fn delete_substituent(&mut self, id: SubstituentId) -> bool {
        if !self.contains_substituent(id) {
            return false;
        };
        let members = self.substituents.get(id).unwrap().members.clone();
        for member in members {
            match member.to_tagged() {
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
        self.substituents.remove(id).is_some()
    }

    /// Removes a molecule from the map, as well as all of its members.
    ///
    /// Returns whether the molecule was present in the map.
    ///
    /// This is infallible – if the molecule is not in the map, nothing changes.
    pub(crate) fn delete_molecule(&mut self, id: MoleculeId) -> bool {
        if !self.contains_molecule(id) {
            return false;
        };
        let members = self.molecules.get(id).unwrap().members.clone();
        for member in members {
            match member.to_tagged() {
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
        self.molecules.remove(id).is_some()
    }

    /// Removes an atom or pseudoatom from the map.
    ///
    /// Returns whether the atomlike was present in the map.
    ///
    /// Bonds to the atom or pseudoatom are also deleted.
    ///
    /// If the atomlike is not in the map, nothing changes.
    pub(crate) fn delete_atomlike(&mut self, atomlike: AtomlikeId) -> bool {
        match atomlike.to_tagged() {
            TaggedAtomlike::Atom(id) => self.delete_atom(id),
            TaggedAtomlike::Pseudoatom(id) => self.delete_pseudoatom(id),
        }
    }

    /// Removes an atom, pseudoatom, or bond from the map.
    ///
    /// Returns whether the fundamental was present in the map.
    ///
    /// Bonds to an atom or pseudoatom are also deleted.
    /// The bonding partners of a bond are not deleted.
    ///
    /// If the fundamental is not in the map, nothing changes.
    pub(crate) fn delete_fundamental(&mut self, fundamental: FundamentalId) -> bool {
        match fundamental.to_tagged() {
            TaggedFundamental::Atom(id) => self.delete_atom(id),
            TaggedFundamental::Pseudoatom(id) => self.delete_pseudoatom(id),
            TaggedFundamental::Bond(id) => self.delete_bond(id),
        }
    }

    /// Removes a collection from the map, as well as all of its members.
    ///
    /// Returns whether the collection was present in the map.
    ///
    /// If the collection is not in the map, nothing changes.
    pub(crate) fn delete_collection(&mut self, collection: CollectionId) -> bool {
        match collection.to_tagged() {
            TaggedCollection::Substituent(id) => self.delete_substituent(id),
            TaggedCollection::Molecule(id) => self.delete_molecule(id),
        }
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
        substituent: SubstituentId,
        fundamental: FundamentalId,
    ) -> bool {
        let sub = self.substituents.get_mut(substituent).unwrap();
        if !sub.members.contains(&fundamental) {
            sub.members.push(fundamental);
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
        molecule: MoleculeId,
        fundamental: FundamentalId,
    ) -> bool {
        let mol = self.molecules.get_mut(molecule).unwrap();
        mol.members.insert(fundamental)
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
        substituent: SubstituentId,
        fundamental: FundamentalId,
    ) -> bool {
        let sub = self.substituents.get_mut(substituent).unwrap();
        if let Some(index) = sub.members.iter().position(|x| *x == fundamental) {
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
                if FundamentalId::from(*atomlike) == fundamental {
                    // No longer has a centre
                    sub.centre = SubstituentCentre::None
                }
            }
            SubstituentCentre::Multiple(atomlikes) => {
                if let Some(atomlike) = match fundamental.to_tagged() {
                    TaggedFundamental::Bond(_) => None,
                    TaggedFundamental::Atom(id) => Some(id.into()),
                    TaggedFundamental::Pseudoatom(id) => Some(id.into()),
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
        molecule: MoleculeId,
        fundamental: FundamentalId,
    ) -> bool {
        let mol = self.molecules.get_mut(molecule).unwrap();
        mol.members.remove(&fundamental)
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
        id: SubstituentId,
    ) -> impl Iterator<Item = FundamentalId> {
        let sub = self
            .substituents
            .get_mut(id)
            .expect("Caller is required to ensure that the SubstituentId is valid");
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
    pub(crate) fn drain_molecule(&mut self, id: MoleculeId) -> impl Iterator<Item = FundamentalId> {
        let mut mol = self
            .molecules
            .get_mut(id)
            .expect("Caller is required to ensure that the MoleculeId is valid");
        mol.members.drain()
    }

    /// Empties a substituent by deleting all its members.
    ///
    /// The substituent itself continues to exist, and will be centreless.
    ///
    /// # Panics
    ///
    /// Panics if the substituent is not in the map.
    pub(crate) fn clear_substituent(&mut self, id: SubstituentId) {
        let sub = self
            .substituents
            .get_mut(id)
            .expect("Caller is required to ensure that the SubstituentId is valid");
        sub.centre = SubstituentCentre::None;
        let former_members: Vec<FundamentalId> = sub.members.drain(..).collect();
        // It's fine to delete in any order as if something isn't in the map any more
        // (e.g. because it's a bond and one of its bonding partners was already deleted
        // and thus it too was already deleted) then nothing changes when the deletion
        // is attempted
        for member in former_members {
            match member.to_tagged() {
                TaggedFundamental::Atom(id) => self.delete_atom(id),
                TaggedFundamental::Pseudoatom(id) => self.delete_pseudoatom(id),
                TaggedFundamental::Bond(id) => self.delete_bond(id),
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
    pub(crate) fn clear_molecule(&mut self, id: MoleculeId) {
        let mut mol = self
            .molecules
            .get_mut(id)
            .expect("Caller is required to ensure that the MoleculeId is valid");
        let former_members: Vec<FundamentalId> = mol.members.drain().collect();
        // It's fine to delete in any order as if something isn't in the map any more
        // (e.g. because it's a bond and one of its bonding partners was already deleted
        // and thus it too was already deleted) then nothing changes when the deletion
        // is attempted
        for member in former_members {
            match member.to_tagged() {
                TaggedFundamental::Atom(id) => self.delete_atom(id),
                TaggedFundamental::Pseudoatom(id) => self.delete_pseudoatom(id),
                TaggedFundamental::Bond(id) => self.delete_bond(id),
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
        id: SubstituentId,
    ) -> impl Iterator<Item = FundamentalId> {
        let sub = self
            .substituents
            .remove(id)
            .expect("Caller is required to ensure that the MoleculeId is valid");
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
        id: MoleculeId,
    ) -> impl Iterator<Item = FundamentalId> {
        let mol = self
            .molecules
            .remove(id)
            .expect("Caller is required to ensure that the MoleculeId is valid");
        mol.members.into_iter()
    }
}

/// Private helper methods.
impl MolGraph {
    /// Determines the substituent that contains the atom, pseudoatom, or bond, if any.
    fn parent_substituent(&self, fundamental: FundamentalId) -> Option<SubstituentId> {
        for (substituent_id, substituent) in self.substituents.iter() {
            if substituent.members.contains(&fundamental) {
                return Some(substituent_id);
            }
        }
        None
    }

    /// Determines the molecule that contains the atom, pseudoatom, or bond, if any.
    fn parent_molecule(&self, fundamental: FundamentalId) -> Option<MoleculeId> {
        for (mol_id, mol) in self.molecules.iter() {
            if mol.members.contains(&fundamental) {
                return Some(mol_id);
            }
        }
        None
    }
}
