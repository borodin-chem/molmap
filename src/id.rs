// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// New implementation using SlotMap
use slotmap::new_key_type;

// Create all the Id types
new_key_type! {
    pub struct BondId;
}
new_key_type! {
    pub struct AtomId;
}
new_key_type! {
    pub struct PseudoatomId;
}
new_key_type! {
    pub struct SubstituentId;
}
new_key_type! {
    pub struct MoleculeId;
}
new_key_type! {
    pub struct ObjectId;
}

// Instead of using traits we narrow functionality using enums
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Entity {
    Bond(BondId),
    Atom(AtomId),
    Pseudoatom(PseudoatomId),
    Substituent(SubstituentId),
    Molecule(MoleculeId),
    Object(ObjectId),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Bondable {
    Bond(BondId),
    Atom(AtomId),
    Pseudoatom(PseudoatomId),
    Node(SubstituentId),
}

impl From<Bondable> for Entity {
    fn from(bondable: Bondable) -> Self {
        match bondable {
            Bondable::Bond(id) => Entity::Bond(id),
            Bondable::Atom(id) => Entity::Atom(id),
            Bondable::Pseudoatom(id) => Entity::Pseudoatom(id),
            Bondable::Node(id) => Entity::Substituent(id),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Atomlike {
    Atom(AtomId),
    Pseudoatom(PseudoatomId),
}

impl From<Atomlike> for Entity {
    fn from(atomlike: Atomlike) -> Self {
        match atomlike {
            Atomlike::Atom(id) => Entity::Atom(id),
            Atomlike::Pseudoatom(id) => Entity::Pseudoatom(id),
        }
    }
}

impl From<Atomlike> for Bondable {
    fn from(atomlike: Atomlike) -> Self {
        match atomlike {
            Atomlike::Atom(id) => Bondable::Atom(id),
            Atomlike::Pseudoatom(id) => Bondable::Pseudoatom(id),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Collection {
    Molecule(MoleculeId),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Annotatable {
    Bond(BondId),
    Atom(AtomId),
    Pseudoatom(PseudoatomId),
    Substituent(SubstituentId),
    Molecule(MoleculeId),
}

#[derive(Debug)]
pub struct IdError;

impl std::error::Error for IdError {}

impl std::fmt::Display for IdError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "The Id was not found in the Map")
    }
}
