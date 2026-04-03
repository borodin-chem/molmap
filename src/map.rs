// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use slotmap::{SlotMap, basic::Iter};

use std::{fmt::Debug, hash::RandomState};

use crate::{Element, bond::BondType, entities::*, fragment::FragmentCentre, id::*};

/// An extensible arena-like data structure to represent a set of chemical entities,
/// their properties, and the relationships between them, as a molecular graph.
#[derive(Debug, Default)]
pub struct MolMap<E: MolMapExt> {
    pub(crate) atoms: SlotMap<AtomId, Atom>,
    pub(crate) pseudoatoms: SlotMap<PseudoatomId, Pseudoatom>,
    pub(crate) bonds: SlotMap<BondId, Bond>,
    pub(crate) fragments: SlotMap<FragmentId, Fragment>,
    pub(crate) molecules: SlotMap<MoleculeId, Molecule>,
    //pub(crate) objects: SlotMap<ObjectId, Object>,
    pub(crate) extension: E,
}

pub trait MolMapExt: Debug + Default {
    /// Creates an empty `MolMap`.
    fn new() -> Self;

    /// Creates a new `MolMap` with capacity for approximately `n` atoms.
    fn with_capacity(n: usize) -> Self;
}

impl<E: MolMapExt> MolMap<E> {
    /// Creates an empty `MolMap`.
    ///
    /// As the constituent `SlotMap`s are created with an initial capacity of 0, reallocations will
    /// occur frequently if many entities are subsequently inserted.
    /// If you have an idea of approximately how large the `MolMap` needs to be, it is recommended
    /// to use `MolMap.with_capacity()` instead.
    pub fn new() -> Self {
        Self {
            atoms: SlotMap::with_key(),
            pseudoatoms: SlotMap::with_key(),
            bonds: SlotMap::with_key(),
            fragments: SlotMap::with_key(),
            molecules: SlotMap::with_key(),
            //objects: SlotMap::with_key(),
            extension: E::new(),
        }
    }
}

//impl<E: MolMapExt> MolMap<E> {
//    /// Creates a `MolMap` with capacity for approximately `n` molecules and `30 * n` atoms.
//    ///
//    /// The required number of entities of each type is guessed based on an assumption that each
//    /// molecule is a small organic molecule containing approximately 10 to 12 carbon atoms.
//    ///
//    /// The constituent `SlotMap`s are created with initial capacities for the following:
//    /// - `n` molecules
//    /// - `10 * n` fragments
//    /// - `30 * n` atoms
//    /// - `5 * n` pseudoatoms
//    /// - `30 * n` bonds
//    pub fn with_capacity(n: usize) -> Self {
//        Self {
//            atoms: SlotMap::with_capacity_and_key(30 * n),
//            pseudoatoms: SlotMap::with_capacity_and_key(5 * n),
//            bonds: SlotMap::with_capacity_and_key(30 * n),
//            fragments: SlotMap::with_capacity_and_key(10 * n),
//            molecules: SlotMap::with_capacity_and_key(n),
//            //objects: SlotMap::with_capacity_and_key(2 * n),
//            extension: E::default(),
//        }
//    }
//}

/// Methods that apply to all MolMaps, regardless of extension
impl<E: MolMapExt> MolMap<E> {
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
}

/// Methods where each concrete MolMap type may need to add extra logic to deal with the data
/// contained in the extension
impl<E: MolMapExt> MolMap<E> {
    // Methods to add entities

    /// Adds an atom to the map.
    pub(crate) fn _add_atom(&mut self, element: Element) -> AtomId {
        self.atoms.insert(Atom::new(element))
    }

    /// Adds a pseudoatom to the map.
    pub(crate) fn _add_pseudoatom(&mut self, symbol: &str) -> PseudoatomId {
        self.pseudoatoms.insert(Pseudoatom::new(symbol.to_owned()))
    }

