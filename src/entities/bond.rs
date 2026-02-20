// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{BondId, Bondable, MolMap, ObjectId};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum BondType {
    Covalent,
    Intermolecular,
    Coordination,
    Ionic,
}

#[derive(Debug)]
pub struct Bond {
    pub id: BondId,
    pub bond_type: BondType,
    pub order: f32,
}

impl Bond {
    pub fn new(
        id: BondId,
        bond_type: BondType,
        order: f32,
    ) -> Self {
        Self {
            id,
            bond_type,
            order,
        }
    }
}

#[derive(Clone, Copy)]
pub struct BondView<'a> {
    pub molmap: &'a MolMap,
    pub id: BondId,
}

impl<'a> From<BondView<'a>> for BondId {
    fn from(view: BondView<'a>) -> Self {
        view.id
    }
}

impl<'a> BondView<'a> {
    fn inner(&self) -> &'a Bond {
        self.molmap.bonds.get(self.id).unwrap()
    }

    pub fn order(&self) -> f32 {
        self.inner().order
    }
}

pub struct BondViewMut<'a> {
    pub molmap: &'a mut MolMap,
    pub id: BondId,
}

impl<'a> From<BondViewMut<'a>> for BondId {
    fn from(view: BondViewMut<'a>) -> Self {
        view.id
    }
}

impl<'a> BondViewMut<'a> {
    fn as_ref(&self) -> BondView<'_> {
        BondView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    fn inner(&mut self) -> &mut Bond {
        self.molmap.bonds.get_mut(self.id).unwrap()
    }
}
