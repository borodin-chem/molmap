// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The data held by an `AtomMap` for the fundamental entities.

use crate::{
    bonding::BondType,
    elements::{Element, Pseudoelement},
    entities::*,
};

/// The core data of an atom entity.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AtomData {
    pub(crate) element: Element,
    pub(crate) bonds: Vec<Bond>,
}

impl AtomData {
    pub(crate) fn new(element: Element) -> Self {
        Self {
            element,
            bonds: Vec::new(),
        }
    }
}

impl Stored for Atom {
    type Data = AtomData;
}

/// The core data of a pseudoatom entity.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PseudoatomData {
    #[allow(unused)]
    pub(crate) pseudoelement: Pseudoelement,
    pub(crate) bonds: Vec<Bond>,
}

impl PseudoatomData {
    pub(crate) fn new(pseudoelement: Pseudoelement) -> Self {
        Self {
            pseudoelement,
            bonds: Vec::new(),
        }
    }
}

impl Stored for Pseudoatom {
    type Data = PseudoatomData;
}

/// The core data of a bond entity.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BondData {
    pub(crate) bond_type: BondType,
    pub(crate) start: AnyBondable,
    pub(crate) end: AnyBondable,
}

impl BondData {
    pub(crate) fn new(bond_type: BondType, start: AnyBondable, end: AnyBondable) -> Self {
        Self {
            bond_type,
            start,
            end,
        }
    }
}

impl Stored for Bond {
    type Data = BondData;
}
