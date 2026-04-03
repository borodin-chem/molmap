// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use slotmap::{SlotMap, basic::Iter};

use std::hash::RandomState;

use crate::{Element, bond::BondType, entities::*, id::*};

/// An extensible arena-like data structure to represent a set of chemical entities,
/// their properties, and the relationships between them, as a molecular graph.
#[derive(Debug)]
pub struct MolMap<Extension> {
    pub(crate) bonds: SlotMap<BondId, Bond>,
    pub(crate) atoms: SlotMap<AtomId, Atom>,
    pub(crate) pseudoatoms: SlotMap<PseudoatomId, Pseudoatom>,
    pub(crate) fragments: SlotMap<FragmentId, Fragment>,
    pub(crate) molecules: SlotMap<MoleculeId, Molecule>,
    //pub(crate) objects: SlotMap<ObjectId, Object>,
    pub(crate) extension: Extension,
}

/// A convenient alias for `MolMap<()>`, a `MolMap` that represents just a molecular graph.
pub type MolGraph = MolMap<()>;

impl<E: Default> Default for MolMap<E> {
    /// Creates a `MolMap` with a modest capacity of approximately 5 molecules and 150 atoms.
    /// 
    /// Equivalent to `MolMap::with_capacity(5)`
    fn default() -> Self {
        Self::with_capacity(5)
    }
}

impl<E: Default> MolMap<E> {

    /// Creates an empty `MolMap`.
    /// 
    /// As the constituent `SlotMap`s are created with an initial capacity of 0, reallocations will
    /// occur frequently if many entities are subsequently inserted.
    /// If you have an idea of approximately how large the `MolMap` needs to be, it is recommended
    /// to use `MolMap.with_capacity()` instead.
    pub fn new() -> Self {
        Self {
            bonds: SlotMap::with_key(),
            atoms: SlotMap::with_key(),
            pseudoatoms: SlotMap::with_key(),
            fragments: SlotMap::with_key(),
            molecules: SlotMap::with_key(),
            //objects: SlotMap::with_key(),
            extension: E::default(),
        }
    }

    /// Creates a `MolMap` with capacity for approximately `n` molecules and `30 * n` atoms.
    /// 
    /// The required number of entities of each type is guessed based on an assumption that each
    /// molecule is a small organic molecule containing approximately 10 to 12 carbon atoms.
    /// 
    /// The constituent `SlotMap`s are created with initial capacities for the following:
    /// - `n` molecules
    /// - `10 * n` fragments
    /// - `30 * n` atoms
    /// - `5 * n` pseudoatoms
    /// - `30 * n` bonds
    pub fn with_capacity(n: usize) -> Self {
        Self {
            bonds: SlotMap::with_capacity_and_key(30 * n),
            atoms: SlotMap::with_capacity_and_key(30 * n),
            pseudoatoms: SlotMap::with_capacity_and_key(5 * n),
            fragments: SlotMap::with_capacity_and_key(10 * n),
            molecules: SlotMap::with_capacity_and_key(n),
            //objects: SlotMap::with_capacity_and_key(2 * n),
            extension: E::default(),
        }
    }
}

impl<E> MolMap<E> {
    // Getters
    // One method per entity type for:
    // - getting a view
    // - getting a mutable view
    // - iterating over IDs
    // - validating an ID

