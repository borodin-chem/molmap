// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::*;

/// A chemical bond: an attraction between molecular entities.
///
/// > There is a chemical bond between two atoms or groups of atoms in the case
/// > that the forces acting between them are such as to lead to the formation
/// > of an aggregate with sufficient stability to make it convenient for the
/// > chemist to consider it as an independent 'molecular species'.
/// >
/// > [_'bond' in IUPAC Compendium of Chemical Terminology, 5th ed. International Union of Pure and Applied Chemistry; 2025._](https://doi.org/10.1351/goldbook.B00697)
#[derive(Copy, Clone, Debug)]
pub struct Bond;

impl Entity for Bond {}

impl KeyEntity for Bond {
    fn kind() -> EntityKind {
        EntityKind::Bond
    }
}

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
    pub(crate) start: Id<Bondable>,
    pub(crate) end: Id<Bondable>,
}

impl BondData {
    pub fn new(bond_type: BondType, order: f32, start: Id<Bondable>, end: Id<Bondable>) -> Self {
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
        self.map.core().bonds.get(self.id).unwrap()
    }

    pub fn bond_type(&self) -> BondType {
        self.core().bond_type
    }

    pub fn order(&self) -> f32 {
        self.core().order
    }

    pub fn partners(&self) -> [Id<Bondable>; 2] {
        let inner = self.core();
        [inner.start, inner.end]
    }
}
