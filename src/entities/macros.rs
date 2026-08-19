// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::MolMap;

// It isn't possible to define the views here without a circular reference, as
// the view definitions require the core structs to have been defined first.

/// Defines immutable and mutable views of an entity type for a generic [`MolMap`],
/// as well as an iterator over the immutable view type.
///
/// The view structs simply hold an immutable or mutable reference to the parent map,
/// as appropriate, and the corresponding ID. Both have the visibility `pub(crate)`
/// so that views can be easily constructed in other places in `molmap`, but not by
/// users, as the existence of a view is considered proof that the ID is valid.
/// A view should only ever be created after an ID has been validated or is already
/// known with absolute certainty to be valid.
///
/// Generally, the methods that need to be implemented for an entity's view types
/// are specific to that entity and often also to a concrete `MolMap`, and so are
/// defined elsewhere.
///
/// However, a few methods are common to all views, and these are also implemented
/// by the macro. The following are implemented by the macro for the public API:
///
/// 1. a `pub` method to access the ID
/// 2. a `From` implementation for the ID type from the view so that a view can be
///    easily used in places that expect an ID
///
/// and a couple of convenience methods are implemented for internal development:
///
/// 1. a private method to quickly access the core [`MolGraph`] held by `self.map`
/// 2. _for mutable views_, a private method that converts the mutable view to an
///    immutable one, so that any methods on the immutable view can be used within
///    methods of the mutable one (note that this cannot be exposed publicly as it
///    does not consume the mutable view)
macro_rules! define_entity_views {
    ($name:ident) => {
        use std::iter::FusedIterator;

        paste::paste! {
            #[doc = concat!(
                "An immutable view of an individual", stringify!([<$name:lower>]), "in a specific [`MolMap`].",
            )]
            #[derive(Copy, Clone, Debug)]
            pub struct [<$name View>]<'a, M: MolMap> {
                pub(crate) map: &'a M,
                pub(crate) id: [<$name Id>],
            }

            impl<'a, M: MolMap> [<$name View>]<'a, M> {
                #[doc = concat!(
                    "Returns a reference to the corresponding ", stringify!([<$name>]), " struct in the core [`MolGraph`]."
                )]
                fn core(&self) -> &[<$name>] {
                    self.map.core().[<$name:lower s>].get(self.id).unwrap()
                }

                pub fn id(&self) -> [<$name Id>] {
                    self.id
                }
            }

            impl<'a, M: MolMap> From<[<$name View>]<'a, M>> for [<$name Id>] {
                fn from(view: [<$name View>]<'a, M>) -> Self {
                    view.id
                }
            }

            #[doc = concat!(
                "A mutable view of an individual ", stringify!([<$name:lower>]), " in a specific [`MolMap`].\n\
                \n\
                All views are intended to be ephemeral, but this is especially the case for a \
                mutable view. A new one should be obtained from the parent map for each mutating \
                operation. As such, all public methods of a mutable view, other than `id`, \
                consume it."
            )]
            #[derive(Debug)]
            pub struct [<$name ViewMut>]<'a, M: MolMap> {
                pub(crate) map: &'a mut M,
                pub(crate) id: [<$name Id>],
            }

            impl<'a, M: MolMap> [<$name ViewMut>]<'a, M> {
                #[doc = concat!(
                    "Returns a mutable reference to the corresponding ", stringify!([<$name>]), " struct in the core [`MolGraph`]."
                )]
                fn core(&mut self) -> &mut [<$name>] {
                    self.map.core_mut().[<$name:lower s>].get_mut(self.id).unwrap()
                }

                #[doc = concat!(
                    "Returns an immutable view of the same ", stringify!([<$name:lower>])
                )]
                fn as_view(&self) -> [<$name View>]<'_, M> {
                    [<$name View>] {
                        map: &*self.map,
                        id: self.id,
                    }
                }
            }

            impl<'a, M: MolMap> From<[<$name ViewMut>]<'a, M>> for [<$name Id>] {
                fn from(view: [<$name ViewMut>]<'a, M>) -> Self {
                    view.id
                }
            }

            #[doc = concat!(
                "An iterator that yields an immutable view of each of a set of ", stringify!([<$name:lower>]), "s in turn."
            )]
            pub struct [<$name Views>]<'a, M: MolMap, I: Iterator<Item = [<$name Id>]>> {
                pub(crate) map: &'a M,
                pub(crate) ids: [<$name Ids>]<I>,
            }

            impl<'a, M: MolMap, I: Iterator<Item = [<$name Id>]>> [<$name Views>]<'a, M, I> {
                pub fn ids(self) -> [<$name Ids>]<I> {
                    self.ids
                }
            }

            impl<'a, M: MolMap, I: Iterator<Item = [<$name Id>]>> Iterator for [<$name Views>]<'a, M, I> {
                type Item = [<$name View>]<'a, M>;

                fn next(&mut self) -> Option<Self::Item> {
                    if let Some(id) = self.ids.next() {
                        Some([<$name View>] { map: self.map, id })
                    } else {
                        None
                    }
                }
            }

            impl<'a, M: MolMap, I: Iterator<Item = [<$name Id>]> + ExactSizeIterator> ExactSizeIterator for [<$name Views>]<'a, M, I> {
                fn len(&self) -> usize {
                    self.ids.len()
                }
            }

            impl<'a, M: MolMap, I: Iterator<Item = [<$name Id>]> + FusedIterator> FusedIterator for [<$name Views>]<'a, M, I> {}
        }
    }
}
pub(crate) use define_entity_views;