    /// Constructs an immutable `AtomView` for the given atom,
    /// returning `None` if the ID is invalid.
    pub fn atom(&'_ self, id: AtomId) -> Option<AtomView<'_, E>> {
        self.atoms
            .contains_key(id)
            .then_some(AtomView { molmap: self, id })
    }

    /// Constructs a mutable `AtomViewMut` for the given atom, returning `None` if the ID is
    /// invalid.
    pub fn atom_mut(&'_ mut self, id: AtomId) -> Option<AtomViewMut<'_, E>> {
        self.atoms
            .contains_key(id)
            .then_some(AtomViewMut { molmap: self, id })
    }

    /// Returns an iterator over all the IDs of all atoms in the map.
    pub fn atom_ids(&'_ self) -> impl Iterator<Item = AtomId> + '_ {
        self.atoms.keys()
    }

    /// Constructs an immutable `PseudoatomView` for the given pseudoatom, returning `None` if the
    /// ID is invalid.
    pub fn pseudoatom(&'_ self, id: PseudoatomId) -> Option<PseudoatomView<'_, E>> {
        self.pseudoatoms
            .contains_key(id)
            .then_some(PseudoatomView { molmap: self, id })
    }

    /// Constructs a mutable `PseudoatomViewMut` for the given pseudoatom, returning `None` if the
    /// ID is invalid.
    pub fn pseudoatom_mut(&'_ mut self, id: PseudoatomId) -> Option<PseudoatomViewMut<'_, E>> {
        self.pseudoatoms
            .contains_key(id)
            .then_some(PseudoatomViewMut { molmap: self, id })
    }

    /// Returns an iterator over all the IDs of all pseudoatoms in the map.
    pub fn pseudoatom_ids(&'_ self) -> impl Iterator<Item = PseudoatomId> + '_ {
        self.pseudoatoms.keys()
    }

    /// Constructs an immutable `BondView` for the given bond, returning `None` if the ID is
    /// invalid.
    pub fn bond(&'_ self, id: BondId) -> Option<BondView<'_, E>> {
        self.bonds
            .contains_key(id)
            .then_some(BondView { molmap: self, id })
    }

    /// Constructs a mutable `BondViewMut` for the given bond, returning `None` if the ID is
    /// invalid.
    pub fn bond_mut(&'_ mut self, id: BondId) -> Option<BondViewMut<'_, E>> {
        self.bonds
            .contains_key(id)
            .then_some(BondViewMut { molmap: self, id })
    }

    /// Returns an iterator over all the IDs of all bonds in the map.
    pub fn bond_ids(&'_ self) -> impl Iterator<Item = BondId> + '_ {
        self.bonds.keys()
    }

    /// Constructs an immutable `FragmentView` for the given fragment, returning `None` if the ID is
    /// invalid.
    pub fn fragment(&'_ self, id: FragmentId) -> Option<FragmentView<'_, E>> {
        self.fragments
            .contains_key(id)
            .then_some(FragmentView { molmap: self, id })
    }

    /// Constructs a mutable `FragmentViewMut` for the given fragment, returning `None` if the ID is
    /// invalid.
    pub fn fragment_mut(&'_ mut self, id: FragmentId) -> Option<FragmentViewMut<'_, E>> {
        self.fragments
            .contains_key(id)
            .then_some(FragmentViewMut { molmap: self, id })
    }

    /// Returns an iterator over all the IDs of all fragments in the map.
    pub fn fragment_ids(&'_ self) -> impl Iterator<Item = FragmentId> + '_ {
        self.fragments.keys()
    }

    /// Constructs an immutable `MoleculeView` for the given molecule, returning `None` if the ID is
    /// invalid.
    pub fn molecule(&'_ self, id: MoleculeId) -> Option<MoleculeView<'_, E>> {
        self.molecules
            .contains_key(id)
            .then_some(MoleculeView { molmap: self, id })
    }

    /// Constructs a mutable `MoleculeViewMut` for the given molecule, returning `None` if the ID is
    /// invalid.
    pub fn molecule_mut(&'_ mut self, id: MoleculeId) -> Option<MoleculeViewMut<'_, E>> {
        self.molecules
            .contains_key(id)
            .then_some(MoleculeViewMut { molmap: self, id })
    }

    /// Returns an iterator over all the IDs of all molecules in the map.
    pub fn molecule_ids(&'_ self) -> impl Iterator<Item = MoleculeId> + '_ {
        self.molecules.keys()
    }

    // Methods to check the validity of IDs and entity enums

    /// Checks if the given ID is valid.
    fn contains_atom(&self, id: AtomId) -> bool {
        self.atoms.contains_key(id)
    }

    /// Checks if the given ID is valid.
    fn contains_pseudoatom(&self, id: PseudoatomId) -> bool {
        self.pseudoatoms.contains_key(id)
    }

    /// Checks if the given ID is valid.
    fn contains_bond(&self, id: BondId) -> bool {
        self.bonds.contains_key(id)
    }

    /// Checks if the given ID is valid.
    fn contains_fragment(&self, id: FragmentId) -> bool {
        self.fragments.contains_key(id)
    }

    /// Checks if the given ID is valid.
    fn contains_molecule(&self, id: MoleculeId) -> bool {
        self.molecules.contains_key(id)
    }

    /// Checks if the given enum wraps a valid ID.
    fn contains_atomlike(&self, atomlike: Atomlike) -> bool {
        match atomlike {
            Atomlike::Atom(id) => self.contains_atom(id),
            Atomlike::Pseudoatom(id) => self.contains_pseudoatom(id),
        }
    }

    /// Checks if the given enum wraps a valid ID.
    fn contains_fundamental(&self, fundamental: Fundamental) -> bool {
        match fundamental {
            Fundamental::Atom(id) => self.contains_atom(id),
            Fundamental::Pseudoatom(id) => self.contains_pseudoatom(id),
            Fundamental::Bond(id) => self.contains_bond(id),
        }
    }

    /// Checks if the given enum wraps a valid ID.
    fn contains_bondable(&self, bondable: Bondable) -> bool {
        match bondable {
            Bondable::Atom(id) => self.contains_atom(id),
            Bondable::Pseudoatom(id) => self.contains_pseudoatom(id),
            Bondable::Fragment(id) => self.contains_fragment(id),
        }
    }

    /// Checks if the map contains the entity with the wrapped ID.
    fn contains_entity(&self, entity: Entity) -> bool {
        match entity {
            Entity::Atom(id) => self.contains_atom(id),
            Entity::Pseudoatom(id) => self.contains_pseudoatom(id),
            Entity::Bond(id) => self.contains_bond(id),
            Entity::Fragment(id) => self.contains_fragment(id),
            Entity::Molecule(id) => self.contains_molecule(id),
        }
    }

    // Methods to add entities

    /// Adds an `Atom` to the map.
    pub fn add_atom(&mut self, element: Element) -> AtomId {
        self.atoms.insert(Atom::new(element))
    }

    /// Adds a `Pseudoatom` to the map.
    pub fn add_pseudoatom(&mut self, symbol: &str) -> PseudoatomId {
        self.pseudoatoms.insert(Pseudoatom::new(symbol.to_owned()))
    }

    /// Creates a new (single covalent) `Bond` between two bondable entities.
    ///
    /// Fails if either of `start` and `end` are invalid.
    pub fn create_bond(&mut self, start: Bondable, end: Bondable) -> Result<BondId, IdError> {
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
                BondingPartner::AmbiguouslyBondingFragment(id) => self
                    .fragments
                    .get_mut(id)
                    .expect("Already checked")
                    .bonds
                    .push(bond_id),
            }
        }
        Ok(bond_id)
    }

    /// Adds a `Fragment` to the map with a single initial atom.
    ///
    /// Fails if `centre` is invalid.
    pub fn add_fragment(&mut self, centre: Atomlike) -> Result<FragmentId, IdError> {
        if !self.contains_atomlike(centre) {
            return Err(IdError);
        }
        Ok(self.fragments.insert(Fragment {
            centres: vec![centre],
            members: vec![centre.into()],
            bonds: Vec::new(),
        }))
    }

    /// Adds an empty `Molecule` to the map.
    pub fn add_molecule(&mut self) -> MoleculeId {
        self.molecules.insert(Molecule::new())
    }

    // Methods to add entities to collections

    // Methods to remove entities from collections

    // Methods to remove entities entirely
}

// Private methods
impl<E> MolMap<E> {
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
                match fragment.centres.first() {
                    Some(atomlike) => Ok((*atomlike).into()),
                    None => Ok(BondingPartner::AmbiguouslyBondingFragment(id)),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Element;
    
    use super::*;
    
    #[test]
    fn create_bond_between_atoms() {
        let mut mm = MolMap::<()>::new();
        let h1 = mm.add_atom(Element::H);
        let h2 = mm.add_atom(Element::H);
        let b1 = mm.create_bond(h1.into(), h2.into()).unwrap();
        assert!(mm.atoms.get(h1).unwrap().bonds.contains(&b1));
        assert!(mm.atoms.get(h2).unwrap().bonds.contains(&b1));
        assert!(mm.bonds.get(b1).unwrap().start == h1.into());
        assert!(mm.bonds.get(b1).unwrap().end == h2.into());
    }
}
