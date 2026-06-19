// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use slotmap::SlotMap;

use crate::{
    BondType, Element,
    entities::{substituent::SubstituentCentre, *},
    ids::*,
};

/// An arena-like data structure to represent a set of chemical entities,
/// their properties, and the relationships between them, as a molecular graph.
///
/// This forms the core of all `MolMap` types, but is not public.
/// [`MolMap0`] is the `MolMap` type that provides a molecular graph for users.
///
/// In general, the methods of `MolGraph` should be small in scope and efficient
/// so that the higher maps can combine them to create a nice public API.
/// The methods should also do as little checking and validation as possible, with
/// panicking preferred - the higher maps are then responsible for careful usage.
#[derive(Debug, Default)]
pub(crate) struct MolGraph {
    pub(crate) atoms: SlotMap<AtomId, Atom>,
    pub(crate) pseudoatoms: SlotMap<PseudoatomId, Pseudoatom>,
    pub(crate) bonds: SlotMap<BondId, Bond>,
    pub(crate) substituents: SlotMap<SubstituentId, Substituent>,
    pub(crate) molecules: SlotMap<MoleculeId, Molecule>,
}

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

impl MolGraph {
    // Methods to iterate over IDs

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

    // Methods to check the validity of IDs and entity enums

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

    /// Checks if the given enum wraps a valid ID.
    pub(crate) fn contains_atomlike(&self, atomlike: AtomlikeId) -> bool {
        match atomlike {
            AtomlikeId::Atom(id) => self.contains_atom(id),
            AtomlikeId::Pseudoatom(id) => self.contains_pseudoatom(id),
        }
    }

    /// Checks if the given enum wraps a valid ID.
    pub(crate) fn contains_fundamental(&self, fundamental: FundamentalId) -> bool {
        match fundamental {
            FundamentalId::Atom(id) => self.contains_atom(id),
            FundamentalId::Pseudoatom(id) => self.contains_pseudoatom(id),
            FundamentalId::Bond(id) => self.contains_bond(id),
        }
    }

    /// Checks if the given enum wraps a valid ID.
    pub(crate) fn contains_bondable(&self, bondable: BondableId) -> bool {
        match bondable {
            BondableId::Atom(id) => self.contains_atom(id),
            BondableId::Pseudoatom(id) => self.contains_pseudoatom(id),
        }
    }

    /// Checks if the map currently contains the entity with the given ID.
    pub(crate) fn contains(&self, entity: EntityId) -> bool {
        match entity {
            EntityId::Atom(id) => self.contains_atom(id),
            EntityId::Pseudoatom(id) => self.contains_pseudoatom(id),
            EntityId::Bond(id) => self.contains_bond(id),
            EntityId::Substituent(id) => self.contains_substituent(id),
            EntityId::Molecule(id) => self.contains_molecule(id),
        }
    }

    // Methods to add entities

    /// Adds an atom to the map.
    pub(crate) fn add_atom(&mut self, element: Element) -> AtomId {
        self.atoms.insert(Atom::new(element))
    }

    /// Adds a pseudoatom to the map.
    pub(crate) fn add_pseudoatom(&mut self, symbol: &str) -> PseudoatomId {
        self.pseudoatoms.insert(Pseudoatom::new(symbol.to_owned()))
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
            match partner {
                BondableId::Atom(id) => self.atoms.get_mut(id).unwrap().bonds.push(bond_id),
                BondableId::Pseudoatom(id) => {
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

    // Methods to add entities to collections

    /// Adds the atom, pseudoatom, or bond to the substituent.
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
        if sub.members.contains(&fundamental) {
            sub.members.push(fundamental);
            false
        } else {
            true
        }
    }

    /// Adds the atom, pseudoatom, or bond to the molecule.
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
        if mol.members.contains(&fundamental) {
            mol.members.push(fundamental);
            false
        } else {
            true
        }
    }

    // Methods to remove entities from collections

    /// Removes the atom, pseudoatom, or bond from the substituent.
    ///
    /// Returns whether the fundamental was a member of the substituent.
    ///
    /// If the fundamental is an atomlike and is the centre of the substituent,
    /// the centre is adjusted accordingly; if it is the lone centre, the
    /// substituent becomes centreless. If it is one of two centres,
    /// however, the centre remains `SubstituentCentre::Multiple` rather than
    /// becoming `Single`.
    ///
    /// The substituent continues to exist even if empty.
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
                if let Some(atomlike) = match fundamental {
                    FundamentalId::Bond(_) => None,
                    FundamentalId::Atom(id) => Some(id.into()),
                    FundamentalId::Pseudoatom(id) => Some(id.into()),
                } && let Some(index) = atomlikes.iter().position(|x| *x == atomlike)
                {
                    // We want to preserve order
                    atomlikes.remove(index);
                }
            }
        }
        true
    }

    /// Removes the atom, pseudoatom, or bond from the molecule.
    ///
    /// Returns whether the fundamental was a member of the molecule.
    ///
    /// The molecule continues to exist even if empty.
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
        if let Some(index) = mol.members.iter().position(|x| *x == fundamental) {
            mol.members.swap_remove(index);
            true
        } else {
            false
        }
    }

    // Methods to remove entities entirely

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
                match bonding_partner {
                    BondableId::Atom(atom_id) => {
                        let mut atom = self
                            .atoms
                            .get_mut(atom_id)
                            .expect("Bonds are always removed before their bonding partners");
                        let pos = atom.bonds.iter().position(|x| *x == id).expect(
                            "Bond should be listed in the bonding partner's bonds until deletion",
                        );
                        atom.bonds.remove(pos);
                    }
                    BondableId::Pseudoatom(pseudoatom_id) => {
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
            match member {
                FundamentalId::Atom(id) => {
                    self.delete_atom(id);
                }
                FundamentalId::Pseudoatom(id) => {
                    self.delete_pseudoatom(id);
                }
                FundamentalId::Bond(id) => {
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
            match member {
                FundamentalId::Atom(id) => {
                    self.delete_atom(id);
                }
                FundamentalId::Pseudoatom(id) => {
                    self.delete_pseudoatom(id);
                }
                FundamentalId::Bond(id) => {
                    self.delete_bond(id);
                }
            }
        }
        self.molecules.remove(id).is_some()
    }

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
