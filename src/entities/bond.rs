// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use slotmap::{basic::Keys, new_key_type};

use crate::{
    entities::macros::define_entity_views,
    ids::{AtomId, AtomlikeId, BondId, BondIds, BondableId, KeyId, PseudoatomId, SubstituentId},
    traits::MolMap,
};

/// The type of a bond e.g. covalent, ionic.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BondType {
    Covalent,
    Intermolecular,
    Coordination,
    Ionic,
}

/// The core data of a bond entity.
#[derive(Clone, Debug)]
pub(crate) struct Bond {
    pub(crate) bond_type: BondType,
    pub(crate) order: f32,
    pub(crate) start: BondableId,
    pub(crate) end: BondableId,
}

impl Bond {
    pub fn new(bond_type: BondType, order: f32, start: BondableId, end: BondableId) -> Self {
        Self {
            bond_type,
            order,
            start,
            end,
        }
    }
}

define_entity_views!(Bond);

impl<'a, M: MolMap> BondView<'a, M> {
    pub fn bond_type(&self) -> BondType {
        self.core().bond_type
    }

    pub fn order(&self) -> f32 {
        self.core().order
    }

    pub fn partners(&self) -> [BondableId; 2] {
        let inner = self.core();
        [inner.start, inner.end]
    }
}
