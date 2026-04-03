// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{AtomId, BondId, FragmentId, Fundamental, MolMap, MolMapExt, MoleculeId, PseudoatomId};

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
pub struct MoleculeView<'a, E: MolMapExt> {
    pub molmap: &'a MolMap<E>,
    pub id: MoleculeId,
}

impl<'a, E: MolMapExt> From<MoleculeView<'a, E>> for MoleculeId {
    fn from(view: MoleculeView<'a, E>) -> Self {
        view.id
    }
}

impl<'a, E: MolMapExt> MoleculeView<'a, E> {
    fn inner(&self) -> &'a Molecule {
        self.molmap.molecules.get(self.id).unwrap()
    }
}

pub struct MoleculeViewMut<'a, E: MolMapExt> {
    pub molmap: &'a mut MolMap<E>,
    pub id: MoleculeId,
}

impl<'a, E: MolMapExt> From<MoleculeViewMut<'a, E>> for MoleculeId {
    fn from(view: MoleculeViewMut<'a, E>) -> Self {
        view.id
    }
}

impl<'a, E: MolMapExt> MoleculeViewMut<'a, E> {
    fn as_ref(&self) -> MoleculeView<'_, E> {
        MoleculeView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    fn inner(mut self) -> &'a mut Molecule {
        self.molmap.molecules.get_mut(self.id).unwrap()
    }
}
