// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashSet;

use slotmap::{basic::Keys, new_key_type};

use crate::{
    entities::macros::define_entity_views,
    ids::{FundamentalId, MoleculeId, MoleculeIds},
    traits::MolMap,
};

/// The core data of a molecule entity.
#[derive(Clone, Debug)]
pub(crate) struct Molecule {
    pub(crate) members: HashSet<FundamentalId>,
}

impl Molecule {
    pub fn new() -> Self {
        Self {
            members: HashSet::new(),
        }
    }
}

define_entity_views!(Molecule);

impl<'a, M: MolMap> MoleculeView<'a, M> {
    /// Returns an iterator over the IDs of all constituent atoms, pseudoatoms, and bonds.
    pub fn members(&self) -> impl Iterator<Item = FundamentalId> {
        self.core().members.iter().copied()
    }

    /// Checks if the molecule contains the given atom, pseudoatom, or bond.
    pub fn contains(&self, fundamental: FundamentalId) -> bool {
        self.core().members.contains(&fundamental)
    }
}