    /// Creates a new (single covalent) bond between two bondable entities.
    ///
    /// Fails if either of `start` and `end` are invalid.
    pub(crate) fn _add_bond(&mut self, start: Bondable, end: Bondable) -> Result<BondId, IdError> {
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
    pub(crate) fn _add_fragment(&mut self, centre: Atomlike) -> Result<FragmentId, IdError> {
        if !self.contains_atomlike(centre) {
            return Err(IdError);
        }
        Ok(self.fragments.insert(Fragment {
            centre: FragmentCentre::Single(centre),
            members: vec![centre.into()],
        }))
    }

    /// Adds an empty molecule to the map.
    pub(crate) fn _add_molecule(&mut self) -> MoleculeId {
        self.molecules.insert(Molecule::new())
    }

    // Methods to add entities to collections

    // Methods to remove entities from collections

    /// Removes the atom, pseudoatom, or bond from the fragment.
    ///
    /// This in infallible – if the entity is not a member of the fragment, nothing happens.
    pub(crate) fn _remove_from_fragment(
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
    /// This in infallible – if the entity is not a member of the molecule, nothing happens.
    pub(crate) fn _remove_from_molecule(
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
    pub(crate) fn _remove_atom(&mut self, id: AtomId) {
        if !self.contains_atom(id) {
            return;
        }
        // Make sure we always remove bonds first
        let bonds = self.atoms.get(id).unwrap().bonds.clone();
        for bond_id in bonds {
            self._remove_bond(bond_id);
        }
        // Remove from any collections
        if let Some(frag_id) = self.parent_fragment(id.into()) {
            self._remove_from_fragment(frag_id, id.into()).unwrap()
        }
        if let Some(mol_id) = self.parent_molecule(id.into()) {
            self._remove_from_molecule(mol_id, id.into()).unwrap()
        }
        // Now we can safely remove the atom itself without leaving dangling bonds
        self.atoms.remove(id);
    }

    /// Removes a pseudoatom from the map, as well as any bonds to it.
    ///
    /// This is infallible – if the pseudoatom is not in the map, nothing happens.
    pub(crate) fn _remove_pseudoatom(&mut self, id: PseudoatomId) {
        if !self.contains_pseudoatom(id) {
            return;
        }
        // Make sure we always remove bonds first
        let bonds = self.pseudoatoms.get(id).unwrap().bonds.clone();
        for bond_id in bonds {
            self._remove_bond(bond_id);
        }
        // Remove from any collections
        if let Some(frag_id) = self.parent_fragment(id.into()) {
            self._remove_from_fragment(frag_id, id.into()).unwrap()
        }
        if let Some(mol_id) = self.parent_molecule(id.into()) {
            self._remove_from_molecule(mol_id, id.into()).unwrap()
        }
        // Now we can safely remove the pseudoatom itself without leaving dangling bonds
        self.pseudoatoms.remove(id);
    }

    /// Removes a bond from the map (but not its bonding partners).
    ///
    /// This is infallible – if the bond is not in the map, nothing happens.
    pub(crate) fn _remove_bond(&mut self, id: BondId) {
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
                self._remove_from_fragment(frag_id, id.into()).unwrap()
            }
            if let Some(mol_id) = self.parent_molecule(id.into()) {
                self._remove_from_molecule(mol_id, id.into()).unwrap()
            }
        }
    }

    /// Removes a fragment from the map, as well as all of its members.
    ///
    /// This is infallible – if the fragment is not in the map, nothing happens.
    pub(crate) fn _remove_fragment(&mut self, id: FragmentId) {
        todo!("Remove members first");
        self.fragments.remove(id);
    }

    /// Removes a molecule from the map, as well as all of its members.
    ///
    /// This is infallible – if the molecule is not in the map, nothing happens.
    pub(crate) fn _remove_molecule(&mut self, id: MoleculeId) {
        todo!("Remove members first");
        self.molecules.remove(id);
    }
}

// Private methods
impl<E: MolMapExt> MolMap<E> {
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

impl MolMapExt for () {
    fn new() -> Self {}

    fn with_capacity(n: usize) -> Self {}
}

/// A convenient alias for `MolMap<()>`, a `MolMap` that represents just a molecular graph.
pub type MolMap0 = MolMap<()>;

// Implement the addition/removal methods for the basic graph version of MolMap
impl MolMap0 {
    /// Adds an atom to the map.
    pub fn add_atom(&mut self, element: Element) -> AtomId {
        self._add_atom(element)
    }

    /// Adds a pseudoatom to the map.
    pub fn add_pseudoatom(&mut self, symbol: &str) -> PseudoatomId {
        self._add_pseudoatom(symbol)
    }

    /// Creates a new (single covalent) bond between two bondable entities.
    ///
    /// Fails if either of `start` and `end` are invalid.
    pub fn add_bond(&mut self, start: Bondable, end: Bondable) -> Result<BondId, IdError> {
        self._add_bond(start, end)
    }

    /// Adds a fragment to the map with a single initial atom.
    ///
    /// Fails if `centre` is invalid.
    pub fn add_fragment(&mut self, centre: Atomlike) -> Result<FragmentId, IdError> {
        self._add_fragment(centre)
    }

