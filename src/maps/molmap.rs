// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{fmt::Debug, iter::FusedIterator};

use nalgebra::{Point, Point2};
use slotmap::SecondaryMap;
use slotmap::SlotMap;

use crate::categories::Collection;
use crate::{entities::*, graph::MolGraph, graph::keys::*, view::*};

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
    /// As the constituent `SlotMap`s are created with an initial capacity of 0,
    /// reallocations will occur frequently if many entities are subsequently inserted.
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
    fn contains<E: Entity>(&self, entity: E) -> bool {
        // This version of contains is flexible, for the public API, and can do
        // the check for any Entity type, not just Keyed ones
        match entity.to_resolved() {
            ResolvedEntity::Atom(atom) => {
                Atom::get_slotmap(self.core()).contains_key(atom.to_key())
            }
            ResolvedEntity::Bond(bond) => {
                Bond::get_slotmap(self.core()).contains_key(bond.to_key())
            }
            ResolvedEntity::Pseudoatom(pseudoatom) => {
                Pseudoatom::get_slotmap(self.core()).contains_key(pseudoatom.to_key())
            }
            ResolvedEntity::Substituent(substituent) => {
                Substituent::get_slotmap(self.core()).contains_key(substituent.to_key())
            }
            ResolvedEntity::Molecule(molecule) => {
                Molecule::get_slotmap(self.core()).contains_key(molecule.to_key())
            }
        }
    }

    /// Returns an iterator over all of a given kind of entity in the map.
    fn iter<E: Kind>(&'_ self) -> impl Iterator<Item = E> + ExactSizeIterator + FusedIterator {
        self.core().iter::<E>()
    }

    // Getters
    // -------
    // One method per entity kind (via monomorphization) for:
    // - getting a view
    // - getting a mutable view
    // - iterating over (immutable) views

    /// Constructs an immutable view of the given entity, returning `None` if the ID is invalid.
    fn view<E: Kind>(&'_ self, entity: E) -> Option<View<'_, Self, E>> {
        self.contains(entity).then_some(View {
            map: self,
            id: entity,
        })
    }

    /// Constructs a mutable view of the given entity, returning `None` if the ID is invalid.
    fn view_mut<E: Kind>(&'_ mut self, entity: E) -> Option<ViewMut<'_, Self, E>> {
        self.contains(entity).then_some(ViewMut {
            map: self,
            id: entity,
        })
    }

    /// Returns an iterator over views of all the given entities, returning `None`
    /// if any ID is invalid.
    ///
    /// This method is most useful for situations where it is important to have
    /// ensured all IDs are valid before beginning some operation involving them.
    ///
    /// As the IDs are validated eagerly using a clone of the iterator, in other
    /// situations it is probably more sensible to use `map` on the ID iterator
    /// to get an iterator that returns views for each entity in turn, lazily.
    fn views<E, I>(&'_ mut self, entities: I) -> Option<Views<'_, Self, E, I::IntoIter>>
    where
        E: Kind,
        I: IntoIterator<Item = E>,
        I::IntoIter: Clone,
    {
        let entities = entities.into_iter();
        if entities.clone().all(|e| self.contains(e)) {
            Some(Views {
                map: self,
                ids: entities,
            })
        } else {
            None
        }
    }

    /// Returns an iterator over views of all of a given kind of entity in the map.
    fn iter_views<E: Kind>(
        &'_ self,
    ) -> Views<'_, Self, E, impl Iterator<Item = E> + ExactSizeIterator> {
        Views {
            map: self,
            ids: self.iter(),
        }
    }
}
