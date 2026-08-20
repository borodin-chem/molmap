// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::*;

/// A pseudoatom: something that forms bonds and can be represented by an
/// "element symbol" like a normal atom but represents something else.
///
/// It may have an unknown composition like R, or a known structure like Ph.
#[derive(Copy, Clone, Debug)]
pub struct Pseudoatom;

impl Entity for Pseudoatom {}

impl KeyEntity for Pseudoatom {
    fn kind() -> EntityKind {
        EntityKind::Pseudoatom
    }
}

/// The core data of a pseudoatom entity.
///
/// A pseudoatom is something that has a "symbol" like a normal atom but
/// represents something else.
/// It may have an unknown composition like R, or a known structure like Ph.
#[derive(Clone, Debug)]
pub(crate) struct PseudoatomData {
    pub(crate) pseudoelement: Pseudoelement,
    pub(crate) bonds: Vec<Id<Bond>>,
}

impl PseudoatomData {
    pub fn new(pseudoelement: Pseudoelement) -> Self {
        Self {
            pseudoelement,
            bonds: Vec::new(),
        }
    }
}

impl<'a, M: MolMap> View<'a, M, Pseudoatom> {
    fn core(&self) -> &PseudoatomData {
        self.map.core().pseudoatoms.get(self.id).unwrap()
    }

    pub fn bonds(&self) -> &[Id<Bond>] {
        &self.core().bonds
    }
}
