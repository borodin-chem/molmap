// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{BondId, MolMap, SubstituentId, ObjectId, PseudoatomId};

// Something that has a "symbol" like a normal atom but represents something else
// May have an unknown composition like R, or a known structure like Ph
#[derive(Debug)]
pub struct Pseudoatom {
    pub id: PseudoatomId,
    pub symbol: String,
    pub annotations: Vec<ObjectId>,
}

impl Pseudoatom {
    pub fn new(id: PseudoatomId, symbol: String) -> Self {
        Self {
            id,
            symbol,
            annotations: Vec::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct PseudoatomView<'a> {
    pub molmap: &'a MolMap,
    pub id: PseudoatomId,
}

impl<'a> From<PseudoatomView<'a>> for PseudoatomId {
    fn from(view: PseudoatomView<'a>) -> Self {
        view.id
    }
}

impl<'a> PseudoatomView<'a> {
    fn inner(&self) -> &'a Pseudoatom {
        self.molmap.pseudoatoms.get(self.id).unwrap()
    }
    
    pub fn symbol(&self) -> &str {
        &self.inner().symbol
    }
}

pub struct PseudoatomViewMut<'a> {
    pub molmap: &'a mut MolMap,
    pub id: PseudoatomId,
}

impl<'a> From<PseudoatomViewMut<'a>> for PseudoatomId {
    fn from(view: PseudoatomViewMut<'a>) -> Self {
        view.id
    }
}

impl<'a> PseudoatomViewMut<'a> {
    fn as_ref(&self) -> PseudoatomView<'_> {
        PseudoatomView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    fn inner(&mut self) -> &mut Pseudoatom {
        self.molmap.pseudoatoms.get_mut(self.id).unwrap()
    }
}
