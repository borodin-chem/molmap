// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use slotmap::{basic::Keys, new_key_type};

use crate::{
    Pseudoelement,
    entities::macros::define_entity_views,
    ids::{BondId, KeyId, PseudoatomId, PseudoatomIds},
    traits::MolMap,
};

/// The core data of a pseudoatom entity.
///
/// A pseudoatom is something that has a "symbol" like a normal atom but
/// represents something else.
/// It may have an unknown composition like R, or a known structure like Ph.
#[derive(Clone, Debug)]
pub(crate) struct Pseudoatom {
    pub(crate) pseudoelement: Pseudoelement,
    pub(crate) bonds: Vec<BondId>,
}

impl Pseudoatom {
    pub fn new(pseudoelement: Pseudoelement) -> Self {
        Self {
            pseudoelement,
            bonds: Vec::new(),
        }
    }
}

define_entity_views!(Pseudoatom);

impl<'a, M: MolMap> PseudoatomView<'a, M> {
    pub fn bonds(&self) -> &[BondId] {
        &self.core().bonds
    }
}
