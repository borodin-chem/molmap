// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Immutable and mutable views of an entity in a specific [`MolMap`].

use std::iter::FusedIterator;

use crate::{MolMap, entities::*};

// The view structs simply hold an immutable or mutable reference to the parent map,
// as appropriate, and the corresponding ID. Both have the visibility `pub(crate)`
// so that views can be easily constructed in other places in `molmap`, but not by
// users, as the existence of a view is considered proof that the ID is valid.
// A view should only ever be created after an ID has been validated or is already
// known with absolute certainty to be valid.
//
// Generally, the methods that need to be implemented for an entity's view types
// are specific to that entity and often also to a concrete `MolMap`, and so are
// defined elsewhere.
//
// However, a few methods are common to all views, and these are also implemented
// by the macro. The following are implemented by the macro for the public API:
//
// 1. a `pub` method to access the ID
// 2. a `From` implementation for the ID type from the view so that a view can be
//    easily used in places that expect an ID
//
// and a couple of convenience methods are implemented for internal development:
//
// 1. a private method to quickly access the core [`MolGraph`] held by `self.map`
// 2. _for mutable views_, a private method that converts the mutable view to an
//    immutable one, so that any methods on the immutable view can be used within
//    methods of the mutable one (note that this cannot be exposed publicly as it
//    does not consume the mutable view)

/// An immutable view of an individual entity in a specific [`MolMap`].
pub struct View<'m, M: MolMap, E: Entity> {
    pub(crate) map: &'m M,
    pub(crate) id: E,
}

impl<'m, M: MolMap, E: Entity> View<'m, M, E> {
    pub fn id(&self) -> E {
        self.id
    }
}

impl<'m, M: MolMap, E: Kind> View<'m, M, E> {
    /// Returns a reference to the entity's data struct in the core [`MolGraph`]."
    pub(crate) fn data(&self) -> &E::DATA {
        self.map.core().data(self.id).unwrap()
    }
}

/// A mutable view of an individual entity in a specific [`MolMap`].
///
/// All views are intended to be ephemeral, but this is especially the case for a
/// mutable view. A new one should be obtained from the parent map for each mutating
/// operation. As such, all public methods of a mutable view, other than `id`,
/// consume it.
#[derive(Debug)]
pub struct ViewMut<'m, M: MolMap, E: Entity> {
    pub(crate) map: &'m mut M,
    pub(crate) id: E,
}

impl<'m, M: MolMap, E: Entity> ViewMut<'m, M, E> {
    /// Returns an immutable view of the same entity.
    fn as_view(&'m self) -> View<'m, M, E> {
        View {
            map: &*self.map,
            id: self.id,
        }
    }
}

/// An iterator that yields an immutable view of each of a set of entities in turn.
pub struct Views<'m, M, E, I>
where
    M: MolMap,
    E: Entity,
    I: Iterator<Item = E>,
{
    pub(crate) map: &'m M,
    pub(crate) ids: I,
}

impl<'m, M, E, I> Views<'m, M, E, I>
where
    M: MolMap,
    E: Entity,
    I: Iterator<Item = E>,
{
    pub fn ids(self) -> I {
        self.ids
    }
}

impl<'m, M, E, I> Iterator for Views<'m, M, E, I>
where
    M: MolMap,
    E: Entity,
    I: Iterator<Item = E>,
{
    type Item = View<'m, M, E>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(id) = self.ids.next() {
            Some(View { map: self.map, id })
        } else {
            None
        }
    }
}

impl<'m, M, E, I> ExactSizeIterator for Views<'m, M, E, I>
where
    M: MolMap,
    E: Entity,
    I: Iterator<Item = E> + ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.ids.len()
    }
}

impl<'m, M, E, I> FusedIterator for Views<'m, M, E, I>
where
    M: MolMap,
    E: Entity,
    I: Iterator<Item = E> + FusedIterator,
{
}
