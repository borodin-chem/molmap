// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{AtomId, BondId, FragmentId, Fundamental, MolMap, MoleculeId, PseudoatomId};

#[derive(Debug)]
pub(crate) struct Molecule {
    pub(crate) members: Vec<Fundamental>,
    //pub annotations: Vec<ObjectId>,
}

impl Molecule {
    pub(crate) fn new() -> Self {
        Self {
            members: Vec::new(),
            //annotations: Vec::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct MoleculeView<'a, M: MolMap> {
    pub molmap: &'a M,
    pub id: MoleculeId,
}

impl<'a, M: MolMap> From<MoleculeView<'a, M>> for MoleculeId {
    fn from(view: MoleculeView<'a, M>) -> Self {
        view.id
    }
}

impl<'a, M: MolMap> MoleculeView<'a, M> {
    fn core(&self) -> &'a Molecule {
        self.molmap.core().molecules.get(self.id).unwrap()
    }
}

pub struct MoleculeViewMut<'a, M: MolMap> {
    pub molmap: &'a mut M,
    pub id: MoleculeId,
}

impl<'a, M: MolMap> From<MoleculeViewMut<'a, M>> for MoleculeId {
    fn from(view: MoleculeViewMut<'a, M>) -> Self {
        view.id
    }
}

impl<'a, M: MolMap> MoleculeViewMut<'a, M> {
    fn as_ref(&self) -> MoleculeView<'_, M> {
        MoleculeView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    fn inner(mut self) -> &'a mut Molecule {
        self.molmap.core_mut().molecules.get_mut(self.id).unwrap()
    }
}
