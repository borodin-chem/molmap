// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Immutable and mutable views of an entity in a specific [`MolMap`].

use std::iter::FusedIterator;

use crate::entities::*;

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
#[derive(Copy, Clone, Debug)]
pub struct View<'m, M, E> {
    pub(crate) map: &'m M,
    pub(crate) id: E,
}

impl<'m, M, E> View<'m, M, E>
where
    E: Entity,
{
    pub fn id(&self) -> E {
        self.id
    }
}

/// A mutable view of an individual entity in a specific [`MolMap`].
///
/// All views are intended to be ephemeral, but this is especially the case for
/// mutable views. A new one should be obtained from the parent map for each mutating
/// operation. As such, all public methods of a mutable view, other than `id`,
/// consume it.
#[derive(Debug)]
pub struct ViewMut<'m, M, E> {
    pub(crate) map: &'m mut M,
    pub(crate) id: E,
}

impl<'m, M, E> ViewMut<'m, M, E>
where
    E: Entity,
{
    /// Returns an immutable view of the same entity.
    #[allow(unused)]
    pub(crate) fn as_view(&'m self) -> View<'m, M, E> {
        View {
            map: &*self.map,
            id: self.id,
        }
    }
}

/// An iterator that yields an immutable view of each of a set of entities in turn.
pub struct Views<'m, M, E>
where
    E: Kind,
{
    pub(crate) map: &'m M,
    pub(crate) ids: Entities<'m, E>,
}

impl<'m, M, E> Views<'m, M, E>
where
    E: Kind,
{
    pub fn ids(self) -> Entities<'m, E> {
        self.ids
    }
}

impl<'m, M, E> Iterator for Views<'m, M, E>
where
    E: Kind,
{
    type Item = View<'m, M, E>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(id) = self.ids.next() {
            Some(View { map: self.map, id })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ids.size_hint()
    }
}

impl<'m, M, E> ExactSizeIterator for Views<'m, M, E> where E: Kind {}

impl<'m, M, E> FusedIterator for Views<'m, M, E> where E: Kind {}
