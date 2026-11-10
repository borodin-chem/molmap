// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The crate's various iterator types.

use std::iter::FusedIterator;

use slotmap::basic::Keys;

use crate::{View, entities::Kind};

/// A union type for iterators over entity IDs from various sources.
enum EntitiesSource<'m, E: Kind> {
    Keys(Keys<'m, E::Key, E::Data>),
    Vec(std::vec::IntoIter<E>),
}

impl<'m, E: Kind> Iterator for EntitiesSource<'m, E> {
    type Item = E;

    fn next(&mut self) -> Option<E> {
        match self {
            Self::Keys(keys) => keys.next().map(E::from_key),
            Self::Vec(ids) => ids.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Keys(keys) => keys.size_hint(),
            Self::Vec(ids) => ids.size_hint(),
        }
    }
}

impl<'m, E: Kind> ExactSizeIterator for EntitiesSource<'m, E> {}

impl<'m, E: Kind> FusedIterator for EntitiesSource<'m, E> {}

/// An iterator over entity IDs.
pub struct Entities<'m, E: Kind>(EntitiesSource<'m, E>);

impl<'m, E: Kind> Entities<'m, E> {
    pub(crate) fn from_keys(keys: Keys<'m, E::Key, E::Data>) -> Self {
        Self(EntitiesSource::Keys(keys))
    }

    pub(crate) fn from_vec(vec: Vec<E>) -> Self {
        Self(EntitiesSource::Vec(vec.into_iter()))
    }
}

impl<'m, E: Kind> Iterator for Entities<'m, E> {
    type Item = E;

    fn next(&mut self) -> Option<E> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'m, E: Kind> ExactSizeIterator for Entities<'m, E> {}

impl<'m, E: Kind> FusedIterator for Entities<'m, E> {}

/// An iterator over _all_ of a given kind of entity in a map.
pub type Iter<'m, E> = Entities<'m, E>;

/// A subset of entities selected from some larger set.
pub enum Selection<'m, E: Kind> {
    All,
    Iter(Box<dyn Iterator<Item = E> + 'm>),
}

impl<'m, E, I> From<I> for Selection<'m, E>
where
    E: Kind,
    I: IntoIterator<Item = E>,
    I::IntoIter: 'm,
{
    fn from(iterable: I) -> Self {
        Selection::Iter(Box::new(iterable.into_iter()))
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
