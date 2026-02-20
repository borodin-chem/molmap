// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use nalgebra as na;
use slotmap::SlotMap;

use std::hash::RandomState;

use crate::{bond::BondType, entities::*, id::*, Element};

pub struct MolMap {
    pub bonds: SlotMap<BondId, Bond>,
    pub atoms: SlotMap<AtomId, Atom>,
    pub pseudoatoms: SlotMap<PseudoatomId, Pseudoatom>,
    pub nodes: SlotMap<SubstituentId, Substituent>,
    pub molecules: SlotMap<MoleculeId, Molecule>,
    pub objects: SlotMap<ObjectId, Object>,
}

// Loading from file involves a lot of insertions and therefore if the initial capacity was 0
// multiple expensive reallocations would occur every time the slotmaps filled up, so try to improve
// performance by pre-allocating a sensible amount of space (say enough for a well-populated scheme
// of A4 size) for each slotmap

impl Default for MolMap {
    fn default() -> Self {
        Self {
            bonds: SlotMap::with_capacity_and_key(500),
            atoms: SlotMap::with_capacity_and_key(500),
            pseudoatoms: SlotMap::with_capacity_and_key(500),
            nodes: SlotMap::with_capacity_and_key(1000),
            molecules: SlotMap::with_capacity_and_key(50),
            objects: SlotMap::with_capacity_and_key(100),
        }
    }
}

impl MolMap {
    pub fn new() -> Self {
        Self::default()
    }
}
