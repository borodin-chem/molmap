// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{BondId, MolMap, MoleculeId, SubstituentId, ObjectId};

#[derive(Debug)]
pub struct Molecule {
    pub id: MoleculeId,
    pub nodes: Vec<SubstituentId>,
    pub bonds: Vec<BondId>,
    pub annotations: Vec<ObjectId>,
}

impl Molecule {
    pub fn new(id: MoleculeId, nodes: &[SubstituentId], bonds: &[BondId]) -> Self {
        Self {
            id,
            nodes: nodes.to_vec(),
            bonds: bonds.to_vec(),
            annotations: Vec::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct MoleculeView<'a> {
    pub molmap: &'a MolMap,
    pub id: MoleculeId,
}

impl<'a> From<MoleculeView<'a>> for MoleculeId {
    fn from(view: MoleculeView<'a>) -> Self {
        view.id
    }
}

impl<'a> MoleculeView<'a> {
    fn inner(&self) -> &'a Molecule {
        self.molmap.molecules.get(self.id).unwrap()
    }
}

pub struct MoleculeViewMut<'a> {
    pub molmap: &'a mut MolMap,
    pub id: MoleculeId,
}

impl<'a> From<MoleculeViewMut<'a>> for MoleculeId {
    fn from(view: MoleculeViewMut<'a>) -> Self {
        view.id
    }
}

impl<'a> MoleculeViewMut<'a> {
    fn as_ref(&self) -> MoleculeView<'_> {
        MoleculeView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    fn inner(mut self) -> &'a mut Molecule {
        self.molmap.molecules.get_mut(self.id).unwrap()
    }
}
