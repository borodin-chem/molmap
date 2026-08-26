// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{MolMap, Pseudoelement, categories::*, entities::*, view::*};

/// The core data of a pseudoatom entity.
#[derive(Clone, Debug)]
pub struct PseudoatomData {
    pub(crate) pseudoelement: Pseudoelement,
    pub(crate) bonds: Vec<Bond>,
}

impl PseudoatomData {
    pub fn new(pseudoelement: Pseudoelement) -> Self {
        Self {
            pseudoelement,
            bonds: Vec::new(),
        }
    }
}

impl<'m, M: MolMap> View<'m, M, Pseudoatom> {
    pub fn bonds(&self) -> &[Bond] {
        &self.data().bonds
    }
}
