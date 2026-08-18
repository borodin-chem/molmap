// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt::Debug;

use crate::{graph::MolGraph, ids::*, views::*};

/// A trait implemented by all `MolMap` types to provide access to their core
/// `MolGraph` without exposing a public interface to it.
///
/// The trait has visibility `pub` to match `MolMap`, but it should not be
/// exposed publicly, hence the re-export in `crate::traits` is `pub(crate)`.
///
/// The use of this trait as a bound for `MolMap` makes it an example of the
/// sealed trait pattern, see
/// https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed
pub trait MolMapCore {
    /// Returns the core molecular graph.
    fn core(&self) -> &MolGraph;

    /// Returns the core molecular graph in mutable form.
    fn core_mut(&mut self) -> &mut MolGraph;
}

/// An arena-like data structure to represent a set of chemical entities, their
/// properties, and the relationships between them, with or without spatial positions.
///
/// This trait provides methods for:
/// 1. obtaining an immutable or mutable view of an entity from its ID e.g.
///    [`MolMap::atom`] and [`MolMap::atom_mut`]
/// 2. verifying an ID e.g. [`MolMap::contains_atom`]
/// 3. iterating over views of all of a given kind of entity e.g. [`MolMap::atoms`]
/// 4. iterating over all IDs of a given kind of entity e.g. [`MolMap::atom_ids`]
///
/// All implementors of `MolMap` should also always provide methods for adding new
/// entities, but the signature for these will vary according to the needs of the
/// concrete map type.
///
/// This trait is sealed and is not intended for implementation outside of `molmap`.
pub trait MolMap: Sized + MolMapCore {
    // ---------------------
    // Required constructors
    // ---------------------

    /// Creates an empty `MolMap`.
    ///
    /// As the constituent `SlotMap`s are created with an initial capacity of 0, reallocations
    /// will occur frequently if many entities are subsequently inserted.
    /// If you have an idea of approximately how large the `MolMap` needs to be, it is
    /// recommended to use `MolMap.with_capacity` or `with_capacities` instead.
    fn new() -> Self;

    /// Creates a new `MolMap` with the specified initial capacities for each kind of entity.
    fn with_capacities(
        atoms: usize,
        pseudoatoms: usize,
        bonds: usize,
        substituents: usize,
        molecules: usize,
    ) -> Self;

    /// Creates a new `MolMap` with initial capacity for approximately `n` atoms.
    ///
    /// In the default implementation, this results in a map with capacity for:
    /// - `n` atoms
    /// - `n / 10` pseudoatoms
    /// - `n` bonds
    /// - `n / 3` substituents
    /// - `(n / 100) + 1` molecules
    fn with_capacity(n: usize) -> Self {
        Self::with_capacities(n, n / 10, n, n / 3, (n / 100) + 1)
    }

    // ------------------
    // ID-related methods
    // ------------------
    // These all just defer to the inner core struct

