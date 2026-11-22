// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The `Atomic` trait and methods universal to all `Atomic` maps (i.e. all maps).

use crate::{
    bonding::BondType,
    elements::Element,
    entities::*,
    traits::{Map, Stores},
    view::View,
};

use super::{AtomMap, data::*};

/// A map that holds fundamental atomic entities, and a network of bonds between them.
///
/// This applies to all maps, and so all map types in the crate implement this trait.
pub trait Atomic: Map {
    /// Returns the the core network of atoms and bonds without any spatial positions.
    fn into_atomic(self) -> AtomMap;

    /// Provides read-only access to the core network of atoms and bonds.
    fn as_atomic(&self) -> &AtomMap;
}

// --- View methods ---
// For all `Atomic` maps (i.e. all maps)

/// Methods on atom views common to all [`Atomic`] maps (i.e. all map types).
impl<'m, M> View<'m, M, Atom>
where
    M: Map + Atomic,
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

/// Methods on pseudoatom views common to all [`Atomic`] maps (i.e. all map types).
impl<'m, M> View<'m, M, Pseudoatom>
where
    M: Map + Atomic,
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

/// Methods on bond views common to all [`Atomic`] maps (i.e. all map types).
impl<'m, M> View<'m, M, Bond>
where
    M: Map + Atomic,
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
