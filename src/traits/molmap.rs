// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{fmt::Debug, iter::FusedIterator};

use slotmap::SlotMap;

use crate::{
    entities::{AtomKey, atom::AtomData},
    graph::MolGraph,
    id::Id,
    view::ViewIter,
    *,
};

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

// The Stored trait allows methods of MolGraph and the MolMap types to be
// generic over all kinds of entity *when the same thing is done for each kind*.
// However, the number of methods in the public API this applies to is limited:
// - Methods that involve existing entities should go via the respective View
//   (which delegate to methods on the map, which are unlikely to be generic)
// - Methods that are different for each entity type (e.g. entity addition)
//   should not be generic (as they would require a new trait anyway)

/// An arena-like data structure to represent a set of chemical entities, their
/// properties, and the relationships between them, with or without spatial positions.
///
/// This trait provides methods for:
/// 1. obtaining an immutable or mutable view of an entity from its ID
/// 2. verifying an ID
/// 3. iterating over views of all of a given kind of entity
/// 4. iterating over all IDs of a given kind of entity
///
/// All implementors of `MolMap` should also always provide methods for adding new
/// entities, but the signature for these will vary according to the needs of the
/// concrete map type.
///
/// This trait is sealed and is not intended for implementation outside of `molmap`.
pub trait MolMap: Sized + MolMapCore {
    // Constructors
    // ------------

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

    // ID-related methods
    // ------------------

    /// Checks if the map currently contains the given entity.
    fn contains<E: Entity>(&self, id: E) -> bool {
        // This version of contains is flexible, for the public API, and can do
        // the check for any
        match id.as_tagged_entity() {
            entities::TaggedEntity::Atom(atom) => {
                Atom::get_slotmap(self.core()).contains_key(atom.to_key())
            }
            entities::TaggedEntity::Bond(bond) => {
                Bond::get_slotmap(self.core()).contains_key(bond.to_key())
            }
            entities::TaggedEntity::Pseudoatom(pseudoatom) => {
                Pseudoatom::get_slotmap(self.core()).contains_key(pseudoatom.to_key())
            }
            entities::TaggedEntity::Substituent(substituent) => {
                Substituent::get_slotmap(self.core()).contains_key(substituent.to_key())
            }
            entities::TaggedEntity::Molecule(molecule) => {
                Molecule::get_slotmap(self.core()).contains_key(molecule.to_key())
            }
        }
    }

    /// Returns an iterator over all the IDs of all of a given kind of entity in the map.
    fn iter_ids<E: Entity + Keyed>(
        &'_ self,
    ) -> impl Iterator<Item = E> + ExactSizeIterator + FusedIterator {
        self.core().iter_ids::<E>()
    }

    // Getters
    // -------
    // One method per entity kind (via monomorphization) for:
    // - getting a view
    // - getting a mutable view
    // - iterating over (immutable) views

    /// Constructs an immutable view of the given entity, returning `None` if the ID is invalid.
    fn view<E: Entity + Keyed>(&'_ self, id: E) -> Option<View<'_, Self, E>> {
        self.contains(id).then_some(View { map: self, id })
    }

    /// Constructs a mutable view of the given entity, returning `None` if the ID is invalid.
    fn view_mut<E: Entity + Keyed>(&'_ mut self, id: E) -> Option<ViewMut<'_, Self, E>> {
        self.contains(id).then_some(ViewMut { map: self, id })
    }

    /// Returns an iterator over views of all of a given kind of entity in the map.
    fn iter_views<E: Entity + Keyed>(
        &'_ self,
    ) -> ViewIter<'_, Self, E, impl Iterator<Item = E> + ExactSizeIterator> {
        ViewIter {
            map: self,
            ids: self.iter_ids(),
        }
    }
}