    /// Returns an iterator over all the IDs of all atoms in the map.
    fn atom_ids(&'_ self) -> impl Iterator<Item = AtomId> + '_ {
        self.core().atom_ids()
    }

    /// Returns an iterator over all the IDs of all pseudoatoms in the map.
    fn pseudoatom_ids(&'_ self) -> impl Iterator<Item = PseudoatomId> + '_ {
        self.core().pseudoatom_ids()
    }

    /// Returns an iterator over all the IDs of all bonds in the map.
    fn bond_ids(&'_ self) -> impl Iterator<Item = BondId> + '_ {
        self.core().bond_ids()
    }

    /// Returns an iterator over all the IDs of all substituents in the map.
    fn substituent_ids(&'_ self) -> impl Iterator<Item = SubstituentId> + '_ {
        self.core().substituent_ids()
    }

    /// Returns an iterator over all the IDs of all molecules in the map.
    fn molecule_ids(&'_ self) -> impl Iterator<Item = MoleculeId> + '_ {
        self.core().molecule_ids()
    }

    /// Checks if the map currently contains the atom with the given ID.
    fn contains_atom(&self, id: AtomId) -> bool {
        self.core().contains_atom(id)
    }

    /// Checks if the map currently contains the pseudoatom with the given ID.
    fn contains_pseudoatom(&self, id: PseudoatomId) -> bool {
        self.core().contains_pseudoatom(id)
    }

    /// Checks if the map currently contains the bond with the given ID.
    fn contains_bond(&self, id: BondId) -> bool {
        self.core().contains_bond(id)
    }

    /// Checks if the map currently contains the substituent with the given ID.
    fn contains_substituent(&self, id: SubstituentId) -> bool {
        self.core().contains_substituent(id)
    }

    /// Checks if the map currently contains the molecule with the given ID.
    fn contains_molecule(&self, id: MoleculeId) -> bool {
        self.core().contains_molecule(id)
    }

    /// Checks if the map currently contains the atomlike with the given ID.
    fn contains_atomlike(&self, id: AtomlikeId) -> bool {
        self.core().contains_atomlike(id)
    }

    /// Checks if the map currently contains the fundamental with the given ID.
    fn contains_fundamental(&self, id: FundamentalId) -> bool {
        self.core().contains_fundamental(id)
    }

    /// Checks if the map currently contains the bondable with the given ID.
    fn contains_bondable(&self, id: BondableId) -> bool {
        self.core().contains_bondable(id)
    }

    /// Checks if the map currently contains the collection with the given ID.
    fn contains_collection(&self, id: CollectionId) -> bool {
        self.core().contains_collection(id)
    }

    /// Checks if the map currently contains the entity with the given ID.
    fn contains(&self, id: EntityId) -> bool {
        self.core().contains(id)
    }

    // -------
    // Getters
    // -------
    // One method per entity kind for:
    // - getting a view
    // - getting a mutable view
    // - iterating over (immutable) views
    // These can be implemented as default methods for all maps, as the views
    // are generic in the concrete map type.

    /// Constructs an immutable `AtomView` of the given atom,
    /// returning `None` if the ID is invalid.
    fn atom(&'_ self, id: AtomId) -> Option<AtomView<'_, Self>> {
        self.core()
            .contains_atom(id)
            .then_some(AtomView { map: self, id })
    }

    /// Constructs a mutable `AtomViewMut` of the given atom, returning `None` if the ID is
    /// invalid.
    fn atom_mut(&'_ mut self, id: AtomId) -> Option<AtomViewMut<'_, Self>> {
        self.core()
            .contains_atom(id)
            .then_some(AtomViewMut { map: self, id })
    }

    /// Returns an iterator over views of all atoms in the map.
    fn atoms(&'_ self) -> AtomViews<'_, Self, impl Iterator<Item = AtomId> + ExactSizeIterator> {
        AtomViews {
            map: self,
            ids: AtomIds(self.core().atoms.keys()),
        }
    }

    /// Constructs an immutable `PseudoatomView` of the given pseudoatom, returning `None` if the
    /// ID is invalid.
    fn pseudoatom(&'_ self, id: PseudoatomId) -> Option<PseudoatomView<'_, Self>> {
        self.core()
            .pseudoatoms
            .contains_key(id)
            .then_some(PseudoatomView { map: self, id })
    }

    /// Constructs a mutable `PseudoatomViewMut` of the given pseudoatom, returning `None` if the
    /// ID is invalid.
    fn pseudoatom_mut(&'_ mut self, id: PseudoatomId) -> Option<PseudoatomViewMut<'_, Self>> {
        self.core()
            .pseudoatoms
            .contains_key(id)
            .then_some(PseudoatomViewMut { map: self, id })
    }

    /// Returns an iterator over views of all pseudoatoms in the map.
    fn pseudoatoms(
        &'_ self,
    ) -> PseudoatomViews<'_, Self, impl Iterator<Item = PseudoatomId> + ExactSizeIterator> {
        PseudoatomViews {
            map: self,
            ids: PseudoatomIds(self.core().pseudoatoms.keys()),
        }
    }

    /// Constructs an immutable `BondView` of the given bond, returning `None` if the ID is
    /// invalid.
    fn bond(&'_ self, id: BondId) -> Option<BondView<'_, Self>> {
        self.core()
            .bonds
            .contains_key(id)
            .then_some(BondView { map: self, id })
    }

    /// Constructs a mutable `BondViewMut` of the given bond, returning `None` if the ID is
    /// invalid.
    fn bond_mut(&'_ mut self, id: BondId) -> Option<BondViewMut<'_, Self>> {
        self.core()
            .bonds
            .contains_key(id)
            .then_some(BondViewMut { map: self, id })
    }

    /// Returns an iterator over views of all bonds in the map.
    fn bonds(&'_ self) -> BondViews<'_, Self, impl Iterator<Item = BondId> + ExactSizeIterator> {
        BondViews {
            map: self,
            ids: BondIds(self.core().bonds.keys()),
        }
    }

    /// Constructs an immutable `SubstituentView` of the given substituent, returning `None` if the ID is
    /// invalid.
    fn substituent(&'_ self, id: SubstituentId) -> Option<SubstituentView<'_, Self>> {
        self.core()
            .substituents
            .contains_key(id)
            .then_some(SubstituentView { map: self, id })
    }

    /// Constructs a mutable `SubstituentViewMut` of the given substituent, returning `None` if the ID is
    /// invalid.
    fn substituent_mut(&'_ mut self, id: SubstituentId) -> Option<SubstituentViewMut<'_, Self>> {
        self.core()
            .substituents
            .contains_key(id)
            .then_some(SubstituentViewMut { map: self, id })
    }

    /// Returns an iterator over views of all substituents in the map.
    fn substituents(
        &'_ self,
    ) -> SubstituentViews<'_, Self, impl Iterator<Item = SubstituentId> + ExactSizeIterator> {
        SubstituentViews {
            map: self,
            ids: SubstituentIds(self.core().substituents.keys()),
        }
    }

    /// Constructs an immutable `MoleculeView` of the given molecule, returning `None` if the ID is
    /// invalid.
    fn molecule(&'_ self, id: MoleculeId) -> Option<MoleculeView<'_, Self>> {
        self.core()
            .molecules
            .contains_key(id)
            .then_some(MoleculeView { map: self, id })
    }

    /// Constructs a mutable `MoleculeViewMut` of the given molecule, returning `None` if the ID is
    /// invalid.
    fn molecule_mut(&'_ mut self, id: MoleculeId) -> Option<MoleculeViewMut<'_, Self>> {
        self.core()
            .molecules
            .contains_key(id)
            .then_some(MoleculeViewMut { map: self, id })
    }

    /// Returns an iterator over views of all molecules in the map.
    fn molecules(
        &'_ self,
    ) -> MoleculeViews<'_, Self, impl Iterator<Item = MoleculeId> + ExactSizeIterator> {
        MoleculeViews {
            map: self,
            ids: MoleculeIds(self.core().molecules.keys()),
        }
    }
}
