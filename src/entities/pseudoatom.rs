// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{BondId, FragmentId, MolMap, MolMapExt, PseudoatomId};

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
pub struct PseudoatomView<'a, E: MolMapExt> {
    pub molmap: &'a MolMap<E>,
    pub id: PseudoatomId,
}

impl<'a, E: MolMapExt> From<PseudoatomView<'a, E>> for PseudoatomId {
    fn from(view: PseudoatomView<'a, E>) -> Self {
        view.id
    }
}

impl<'a, E: MolMapExt> PseudoatomView<'a, E> {
    fn inner(&self) -> &'a Pseudoatom {
        self.molmap.pseudoatoms.get(self.id).unwrap()
    }

    pub fn symbol(&self) -> &str {
        &self.inner().symbol
    }

    pub fn bonds(&self) -> &[BondId] {
        &self.inner().bonds
    }
}

pub struct PseudoatomViewMut<'a, E: MolMapExt> {
    pub molmap: &'a mut MolMap<E>,
    pub id: PseudoatomId,
}

impl<'a, E: MolMapExt> From<PseudoatomViewMut<'a, E>> for PseudoatomId {
    fn from(view: PseudoatomViewMut<'a, E>) -> Self {
        view.id
    }
}

impl<'a, E: MolMapExt> PseudoatomViewMut<'a, E> {
    fn as_ref(&self) -> PseudoatomView<'_, E> {
        PseudoatomView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    fn inner(&mut self) -> &mut Pseudoatom {
        self.molmap.pseudoatoms.get_mut(self.id).unwrap()
    }
}
