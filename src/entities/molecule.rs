// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashSet;

use crate::*;

/// The core data of a molecule entity.
#[derive(Clone, Debug)]
pub struct MoleculeData {
    pub(crate) members: HashSet<AnyFundamental>,
}

impl MoleculeData {
    pub fn new() -> Self {
        Self {
            members: HashSet::new(),
        }
    }
}

pub type MoleculeView<'a, M> = View<'a, M, Molecule>;

impl<'a, M: MolMap> View<'a, M, Molecule> {
    /// Returns an iterator over the IDs of all constituent atoms, pseudoatoms, and bonds.
    pub fn members(&self) -> impl Iterator<Item = impl Fundamental> {
        self.data().members.iter().copied()
    }

    /// Checks if the molecule contains the given atom, pseudoatom, or bond.
    pub fn contains(&self, fundamental: impl Fundamental) -> bool {
        self.data().members.contains(&fundamental.as_fundamental())
    }
}
