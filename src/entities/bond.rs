// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use slotmap::new_key_type;

use crate::{
    ids::{AtomId, AtomlikeId, BondableId, PseudoatomId, SubstituentId},
    traits::MolMap,
};

new_key_type! {
    /// An ID corresponding to a specific bond entity in a `MolMap`.
    pub struct BondId;
}

/// The type of a bond e.g. covalent, ionic.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum BondType {
    Covalent,
    Intermolecular,
    Coordination,
    Ionic,
}

/// The core data of a bond entity.
#[derive(Debug)]
pub(crate) struct Bond {
    pub(crate) bond_type: BondType,
    pub(crate) order: f32,
    pub(crate) start: BondableId,
    pub(crate) end: BondableId,
}

impl Bond {
    pub(crate) fn new(bond_type: BondType, order: f32, start: BondableId, end: BondableId) -> Self {
        Self {
            bond_type,
            order,
            start,
            end,
        }
    }
}

/// An immutable view over a specific bond entity in a specific `MolMap`.
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
    /// Returns the corresponding data from the core `MolGraph`.
    fn core(&self) -> &'a Bond {
        self.molmap.core().bonds.get(self.id).unwrap()
    }

    pub fn bond_type(&self) -> BondType {
        self.core().bond_type
    }

    pub fn order(&self) -> f32 {
        self.core().order
    }

    pub fn partners(&self) -> [BondableId; 2] {
        let inner = self.core();
        [inner.start, inner.end]
    }
}

/// A mutable view over a specific bond entity in a specific `MolMap`.
///
/// Note that the bonding partners of a bond cannot be changed; the bond must be
/// removed and a new one added between the desired new bonding partners.
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
    /// Returns the corresponding data from the core `MolGraph`.
    fn core(&mut self) -> &mut Bond {
        self.molmap.core_mut().bonds.get_mut(self.id).unwrap()
    }

    /// Returns an immutable view over the same bond.
    fn as_ref(&self) -> BondView<'_, M> {
        BondView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    // Public methods, which should consume the view

    /// Set the type of the bond without any additional effects.
    pub fn set_bond_type(mut self, bond_type: BondType) {
        self.core().bond_type = bond_type;
    }

    /// Set the order of the bond without any additional effects.
    pub fn set_bond_order(mut self, order: f32) {
        self.core().order = order;
    }

    /// Removes the bond from the map (but not its bonding partners).
    pub fn delete(mut self) {
        self.molmap.core_mut().delete_bond(self.id);
    }
}