    /// Adds an empty molecule to the map.
    pub fn add_molecule(&mut self) -> MoleculeId {
        self._add_molecule()
    }

    /// Removes an atom from the map.
    pub fn remove_atom(&mut self, id: AtomId) {
        self._remove_atom(id);
    }

    /// Removes a pseudoatom from the map.
    pub fn remove_pseudoatom(&mut self, id: PseudoatomId) {
        self._remove_pseudoatom(id);
    }

    /// Removes a bond from the map.
    pub fn remove_bond(&mut self, id: BondId) {
        self._remove_bond(id);
    }

    /// Removes a fragment from the map.
    pub fn remove_fragment(&mut self, id: FragmentId) {
        self._remove_fragment(id);
    }

    /// Removes a molecule from the map.
    pub fn remove_molecule(&mut self, id: MoleculeId) {
        self._remove_molecule(id);
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
    /// - two fragments (CH3, OH) (n.b. not yet implemented)
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
        // TODO fragments
        mm
    }

    #[test]
    fn add_atom() {
        let mut mm = MolMap0::new();
        assert!(mm.atoms.is_empty());
        let h1 = mm.add_atom(Element::H);
        assert_eq!(mm.atoms.len(), 1);
        let c1 = mm.add_atom(Element::C);
        assert_eq!(mm.atoms.len(), 2);
        // Check the atoms can be accessed by their ID, and that the elements are correct
        assert_eq!(mm.atoms.get(h1).unwrap().element, Element::H);
        assert_eq!(mm.atoms.get(c1).unwrap().element, Element::C);
        // Check that the bond arrays are created empty
        assert!(mm.atoms.get(h1).unwrap().bonds.is_empty());
    }

    #[test]
    fn add_pseudoatom() {
        let mut mm = MolMap0::new();
        assert!(mm.pseudoatoms.is_empty());
        let r1 = mm.add_pseudoatom("R");
        assert_eq!(mm.pseudoatoms.len(), 1);
        // Check the pseudoatom can be accessed by its ID, and that the symbol is correct
        assert_eq!(mm.pseudoatoms.get(r1).unwrap().symbol, "R");
        // Check that the bond arrays are created empty
        assert!(mm.pseudoatoms.get(r1).unwrap().bonds.is_empty());
    }

    #[test]
    fn remove_atom() {
        let mut mm = MolMap0::new();
        let h1 = mm.add_atom(Element::H);
        let c1 = mm.add_atom(Element::C);
        assert_eq!(mm.atoms.len(), 2);
        mm.remove_atom(h1);
        assert_eq!(mm.atoms.len(), 1);
    }

    #[test]
    fn remove_pseudoatom() {
        let mut mm = MolMap0::new();
        let r1 = mm.add_pseudoatom("R");
        assert_eq!(mm.pseudoatoms.len(), 1);
        mm.remove_pseudoatom(r1);
        assert!(mm.pseudoatoms.is_empty());
    }

    #[test]
    fn add_bond_between_atoms() {
        let mut mm = MolMap0::new();
        assert!(mm.bonds.is_empty());
        let h1 = mm.add_atom(Element::H);
        let h2 = mm.add_atom(Element::H);
        let b1 = mm.add_bond(h1.into(), h2.into()).unwrap();
        assert!(mm.bonds.contains_key(b1));
        assert!(mm.atoms.get(h1).unwrap().bonds.contains(&b1));
        assert!(mm.atoms.get(h2).unwrap().bonds.contains(&b1));
        assert_eq!(mm.bonds.get(b1).unwrap().start, h1.into());
        assert_eq!(mm.bonds.get(b1).unwrap().end, h2.into());
    }

    #[test]
    fn remove_bond_between_atoms() {
        let mut mm = MolMap0::new();
        let h1 = mm.add_atom(Element::H);
        let h2 = mm.add_atom(Element::H);
        let b1 = mm.add_bond(h1.into(), h2.into()).unwrap();
        assert!(mm.bonds.contains_key(b1));
        assert!(mm.atoms.get(h1).unwrap().bonds.contains(&b1));
        assert!(mm.atoms.get(h2).unwrap().bonds.contains(&b1));
        assert_eq!(mm.bonds.get(b1).unwrap().start, h1.into());
        assert_eq!(mm.bonds.get(b1).unwrap().end, h2.into());
        mm.remove_bond(b1);
        assert!(!mm.bonds.contains_key(b1));
        assert!(!mm.atoms.get(h1).unwrap().bonds.contains(&b1));
        assert!(!mm.atoms.get(h2).unwrap().bonds.contains(&b1));
    }
}
