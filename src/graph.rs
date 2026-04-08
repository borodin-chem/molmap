// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use slotmap::{SlotMap, basic::Iter};

use std::{fmt::Debug, hash::RandomState};

use crate::{Element, MolMap, bond::BondType, entities::*, fragment::FragmentCentre, id::*};

/// An arena-like data structure to represent a set of chemical entities,
/// their properties, and the relationships between them, as a molecular graph.
/// 
/// This forms the core of all `MolMap` types, but is not public.
/// [`MolMap0`] is the `MolMap` type that provides a molecular graph for users.
#[derive(Debug, Default)]
pub(crate) struct MolGraph {
    pub(crate) atoms: SlotMap<AtomId, Atom>,
    pub(crate) pseudoatoms: SlotMap<PseudoatomId, Pseudoatom>,
    pub(crate) bonds: SlotMap<BondId, Bond>,
    pub(crate) fragments: SlotMap<FragmentId, Fragment>,
    pub(crate) molecules: SlotMap<MoleculeId, Molecule>,
}

impl MolGraph {
    pub(crate) fn new() -> Self {
        Self {
            atoms: SlotMap::with_key(),
            pseudoatoms: SlotMap::with_key(),
            bonds: SlotMap::with_key(),
            fragments: SlotMap::with_key(),
            molecules: SlotMap::with_key(),
        }
    }
    
