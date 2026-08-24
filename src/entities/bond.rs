// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::*;

/// The type of a bond e.g. covalent, ionic.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BondType {
    Covalent,
    Intermolecular,
    Coordination,
    Ionic,
}

/// The core data of a bond entity.
#[derive(Clone, Debug)]
pub(crate) struct BondData {
    pub(crate) bond_type: BondType,
    pub(crate) order: f32,
    pub(crate) start: AnyBondable,
    pub(crate) end: AnyBondable,
}

impl BondData {
    pub fn new(bond_type: BondType, order: f32, start: AnyBondable, end: AnyBondable) -> Self {
        Self {
            bond_type,
            order,
            start,
            end,
        }
    }
}

pub type BondView<'a, M> = View<'a, M, Bond>;

impl<'a, M: MolMap> View<'a, M, Bond> {
    fn core(&self) -> &BondData {
        self.map.core().bonds.get(self.id.into()).unwrap()
    }

    pub fn bond_type(&self) -> BondType {
        self.core().bond_type
    }

    pub fn order(&self) -> f32 {
        self.core().order
    }

    pub fn partners(&self) -> [impl Bondable; 2] {
        let inner = self.core();
        [inner.start, inner.end]
    }
}
