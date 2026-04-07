// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use slotmap::{SlotMap, basic::Iter};

use std::{fmt::Debug, hash::RandomState};

use crate::{Element, graph::MolGraph, bond::BondType, entities::*, fragment::FragmentCentre, id::*};

/// Trait implemented by all varieties of `MolMap`.
/// 
/// All concrete `MolMap` types wrap a [`MolGraph`], so this trait exposes functionality that
/// operates on the core graph such that the different varieties of `MolMap` can be used
/// interchangeably in many instances.
pub trait MolMap: Debug + Default {
    /// Creates an empty `MolMap`.
    ///
    /// As the constituent `SlotMap`s are created with an initial capacity of 0, reallocations will
    /// occur frequently if many entities are subsequently inserted.
    /// If you have an idea of approximately how large the `MolMap` needs to be, it is recommended
    /// to use `MolMap.with_capacity()` instead.
    fn new() -> Self;

    /// Creates a new `MolMap` with capacity for approximately `n` atoms.
    fn with_capacity(n: usize) -> Self;

    /// Returns the core molecular graph.
    #[allow(private_interfaces)]
    fn core(&self) -> &MolGraph;

    /// Returns the core molecular graph in mutable form.
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

    /// Returns an iterator over all the IDs of all fragments in the map.
    fn fragment_ids(&'_ self) -> impl Iterator<Item = FragmentId> + '_ {
        self.core().fragment_ids()
    }

    /// Returns an iterator over all the IDs of all molecules in the map.
    fn molecule_ids(&'_ self) -> impl Iterator<Item = MoleculeId> + '_ {
        self.core().molecule_ids()
    }

    /// Checks if the given ID is valid.
    fn contains_atom(&self, id: AtomId) -> bool {
        self.core().contains_atom(id)
    }

    /// Checks if the given ID is valid.
    fn contains_pseudoatom(&self, id: PseudoatomId) -> bool {
        self.core().contains_pseudoatom(id)
    }

    /// Checks if the given ID is valid.
    fn contains_bond(&self, id: BondId) -> bool {
        self.core().contains_bond(id)
    }

    /// Checks if the given ID is valid.
    fn contains_fragment(&self, id: FragmentId) -> bool {
        self.core().contains_fragment(id)
    }

    /// Checks if the given ID is valid.
    fn contains_molecule(&self, id: MoleculeId) -> bool {
        self.core().contains_molecule(id)
    }

    /// Checks if the given enum wraps a valid ID.
    fn contains_atomlike(&self, atomlike: Atomlike) -> bool {
        match atomlike {
            Atomlike::Atom(id) => self.contains_atom(id),
            Atomlike::Pseudoatom(id) => self.contains_pseudoatom(id),
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
        self.core().contains_atom(id).then_some(AtomView { molmap: self, id })
    }

    /// Constructs a mutable `AtomViewMut` for the given atom, returning `None` if the ID is
    /// invalid.
    fn atom_mut(&'_ mut self, id: AtomId) -> Option<AtomViewMut<'_, Self>> {
        self.core().contains_atom(id).then_some(AtomViewMut { molmap: self, id })
    }

    /// Constructs an immutable `PseudoatomView` for the given pseudoatom, returning `None` if the
    /// ID is invalid.
    fn pseudoatom(&'_ self, id: PseudoatomId) -> Option<PseudoatomView<'_, Self>> {
        self.core().pseudoatoms
            .contains_key(id)
            .then_some(PseudoatomView { molmap: self, id })
    }

    /// Constructs a mutable `PseudoatomViewMut` for the given pseudoatom, returning `None` if the
    /// ID is invalid.
    fn pseudoatom_mut(&'_ mut self, id: PseudoatomId) -> Option<PseudoatomViewMut<'_, Self>> {
        self.core().pseudoatoms
            .contains_key(id)
            .then_some(PseudoatomViewMut { molmap: self, id })
    }

    /// Constructs an immutable `BondView` for the given bond, returning `None` if the ID is
    /// invalid.
    fn bond(&'_ self, id: BondId) -> Option<BondView<'_, Self>> {
        self.core().bonds
            .contains_key(id)
            .then_some(BondView { molmap: self, id })
    }

    /// Constructs a mutable `BondViewMut` for the given bond, returning `None` if the ID is
    /// invalid.
    fn bond_mut(&'_ mut self, id: BondId) -> Option<BondViewMut<'_, Self>> {
        self.core().bonds
            .contains_key(id)
            .then_some(BondViewMut { molmap: self, id })
    }

    /// Constructs an immutable `FragmentView` for the given fragment, returning `None` if the ID is
    /// invalid.
    fn fragment(&'_ self, id: FragmentId) -> Option<FragmentView<'_, Self>> {
        self.core().fragments
            .contains_key(id)
            .then_some(FragmentView { molmap: self, id })
    }

    /// Constructs a mutable `FragmentViewMut` for the given fragment, returning `None` if the ID is
    /// invalid.
    fn fragment_mut(&'_ mut self, id: FragmentId) -> Option<FragmentViewMut<'_, Self>> {
        self.core().fragments
            .contains_key(id)
            .then_some(FragmentViewMut { molmap: self, id })
    }

    /// Constructs an immutable `MoleculeView` for the given molecule, returning `None` if the ID is
    /// invalid.
    fn molecule(&'_ self, id: MoleculeId) -> Option<MoleculeView<'_, Self>> {
        self.core().molecules
            .contains_key(id)
            .then_some(MoleculeView { molmap: self, id })
    }

    /// Constructs a mutable `MoleculeViewMut` for the given molecule, returning `None` if the ID is
    /// invalid.
    fn molecule_mut(&'_ mut self, id: MoleculeId) -> Option<MoleculeViewMut<'_, Self>> {
        self.core().molecules
            .contains_key(id)
            .then_some(MoleculeViewMut { molmap: self, id })
    }
}
