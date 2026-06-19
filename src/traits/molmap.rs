// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt::Debug;

use crate::{graph::MolGraph, ids::*, views::*};

/// An arena-like data structure to represent a set of chemical entities, their
/// properties, and the relationships between them, with or without spatial positions.
///
/// This trait provides methods for:
/// 1. obtaining an immutable or mutable view of an entity from its ID e.g.
///    [`MolMap::atom()`] and [`MolMap::atom_mut()`]
/// 2. verifying an ID e.g. [`MolMap::contains_atom()`]
/// 3. iterating over views of all of a given kind of entity e.g. [`MolMap::atoms()`]
/// 4. iterating over all IDs of a given kind of entity e.g. [`MolMap::atom_ids()`]
pub trait MolMap: Debug + Default {
    /// Creates an empty `MolMap`.
    ///
    /// As the constituent `SlotMap`s are created with an initial capacity of 0, reallocations
    /// will occur frequently if many entities are subsequently inserted.
    /// If you have an idea of approximately how large the `MolMap` needs to be, it is
    /// recommended to use `MolMap.with_capacity()` or `with_capacities()` instead.
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

    /// Returns the core molecular graph (private).
    #[allow(private_interfaces)]
    fn core(&self) -> &MolGraph;

    /// Returns the core molecular graph (private) in mutable form.
    #[allow(private_interfaces)]
    fn core_mut(&mut self) -> &mut MolGraph;

    // ID-related methods
    // These all just defer to the inner core struct
    // One method per entity type for:
    // - iterating over IDs
    // - validating an ID

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

    /// Checks if the given ID corresponds to an atom currently in the map.
    fn contains_atom(&self, id: AtomId) -> bool {
        self.core().contains_atom(id)
    }

    /// Checks if the given ID corresponds to a pseudoatom currently in the map.
    fn contains_pseudoatom(&self, id: PseudoatomId) -> bool {
        self.core().contains_pseudoatom(id)
    }

    /// Checks if the given ID corresponds to a bond currently in the map.
    fn contains_bond(&self, id: BondId) -> bool {
        self.core().contains_bond(id)
    }

    /// Checks if the given ID corresponds to a substituent currently in the map.
    fn contains_substituent(&self, id: SubstituentId) -> bool {
        self.core().contains_substituent(id)
    }

    /// Checks if the given ID corresponds to a molecule currently in the map.
    fn contains_molecule(&self, id: MoleculeId) -> bool {
        self.core().contains_molecule(id)
    }

    /// Checks if the given ID corresponds to an atom or pseudoatom currently in the map.
    fn contains_atomlike(&self, atomlike: AtomlikeId) -> bool {
        match atomlike {
            AtomlikeId::Atom(id) => self.contains_atom(id),
            AtomlikeId::Pseudoatom(id) => self.contains_pseudoatom(id),
        }
    }

    /// Checks if the given ID corresponds to an atom, pseudoatom, or bond currently in the map.
    fn contains_fundamental(&self, fundamental: FundamentalId) -> bool {
        match fundamental {
            FundamentalId::Atom(id) => self.contains_atom(id),
            FundamentalId::Pseudoatom(id) => self.contains_pseudoatom(id),
            FundamentalId::Bond(id) => self.contains_bond(id),
        }
    }

    /// Checks if the given ID corresponds to an atom, pseudoatom, or substituent currently in the map.
    fn contains_bondable(&self, bondable: BondableId) -> bool {
        match bondable {
            BondableId::Atom(id) => self.contains_atom(id),
            BondableId::Pseudoatom(id) => self.contains_pseudoatom(id),
        }
    }

    /// Checks if the map currently contains the entity with the given ID.
    fn contains(&self, entity: EntityId) -> bool {
        match entity {
            EntityId::Atom(id) => self.contains_atom(id),
            EntityId::Pseudoatom(id) => self.contains_pseudoatom(id),
            EntityId::Bond(id) => self.contains_bond(id),
            EntityId::Substituent(id) => self.contains_substituent(id),
            EntityId::Molecule(id) => self.contains_molecule(id),
        }
    }

    // Getters
    // One method per entity type for:
    // - getting a view
    // - getting a mutable view
    // - iterating over (immutable) views

