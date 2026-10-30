// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! View methods universal to all maps.

use crate::{Element, Map, View, entities::*, traits::Stores};

use super::data::*;

impl<'m, M> View<'m, M, Atom>
where
    M: Map,
{
    /// Returns a reference to the data struct in the core graph.
    fn data(&self) -> &AtomData {
        self.map
            .as_atomic()
            .data(self.id)
            .expect("Existence of view guarantees ID validity")
    }

    pub fn element(&self) -> Element {
        self.data().element
    }

    pub fn symbol(&self) -> &str {
        self.data().element.symbol()
    }

    pub fn bonds(&self) -> &[Bond] {
        &self.data().bonds
    }
}

impl<'m, M> View<'m, M, Pseudoatom>
where
    M: Map,
{
    /// Returns a reference to the data struct in the core graph.
    fn data(&self) -> &PseudoatomData {
        self.map
            .as_atomic()
            .data(self.id)
            .expect("Existence of view guarantees ID validity")
    }

    pub fn bonds(&self) -> &[Bond] {
        &self.data().bonds
    }
}

impl<'m, M> View<'m, M, Bond>
where
    M: Map,
{
    /// Returns a reference to the data struct in the core graph.
    fn data(&self) -> &BondData {
        self.map.as_atomic().data(self.id).unwrap()
    }

    pub fn bond_type(&self) -> BondType {
        self.data().bond_type
    }

    /// Returns `true` if the bond type is covalent/dipolar, ionic, metallic, or
    /// the `OtherStrong` variant.
    ///
    /// Note that this excludes hydrogen bonds and σ-hole interactions, even though
    /// these can in some cases have considerable strength.
    pub fn is_strong(&self) -> bool {
        self.bond_type().is_strong()
    }

    /// Returns `true` if the bond type is covalent or dipolar.
    pub fn is_covalent(&self) -> bool {
        self.bond_type().is_covalent()
    }

    /// Returns the order of a covalent bond, or `None` otherwise.
    ///
    /// A dipolar bond is considered covalent.
    pub fn order(&self) -> Option<f32> {
        match self.data().bond_type {
            BondType::Covalent { order } => Some(order),
            BondType::Dipolar { order } => Some(order),
            _ => None,
        }
    }

    pub fn partners(&self) -> [AnyBondable; 2] {
        let data = self.data();
        [data.start, data.end]
    }
}