    pub(crate) fn with_capacity(n: usize) -> Self {
        todo!()
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

    /// Returns an iterator over all the IDs of all fragments in the map.
    pub(crate) fn fragment_ids(&'_ self) -> impl Iterator<Item = FragmentId> + '_ {
        self.fragments.keys()
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
    pub(crate) fn contains_fragment(&self, id: FragmentId) -> bool {
        self.fragments.contains_key(id)
    }

    /// Checks if the given ID is valid.
    pub(crate) fn contains_molecule(&self, id: MoleculeId) -> bool {
        self.molecules.contains_key(id)
    }

    /// Checks if the given enum wraps a valid ID.
    pub(crate) fn contains_atomlike(&self, atomlike: Atomlike) -> bool {
        match atomlike {
            Atomlike::Atom(id) => self.contains_atom(id),
            Atomlike::Pseudoatom(id) => self.contains_pseudoatom(id),
        }
    }

    /// Checks if the given enum wraps a valid ID.
    pub(crate) fn contains_fundamental(&self, fundamental: Fundamental) -> bool {
        match fundamental {
            Fundamental::Atom(id) => self.contains_atom(id),
            Fundamental::Pseudoatom(id) => self.contains_pseudoatom(id),
            Fundamental::Bond(id) => self.contains_bond(id),
        }
    }

    /// Checks if the given enum wraps a valid ID.
    pub(crate) fn contains_bondable(&self, bondable: Bondable) -> bool {
        match bondable {
            Bondable::Atom(id) => self.contains_atom(id),
            Bondable::Pseudoatom(id) => self.contains_pseudoatom(id),
            Bondable::Fragment(id) => self.contains_fragment(id),
        }
    }

    /// Checks if the map contains the entity with the wrapped ID.
    pub(crate) fn contains_entity(&self, entity: Entity) -> bool {
        match entity {
            Entity::Atom(id) => self.contains_atom(id),
            Entity::Pseudoatom(id) => self.contains_pseudoatom(id),
            Entity::Bond(id) => self.contains_bond(id),
            Entity::Fragment(id) => self.contains_fragment(id),
            Entity::Molecule(id) => self.contains_molecule(id),
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
    /// Fails if either of `start` and `end` are invalid.
    pub(crate) fn add_bond(&mut self, start: Bondable, end: Bondable) -> Result<BondId, IdError> {
        // Converting the bondables into `BondingPartner`s checks the IDs at the same time
        let start = self.convert_bondable(start)?;
        let end = self.convert_bondable(end)?;
        let bond_id = self
            .bonds
            .insert(Bond::new(BondType::Covalent, 1.0, start, end));
        for partner in [start, end] {
            match partner {
                BondingPartner::Atom(id) => self
                    .atoms
                    .get_mut(id)
                    .expect("Already checked")
                    .bonds
                    .push(bond_id),
                BondingPartner::Pseudoatom(id) => self
                    .pseudoatoms
                    .get_mut(id)
                    .expect("Already checked")
                    .bonds
                    .push(bond_id),
                BondingPartner::AmbiguouslyBondingFragment(id) => {
                    let FragmentCentre::Ambiguous(bonds) =
                        &mut self.fragments.get_mut(id).expect("Already checked").centre
                    else {
                        unreachable!("Already know it's ambiguous")
                    };
                    bonds.push(bond_id);
                }
            }
        }
        Ok(bond_id)
    }

    /// Adds a fragment to the map with a single initial atom.
    ///
    /// Fails if `centre` is invalid.
    pub(crate) fn add_fragment(&mut self, centre: Atomlike) -> Result<FragmentId, IdError> {
        if !self.contains_atomlike(centre) {
            return Err(IdError);
        }
        Ok(self.fragments.insert(Fragment {
            centre: FragmentCentre::Single(centre),
            members: vec![centre.into()],
        }))
    }

    /// Adds an empty molecule to the map.
    pub(crate) fn add_molecule(&mut self) -> MoleculeId {
        self.molecules.insert(Molecule::new())
    }

    // Methods to add entities to collections

    // Methods to remove entities from collections

    /// Removes the atom, pseudoatom, or bond from the fragment.
    ///
    /// Fails if `fragment` is invalid.
    /// This is otherwise infallible – if the entity is not a member of the fragment,
    /// nothing happens.
    pub(crate) fn remove_from_fragment(
        &mut self,
        fragment: FragmentId,
        fundamental: Fundamental,
    ) -> Result<(), IdError> {
        let frag = self.fragments.get_mut(fragment).ok_or(IdError)?;
        if let Some(index) = frag.members.iter().position(|x| *x == fundamental) {
            frag.members.swap_remove(index);
        }
        // If an atom or bond, potentially have to change the centres of the fragment accordingly
        match &mut frag.centre {
            FragmentCentre::Ambiguous(_) => (),
            FragmentCentre::Single(atomlike) => {
                if Fundamental::from(*atomlike) == fundamental {
                    frag.centre = FragmentCentre::default()
                }
            }
            FragmentCentre::Multiple(atomlikes) => {
                if let Some(atomlike) = match fundamental {
                    Fundamental::Bond(_) => None,
                    Fundamental::Atom(id) => Some(id.into()),
                    Fundamental::Pseudoatom(id) => Some(id.into()),
                } && let Some(index) = atomlikes.iter().position(|x| *x == atomlike)
                {
                    // We want to preserve order
                    atomlikes.remove(index);
                }
            }
        }
        Ok(())
    }

    /// Removes the atom, pseudoatom, or bond from the molecule.
    ///
    /// Fails if `molecule` is invalid.
    /// This is otherwise infallible – if the entity is not a member of the molecule,
    /// nothing happens.
    pub(crate) fn remove_from_molecule(
        &mut self,
        molecule: MoleculeId,
        fundamental: Fundamental,
    ) -> Result<(), IdError> {
        let mol = self.molecules.get_mut(molecule).ok_or(IdError)?;
        if let Some(index) = mol.members.iter().position(|x| *x == fundamental) {
            mol.members.swap_remove(index);
        }
        Ok(())
    }

    // Methods to remove collections but retain their members

    // Methods to remove entities entirely

    /// Removes an atom from the map, as well as any bonds to it.
    ///
    /// This is infallible – if the atom is not in the map, nothing happens.
    pub(crate) fn remove_atom(&mut self, id: AtomId) {
        if !self.contains_atom(id) {
            return;
        }
        // Make sure we always remove bonds first
        let bonds = self.atoms.get(id).unwrap().bonds.clone();
        for bond_id in bonds {
            self.remove_bond(bond_id);
        }
        // Remove from any collections
        if let Some(frag_id) = self.parent_fragment(id.into()) {
            self.remove_from_fragment(frag_id, id.into()).unwrap()
        }
        if let Some(mol_id) = self.parent_molecule(id.into()) {
            self.remove_from_molecule(mol_id, id.into()).unwrap()
        }
        // Now we can safely remove the atom itself without leaving dangling bonds
        self.atoms.remove(id);
    }

    /// Removes a pseudoatom from the map, as well as any bonds to it.
    ///
    /// This is infallible – if the pseudoatom is not in the map, nothing happens.
    pub(crate) fn remove_pseudoatom(&mut self, id: PseudoatomId) {
        if !self.contains_pseudoatom(id) {
            return;
        }
        // Make sure we always remove bonds first
        let bonds = self.pseudoatoms.get(id).unwrap().bonds.clone();
        for bond_id in bonds {
            self.remove_bond(bond_id);
        }
        // Remove from any collections
        if let Some(frag_id) = self.parent_fragment(id.into()) {
            self.remove_from_fragment(frag_id, id.into()).unwrap()
        }
        if let Some(mol_id) = self.parent_molecule(id.into()) {
            self.remove_from_molecule(mol_id, id.into()).unwrap()
        }
        // Now we can safely remove the pseudoatom itself without leaving dangling bonds
        self.pseudoatoms.remove(id);
    }

    /// Removes a bond from the map (but not its bonding partners).
    ///
    /// This is infallible – if the bond is not in the map, nothing happens.
    pub(crate) fn remove_bond(&mut self, id: BondId) {
        if let Some(bond) = self.bonds.remove(id) {
            for bonding_partner in [bond.start, bond.end] {
                match bonding_partner {
                    BondingPartner::Atom(atom_id) => {
                        let mut atom = self
                            .atoms
                            .get_mut(atom_id)
                            .expect("Bonds are always removed before their bonding partners");
                        let pos = atom.bonds.iter().position(|x| *x == id).expect(
                            "Bond should be listed in the bonding partner's bonds until deletion",
                        );
                        atom.bonds.remove(pos);
                    }
                    BondingPartner::Pseudoatom(pseudoatom_id) => {
                        let mut pseudoatom = self
                            .pseudoatoms
                            .get_mut(pseudoatom_id)
                            .expect("Bonds are always removed before their bonding partners");
                        let pos = pseudoatom.bonds.iter().position(|x| *x == id).expect(
                            "Bond should be listed in the bonding partner's bonds until deletion",
                        );
                        pseudoatom.bonds.remove(pos);
                    }
                    BondingPartner::AmbiguouslyBondingFragment(fragment_id) => todo!(),
                }
            }
            // Remove from any collections
            if let Some(frag_id) = self.parent_fragment(id.into()) {
                self.remove_from_fragment(frag_id, id.into()).unwrap()
            }
            if let Some(mol_id) = self.parent_molecule(id.into()) {
                self.remove_from_molecule(mol_id, id.into()).unwrap()
            }
        }
    }

    /// Removes a fragment from the map, as well as all of its members.
    ///
    /// This is infallible – if the fragment is not in the map, nothing happens.
    pub(crate) fn remove_fragment(&mut self, id: FragmentId) {
        todo!("Remove members first");
        self.fragments.remove(id);
    }

    /// Removes a molecule from the map, as well as all of its members.
    ///
    /// This is infallible – if the molecule is not in the map, nothing happens.
    pub(crate) fn remove_molecule(&mut self, id: MoleculeId) {
        todo!("Remove members first");
        self.molecules.remove(id);
    }
}

// Private methods
impl MolGraph {
    /// Gets the actual bonding partner that a `Bondable` refers to, while also validating the ID.
    ///
    /// Fragments generally form bonds from a central atom or pseudoatom, but they might have no
    /// specified centre, or they might have multiple centres.
    /// In the first case, the bond goes to the fragment as a whole; in the second case the first
    /// centre in `centres` is used for the new bond.
    fn convert_bondable(&self, bondable: Bondable) -> Result<BondingPartner, IdError> {
        match bondable {
            Bondable::Atom(id) => self
                .contains_atom(id)
                .then_some(BondingPartner::Atom(id))
                .ok_or(IdError),
            Bondable::Pseudoatom(id) => self
                .contains_pseudoatom(id)
                .then_some(BondingPartner::Pseudoatom(id))
                .ok_or(IdError),
            Bondable::Fragment(id) => {
                // Get the fragment's data while also checking the ID
                let fragment = self.fragments.get(id).ok_or(IdError)?;
                // Use the first centre if any specified, the entire fragment if not
                match &fragment.centre {
                    fragment::FragmentCentre::Ambiguous(_) => {
                        Ok(BondingPartner::AmbiguouslyBondingFragment(id))
                    }
                    fragment::FragmentCentre::Single(centre) => Ok((*centre).into()),
                    fragment::FragmentCentre::Multiple(centres) => Ok((*centres
                        .first()
                        .expect("Will always have at least one centre"))
                    .into()),
                }
            }
        }
    }

    /// Determines the fragment that contains the atom, pseudoatom, or bond, if any.
    fn parent_fragment(&self, fundamental: Fundamental) -> Option<FragmentId> {
        for (fragment_id, fragment) in self.fragments.iter() {
            if fragment.members.contains(&fundamental) {
                return Some(fragment_id);
            }
        }
        None
    }

    /// Determines the molecule that contains the atom, pseudoatom, or bond, if any.
    fn parent_molecule(&self, fundamental: Fundamental) -> Option<MoleculeId> {
        for (mol_id, mol) in self.molecules.iter() {
            if mol.members.contains(&fundamental) {
                return Some(mol_id);
            }
        }
        None
    }
}
