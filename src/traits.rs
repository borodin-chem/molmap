// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! General traits implemented by the different graphs and maps at each level.
//!
//! Traits specific to each layer are defined in the appropriate module.

use slotmap::SlotMap;

use crate::{View, ViewMut, Views, entities::*, error::*, units::Unit};

/// Traits that must remain unnameable from outside the crate.
#[allow(unreachable_pub)]
mod internal {
    use super::*;

    /// Implemented by maps to indicate that they store the specific
    /// kind of entity and provide access to its underlying data struct.
    pub trait Stores<E: Kind> {
        /// Returns a reference to the `SlotMap` that holds the entity.
        fn slotmap(&self) -> &SlotMap<E::Key, E::Data>;

        /// Returns a reference to the entity's data struct, or `None` if the entity is invalid.
        fn data(&self, entity: E) -> Option<&E::Data> {
            self.slotmap().get(entity.to_key())
        }

        /// Returns an iterator over all the keys of this kind of entity in the map.
        fn keys(&'_ self) -> slotmap::basic::Keys<'_, E::Key, E::Data> {
            self.slotmap().keys()
        }

        /// Returns the number of entities of this kind present in the map.
        fn count(&self) -> usize {
            self.slotmap().len()
        }
    }

    /// Implemented by maps to provide themselves with convenient mutable access
    /// to an entity's underlying data struct.
    pub(crate) trait StoresMut<E: Kind>: Stores<E> {
        /// Returns a mutable reference to the `SlotMap` that holds the entity.
        fn slotmap_mut(&mut self) -> &mut SlotMap<E::Key, E::Data>;

        /// Returns a mutable reference to the entity's data struct, or `None` if the entity is invalid.
        fn data_mut(&mut self, entity: E) -> Option<&mut E::Data> {
            self.slotmap_mut().get_mut(entity.to_key())
        }
    }
}
// Re-export for **crate-internal** use
pub(crate) use internal::*;

/// An iterator over all of a given kind of entity in a map.
pub type Iter<'m, E> = Entities<'m, E>;

/// An arena-like data structure to represent a set of chemical entities, their
/// properties, and the relationships between them, with or without spatial positions.
///
/// All `Map` types additionally implement one of [`Graph`] and [`Spatial`].
///
/// As all maps hold, at minimum, fundamental atomic and bond entity types, all
/// `Map` types also implement `Atomic`.
pub trait Map: Sized {
    // Most interaction with the crate's map types is through entity IDs and views of
    // the entities. This trait provides methods for:
    // 1. obtaining an immutable or mutable view of an entity from its ID
    // 2. verifying an ID
    // 3. iterating over views of all of a given kind of entity
    // 4. iterating over all IDs of a given kind of entity
    //
    // Other map-level methods are provided as inherent implementations on the map
    // itself, as methods of the layer traits, or via the `Spatial` trait for maps
    // with spatial positions.
    //
    // All implementors of `Map` provide methods for adding new entities; these are
    // not trait methods because the signature for these will vary according to the
    // needs of the concrete map type.
    //
    // This trait is sealed and is not intended for implementation outside of `molmap`.

    /// The corresponding map type that has no spatial positions.
    ///
    /// For pure-graph maps, this is simply `Self`.
    type Graph: Graph;

    /// Returns the map without any spatial positions.
    ///
    /// For pure-graph maps, this simply returns `self`.
    fn into_graph(self) -> Self::Graph;

    /// Provides read-only access to the graph in a generic fashion.
    fn as_graph(&self) -> &Self::Graph;

    // Constructors
    // ------------

    ///// Creates an empty map.
    /////
    ///// As the constituent `SlotMap`s are created with an initial capacity of 0,
    ///// reallocations will occur frequently if many entities are subsequently inserted.
    ///// If you have an idea of approximately how large the map needs to be, it is
    ///// recommended to use `with_capacity` or `with_capacities` instead.
    //fn new() -> Self;

    ///// Creates a new `MolMap` with the specified initial capacities for each kind of entity.
    //fn with_capacities(
    //    atoms: usize,
    //    pseudoatoms: usize,
    //    bonds: usize,
    //    substituents: usize,
    //    molecules: usize,
    //) -> Self;

    ///// Creates a new `MolMap` with initial capacity for approximately `n` atoms.
    /////
    ///// In the default implementation, this results in a map with capacity for:
    ///// - `n` atoms
    ///// - `n / 10` pseudoatoms
    ///// - `n` bonds
    ///// - `n / 3` substituents
    ///// - `(n / 100) + 1` molecules
    //fn with_capacity(n: usize) -> Self {
    //    Self::with_capacities(n, n / 10, n, n / 3, (n / 100) + 1)
    //}

    // ID-related methods
    // ------------------

    /// Checks if the map currently contains the given entity.
    ///
    /// This method must be flexible and able to do the check for any [`Entity`] type,
    /// not just ones actually in the map, and including category types.
    fn contains<E: Entity>(&self, entity: E) -> bool;

    /// Returns an iterator over all of a given kind of entity in the map.
    fn iter<E>(&'_ self) -> Iter<'_, E>
    where
        E: Kind,
        Self::Graph: Stores<E>,
    {
        Iter::from_keys(self.as_graph().keys())
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

    /// Returns an iterator over views of the given entities, returning an error
    /// if any ID is invalid.
    ///
    /// Pass [`Selection::All`] to get an iterator over views of all of the given
    /// kind of entity (in which case the method is infallible).
    ///
    /// This method is most useful for situations where it is important to have
    /// ensured all IDs are valid before beginning some operation involving them.
    ///
    /// As the IDs are validated eagerly and then stored on the heap, in other
    /// situations it is probably more sensible to use `map` on the ID iterator
    /// to get an iterator that returns views for each entity in turn, lazily.
    ///
    /// # Errors
    ///
    /// Fails if any ID is invalid, in which case the error includes the first
    /// invalid ID encountered.
    fn views<'m, E>(
        &'m self,
        selection: impl Into<Selection<'m, E>>,
    ) -> MolMapResult<Views<'m, Self, E>>
    where
        E: Kind,
        Self::Graph: Stores<E>,
    {
        let ids: Entities<'m, E> = match selection.into() {
            Selection::All => self.iter(),
            Selection::Iter(iterator) => {
                let validated: Vec<E> = iterator
                    .into_iter()
                    .map(|e| {
                        if self.contains(e) {
                            Ok(e)
                        } else {
                            Err(MolMapError::InvalidId(e.as_entity()))
                        }
                    })
                    // An iterator of Result<T, U> can be collected into Result<Vec<T>, U>
                    .collect::<MolMapResult<Vec<E>>>()?;
                Entities::from_vec(validated)
            }
        };
        Ok(Views { map: self, ids })
    }
}

/// A [`Map`] type that represents a collection of molecular entities only in
/// the form of a graph, without any spatial information.
pub trait Graph: Map {}

/// A map that holds the positions of its entities in `D`-dimensional space.
pub trait Spatial<const D: usize>: Map {
    /// The unit represented by a length of 1 in the map.
    type Unit: Unit;
}