    /// Constructs an immutable `AtomView` for the given atom,
    /// returning `None` if the ID is invalid.
    fn atom(&'_ self, id: AtomId) -> Option<AtomView<'_, Self>> {
        self.core()
            .contains_atom(id)
            .then_some(AtomView { molmap: self, id })
    }

    /// Constructs a mutable `AtomViewMut` for the given atom, returning `None` if the ID is
    /// invalid.
    fn atom_mut(&'_ mut self, id: AtomId) -> Option<AtomViewMut<'_, Self>> {
        self.core()
            .contains_atom(id)
            .then_some(AtomViewMut { molmap: self, id })
    }

    /// Returns an iterator over views of all atoms in the map.
    fn atoms(&'_ self) -> impl Iterator<Item = AtomView<'_, Self>> + '_ {
        self.atom_ids().map(|id| self.atom(id).unwrap())
    }

    /// Constructs an immutable `PseudoatomView` for the given pseudoatom, returning `None` if the
    /// ID is invalid.
    fn pseudoatom(&'_ self, id: PseudoatomId) -> Option<PseudoatomView<'_, Self>> {
        self.core()
            .pseudoatoms
            .contains_key(id)
            .then_some(PseudoatomView { molmap: self, id })
    }

    /// Constructs a mutable `PseudoatomViewMut` for the given pseudoatom, returning `None` if the
    /// ID is invalid.
    fn pseudoatom_mut(&'_ mut self, id: PseudoatomId) -> Option<PseudoatomViewMut<'_, Self>> {
        self.core()
            .pseudoatoms
            .contains_key(id)
            .then_some(PseudoatomViewMut { molmap: self, id })
    }

    /// Returns an iterator over views of all pseudoatoms in the map.
    fn pseudoatoms(&'_ self) -> impl Iterator<Item = PseudoatomView<'_, Self>> + '_ {
        self.pseudoatom_ids().map(|id| self.pseudoatom(id).unwrap())
    }

    /// Constructs an immutable `BondView` for the given bond, returning `None` if the ID is
    /// invalid.
    fn bond(&'_ self, id: BondId) -> Option<BondView<'_, Self>> {
        self.core()
            .bonds
            .contains_key(id)
            .then_some(BondView { molmap: self, id })
    }

    /// Constructs a mutable `BondViewMut` for the given bond, returning `None` if the ID is
    /// invalid.
    fn bond_mut(&'_ mut self, id: BondId) -> Option<BondViewMut<'_, Self>> {
        self.core()
            .bonds
            .contains_key(id)
            .then_some(BondViewMut { molmap: self, id })
    }

    /// Returns an iterator over views of all atoms in the map.
    fn bonds(&'_ self) -> impl Iterator<Item = BondView<'_, Self>> + '_ {
        self.bond_ids().map(|id| self.bond(id).unwrap())
    }

    /// Constructs an immutable `SubstituentView` for the given substituent, returning `None` if the ID is
    /// invalid.
    fn substituent(&'_ self, id: SubstituentId) -> Option<SubstituentView<'_, Self>> {
        self.core()
            .substituents
            .contains_key(id)
            .then_some(SubstituentView { molmap: self, id })
    }

    /// Constructs a mutable `SubstituentViewMut` for the given substituent, returning `None` if the ID is
    /// invalid.
    fn substituent_mut(&'_ mut self, id: SubstituentId) -> Option<SubstituentViewMut<'_, Self>> {
        self.core()
            .substituents
            .contains_key(id)
            .then_some(SubstituentViewMut { molmap: self, id })
    }

    /// Returns an iterator over views of all substituents in the map.
    fn substituents(&'_ self) -> impl Iterator<Item = SubstituentView<'_, Self>> + '_ {
        self.substituent_ids()
            .map(|id| self.substituent(id).unwrap())
    }

    /// Constructs an immutable `MoleculeView` for the given molecule, returning `None` if the ID is
    /// invalid.
    fn molecule(&'_ self, id: MoleculeId) -> Option<MoleculeView<'_, Self>> {
        self.core()
            .molecules
            .contains_key(id)
            .then_some(MoleculeView { molmap: self, id })
    }

    /// Constructs a mutable `MoleculeViewMut` for the given molecule, returning `None` if the ID is
    /// invalid.
    fn molecule_mut(&'_ mut self, id: MoleculeId) -> Option<MoleculeViewMut<'_, Self>> {
        self.core()
            .molecules
            .contains_key(id)
            .then_some(MoleculeViewMut { molmap: self, id })
    }

    /// Returns an iterator over views of all molecules in the map.
    fn molecules(&'_ self) -> impl Iterator<Item = MoleculeView<'_, Self>> + '_ {
        self.molecule_ids().map(|id| self.molecule(id).unwrap())
    }
}
