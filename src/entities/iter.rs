// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::iter::FusedIterator;

use crate::{MolMap, entities::ids::*, views::*};

macro_rules! define_entity_iterators {
    ($name:ident) => {
        paste::paste! {
            #[doc = concat!(
                "An iterator over a set of ", stringify!([<$name:lower>]), " IDs"
            )]
            pub struct [<$name Ids>]<I: Iterator<Item = [<$name Id>]>>(pub(crate) I);

            impl<I> Iterator for [<$name Ids>]<I>
                where I: Iterator<Item = [<$name Id>]>
            {
                type Item = [<$name Id>];

                fn next(&mut self) -> Option<Self::Item> {
                    self.0.next()
                }
            }

            impl<I> ExactSizeIterator for [<$name Ids>]<I>
                where I: Iterator<Item = [<$name Id>]> + ExactSizeIterator
            {
                fn len(&self) -> usize {
                    self.0.len()
                }
            }

            impl<I> FusedIterator for [<$name Ids>]<I>
                where I: Iterator<Item = [<$name Id>]> + FusedIterator
            {}

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

define_entity_iterators!(Atom);
define_entity_iterators!(Pseudoatom);
define_entity_iterators!(Bond);
define_entity_iterators!(Substituent);
define_entity_iterators!(Molecule);
