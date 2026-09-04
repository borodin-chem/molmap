// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Definitions of the slotmap::Key types for each kind of entity and their
//! correspondences to the entity types.

use slotmap::{Key, SlotMap, new_key_type};

use crate::{entities::*, graph::MolGraph, graph::data::*, id::Id};

/// A fundamental kind of entity in the graph, with a backing `SlotMap`.
pub trait Keyed: Entity {
    type KEY: slotmap::Key + 'static;
    type DATA: 'static;

    const KIND: EntityKind;

    fn from_key(key: Self::KEY) -> Self {
        Self::new_unchecked(Id::from_key_data(Self::KIND, key.data()))
    }

    fn to_key(self) -> Self::KEY {
        Self::KEY::from(self.into_inner().to_key_data())
    }

    /// Returns a reference to the graph's `SlotMap` that holds this entity.
    fn get_slotmap(graph: &MolGraph) -> &SlotMap<Self::KEY, Self::DATA>;

    /// Returns a mutable reference to the graph's `SlotMap` that holds this entity.
    fn get_slotmap_mut(graph: &mut MolGraph) -> &mut SlotMap<Self::KEY, Self::DATA>;
}

macro_rules! impl_keyed {
    ($kind:ident) => {
        paste::paste! {
            new_key_type! { pub struct [<$kind Key>]; }

            impl Keyed for $kind {
                type KEY = [<$kind Key>];
                type DATA = [<$kind Data>];

                const KIND: EntityKind = EntityKind::$kind;

                #[inline]
                fn get_slotmap(map: &MolGraph) -> &SlotMap<Self::KEY, Self::DATA> {
                    &map.[<$kind:lower s>]
                }

                #[inline]
                fn get_slotmap_mut(map: &mut MolGraph) -> &mut SlotMap<Self::KEY, Self::DATA> {
                    &mut map.[<$kind:lower s>]
                }
            }

            impl From<$kind> for [<$kind Key>] {
                fn from(id: $kind) -> Self {
                    id.to_key()
                }
            }

            impl From<[<$kind Key>]> for $kind {
                fn from(key: [<$kind Key>]) -> Self {
                    $kind::from_key(key)
                }
            }
        }
    };
}

impl_keyed!(Atom);
impl_keyed!(Bond);
impl_keyed!(Pseudoatom);
impl_keyed!(Substituent);
impl_keyed!(Molecule);
