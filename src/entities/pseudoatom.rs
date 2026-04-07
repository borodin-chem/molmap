// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{BondId, FragmentId, MolMap, PseudoatomId};

// Something that has a "symbol" like a normal atom but represents something else
// May have an unknown composition like R, or a known structure like Ph
#[derive(Debug)]
pub(crate) struct Pseudoatom {
    pub(crate) symbol: String,
    pub(crate) bonds: Vec<BondId>,
    //pub annotations: Vec<ObjectId>,
}

impl Pseudoatom {
    pub(crate) fn new(symbol: String) -> Self {
        Self {
            symbol,
            bonds: Vec::new(),
            //annotations: Vec::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct PseudoatomView<'a, M: MolMap> {
    pub molmap: &'a M,
    pub id: PseudoatomId,
}

impl<'a, M: MolMap> From<PseudoatomView<'a, M>> for PseudoatomId {
    fn from(view: PseudoatomView<'a, M>) -> Self {
        view.id
    }
}

impl<'a, M: MolMap> PseudoatomView<'a, M> {
    fn core(&self) -> &'a Pseudoatom {
        self.molmap.core().pseudoatoms.get(self.id).unwrap()
    }

    pub fn symbol(&self) -> &str {
        &self.core().symbol
    }

    pub fn bonds(&self) -> &[BondId] {
        &self.core().bonds
    }
}

pub struct PseudoatomViewMut<'a, M: MolMap> {
    pub molmap: &'a mut M,
    pub id: PseudoatomId,
}

impl<'a, M: MolMap> From<PseudoatomViewMut<'a, M>> for PseudoatomId {
    fn from(view: PseudoatomViewMut<'a, M>) -> Self {
        view.id
    }
}

impl<'a, M: MolMap> PseudoatomViewMut<'a, M> {
    fn as_ref(&self) -> PseudoatomView<'_, M> {
        PseudoatomView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    fn core(&mut self) -> &mut Pseudoatom {
        self.molmap.core_mut().pseudoatoms.get_mut(self.id).unwrap()
    }
}
