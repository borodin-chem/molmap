// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use slotmap::new_key_type;

// Create all the Id types
new_key_type! {
    pub struct AtomId;
}
new_key_type! {
    pub struct PseudoatomId;
}
new_key_type! {
    pub struct BondId;
}
new_key_type! {
    pub struct FragmentId;
}
new_key_type! {
    pub struct MoleculeId;
}
//new_key_type! {
//    pub struct ObjectId;
//}

// We use enums, not traits, to classify entities and narrow functionality

/// All the members of `MolMap`s that have corresponding ID types.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Entity {
    Atom(AtomId),
    Pseudoatom(PseudoatomId),
    Bond(BondId),
    //Object(ObjectId),
    Fragment(FragmentId),
    Molecule(MoleculeId),
}

/// Things that can form bonds.
///
/// The actual entities that bonds connect are represented by `BondingPartner`,
/// which is more restrictive.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Bondable {
    Atom(AtomId),
    Pseudoatom(PseudoatomId),
    //Bond(BondId),
    Fragment(FragmentId),
}

impl From<Bondable> for Entity {
    fn from(bondable: Bondable) -> Self {
        match bondable {
            Bondable::Atom(id) => Entity::Atom(id),
            Bondable::Pseudoatom(id) => Entity::Pseudoatom(id),
            //Bondable::Bond(id) => Entity::Bond(id),
            Bondable::Fragment(id) => Entity::Fragment(id),
        }
    }
}

/// The endpoints of bonds.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BondingPartner {
    Atom(AtomId),
    Pseudoatom(PseudoatomId),
    // BondingSystem(BondingSystemId),  // future
    AmbiguouslyBondingFragment(FragmentId),
}

impl From<BondingPartner> for Entity {
    fn from(partner: BondingPartner) -> Self {
        match partner {
            BondingPartner::Atom(id) => Entity::Atom(id),
            BondingPartner::Pseudoatom(id) => Entity::Pseudoatom(id),
            BondingPartner::AmbiguouslyBondingFragment(id) => Entity::Fragment(id),
        }
    }
}

/// Atoms, and things that need to behave like atoms.
///
/// These are the true nodes of the molecular graph.
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

impl From<Atomlike> for BondingPartner {
    fn from(atomlike: Atomlike) -> Self {
        match atomlike {
            Atomlike::Atom(id) => BondingPartner::Atom(id),
            Atomlike::Pseudoatom(id) => BondingPartner::Pseudoatom(id),
        }
    }
}

impl From<Atomlike> for Fundamental {
    fn from(atomlike: Atomlike) -> Self {
        match atomlike {
            Atomlike::Atom(id) => Fundamental::Atom(id),
            Atomlike::Pseudoatom(id) => Fundamental::Pseudoatom(id),
        }
    }
}

/// The basic building blocks of a `MolMap` that do not group other entities.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Fundamental {
    Atom(AtomId),
    Pseudoatom(PseudoatomId),
    Bond(BondId),
}

impl From<Fundamental> for Entity {
    fn from(fundamental: Fundamental) -> Self {
        match fundamental {
            Fundamental::Atom(id) => Entity::Atom(id),
            Fundamental::Pseudoatom(id) => Entity::Pseudoatom(id),
            Fundamental::Bond(id) => Entity::Bond(id),
        }
    }
}

/// Aggregations of `Fundamental` entities.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Collection {
    Fragment(FragmentId),
    Molecule(MoleculeId),
}

impl From<Collection> for Entity {
    fn from(collection: Collection) -> Self {
        match collection {
            Collection::Fragment(id) => Entity::Fragment(id),
            Collection::Molecule(id) => Entity::Molecule(id),
        }
    }
}

/// Entities that an `Object` can be attached to.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Anchor {
    Atom(AtomId),
    Pseudoatom(PseudoatomId),
    Bond(BondId),
    Fragment(FragmentId),
    Molecule(MoleculeId),
}

impl From<Anchor> for Entity {
    fn from(anchor: Anchor) -> Self {
        match anchor {
            Anchor::Atom(id) => Entity::Atom(id),
            Anchor::Pseudoatom(id) => Entity::Pseudoatom(id),
            Anchor::Bond(id) => Entity::Bond(id),
            Anchor::Fragment(id) => Entity::Fragment(id),
            Anchor::Molecule(id) => Entity::Molecule(id),
        }
    }
}

#[derive(Debug)]
pub struct IdError;

impl std::error::Error for IdError {}

impl std::fmt::Display for IdError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "The Id was not found in the Map")
    }
}
