// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{BondId, Bondable, BondingPartner, MolMap};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum BondType {
    Covalent,
    Intermolecular,
    Coordination,
    Ionic,
}

#[derive(Debug)]
pub(crate) struct Bond {
    pub(crate) bond_type: BondType,
    pub(crate) order: f32,
    pub(crate) start: BondingPartner,
    pub(crate) end: BondingPartner,
}

impl Bond {
    pub(crate) fn new(
        bond_type: BondType,
        order: f32,
        start: BondingPartner,
        end: BondingPartner,
    ) -> Self {
        Self {
            bond_type,
            order,
            start,
            end,
        }
    }
}

#[derive(Clone, Copy)]
pub struct BondView<'a, M: MolMap> {
    pub molmap: &'a M,
    pub id: BondId,
}

impl<'a, M: MolMap> From<BondView<'a, M>> for BondId {
    fn from(view: BondView<'a, M>) -> Self {
        view.id
    }
}

impl<'a, M: MolMap> BondView<'a, M> {
    fn core(&self) -> &'a Bond {
        self.molmap.core().bonds.get(self.id).unwrap()
    }

    pub fn bond_type(&self) -> BondType {
        self.core().bond_type
    }

    pub fn order(&self) -> f32 {
        self.core().order
    }

    pub fn partners(&self) -> [BondingPartner; 2] {
        let inner = self.core();
        [inner.start, inner.end]
    }
}

pub struct BondViewMut<'a, M: MolMap> {
    pub molmap: &'a mut M,
    pub id: BondId,
}

impl<'a, M: MolMap> From<BondViewMut<'a, M>> for BondId {
    fn from(view: BondViewMut<'a, M>) -> Self {
        view.id
    }
}

impl<'a, M: MolMap> BondViewMut<'a, M> {
    fn as_ref(&self) -> BondView<'_, M> {
        BondView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    fn core(&mut self) -> &mut Bond {
        self.molmap.core_mut().bonds.get_mut(self.id).unwrap()
    }
}
