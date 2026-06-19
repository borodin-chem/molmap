// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// The entity ID types are defined in the corresponding entity modules; we
// re-export them publicly here

pub use crate::entities::atom::AtomId;
pub use crate::entities::bond::BondId;
pub use crate::entities::molecule::MoleculeId;
pub use crate::entities::pseudoatom::PseudoatomId;
pub use crate::entities::substituent::SubstituentId;

// We use composite enums of IDs, not traits, to classify entities and narrow
// functionality

/// An ID of an atom or something that behaves like one (a pseudoatom).
///
/// Atomlikes are the true nodes of the molecular graph.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AtomlikeId {
    Atom(AtomId),
    Pseudoatom(PseudoatomId),
}

impl From<AtomId> for AtomlikeId {
    fn from(id: AtomId) -> Self {
        AtomlikeId::Atom(id)
    }
}

impl From<PseudoatomId> for AtomlikeId {
    fn from(id: PseudoatomId) -> Self {
        AtomlikeId::Pseudoatom(id)
    }
}

/// An ID of a fundamental entity, an entity that does not group other entities.
///
/// Fundamentals are the basic building blocks of a `MolMap`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FundamentalId {
    Atom(AtomId),
    Pseudoatom(PseudoatomId),
    Bond(BondId),
}

impl From<AtomId> for FundamentalId {
    fn from(id: AtomId) -> Self {
        FundamentalId::Atom(id)
    }
}

impl From<PseudoatomId> for FundamentalId {
    fn from(id: PseudoatomId) -> Self {
        FundamentalId::Pseudoatom(id)
    }
}

impl From<BondId> for FundamentalId {
    fn from(id: BondId) -> Self {
        FundamentalId::Bond(id)
    }
}

impl From<AtomlikeId> for FundamentalId {
    fn from(atomlike: AtomlikeId) -> Self {
        match atomlike {
            AtomlikeId::Atom(id) => FundamentalId::Atom(id),
            AtomlikeId::Pseudoatom(id) => FundamentalId::Pseudoatom(id),
        }
    }
}

/// An ID of a collection, an aggregation of fundamental entities.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CollectionId {
    Substituent(SubstituentId),
    Molecule(MoleculeId),
}

impl From<SubstituentId> for CollectionId {
    fn from(id: SubstituentId) -> Self {
        CollectionId::Substituent(id)
    }
}

impl From<MoleculeId> for CollectionId {
    fn from(id: MoleculeId) -> Self {
        CollectionId::Molecule(id)
    }
}

/// An ID of an entity that can form bonds.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BondableId {
    Atom(AtomId),
    Pseudoatom(PseudoatomId),
    //Bond(BondId),
}

impl From<AtomId> for BondableId {
    fn from(id: AtomId) -> Self {
        BondableId::Atom(id)
    }
}

impl From<PseudoatomId> for BondableId {
    fn from(id: PseudoatomId) -> Self {
        BondableId::Pseudoatom(id)
    }
}

impl From<AtomlikeId> for BondableId {
    fn from(atomlike: AtomlikeId) -> Self {
        match atomlike {
            AtomlikeId::Atom(id) => BondableId::Atom(id),
            AtomlikeId::Pseudoatom(id) => BondableId::Pseudoatom(id),
        }
    }
}

/// An ID of an entity that an `Object` can be attached to.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AnchorId {
    Atom(AtomId),
    Pseudoatom(PseudoatomId),
    Bond(BondId),
    Substituent(SubstituentId),
    Molecule(MoleculeId),
}

impl From<AtomId> for AnchorId {
    fn from(id: AtomId) -> Self {
        AnchorId::Atom(id)
    }
}

impl From<PseudoatomId> for AnchorId {
    fn from(id: PseudoatomId) -> Self {
        AnchorId::Pseudoatom(id)
    }
}

impl From<BondId> for AnchorId {
    fn from(id: BondId) -> Self {
        AnchorId::Bond(id)
    }
}

impl From<SubstituentId> for AnchorId {
    fn from(id: SubstituentId) -> Self {
        AnchorId::Substituent(id)
    }
}

impl From<MoleculeId> for AnchorId {
    fn from(id: MoleculeId) -> Self {
        AnchorId::Molecule(id)
    }
}

/// An ID of any kind of entity.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EntityId {
    Atom(AtomId),
    Pseudoatom(PseudoatomId),
    Bond(BondId),
    Substituent(SubstituentId),
    Molecule(MoleculeId),
}

impl From<AtomId> for EntityId {
    fn from(id: AtomId) -> Self {
        EntityId::Atom(id)
    }
}

impl From<PseudoatomId> for EntityId {
    fn from(id: PseudoatomId) -> Self {
        EntityId::Pseudoatom(id)
    }
}

impl From<BondId> for EntityId {
    fn from(id: BondId) -> Self {
        EntityId::Bond(id)
    }
}

impl From<SubstituentId> for EntityId {
    fn from(id: SubstituentId) -> Self {
        EntityId::Substituent(id)
    }
}

impl From<MoleculeId> for EntityId {
    fn from(id: MoleculeId) -> Self {
        EntityId::Molecule(id)
    }
}

impl From<AtomlikeId> for EntityId {
    fn from(atomlike: AtomlikeId) -> Self {
        match atomlike {
            AtomlikeId::Atom(id) => EntityId::Atom(id),
            AtomlikeId::Pseudoatom(id) => EntityId::Pseudoatom(id),
        }
    }
}

impl From<FundamentalId> for EntityId {
    fn from(fundamental: FundamentalId) -> Self {
        match fundamental {
            FundamentalId::Atom(id) => EntityId::Atom(id),
            FundamentalId::Pseudoatom(id) => EntityId::Pseudoatom(id),
            FundamentalId::Bond(id) => EntityId::Bond(id),
        }
    }
}

impl From<CollectionId> for EntityId {
    fn from(collection: CollectionId) -> Self {
        match collection {
            CollectionId::Substituent(id) => EntityId::Substituent(id),
            CollectionId::Molecule(id) => EntityId::Molecule(id),
        }
    }
}

impl From<BondableId> for EntityId {
    fn from(bondable: BondableId) -> Self {
        match bondable {
            BondableId::Atom(id) => EntityId::Atom(id),
            BondableId::Pseudoatom(id) => EntityId::Pseudoatom(id),
            //Bondable::Bond(id) => Entity::Bond(id),
        }
    }
}

impl From<AnchorId> for EntityId {
    fn from(anchor: AnchorId) -> Self {
        match anchor {
            AnchorId::Atom(id) => EntityId::Atom(id),
            AnchorId::Pseudoatom(id) => EntityId::Pseudoatom(id),
            AnchorId::Bond(id) => EntityId::Bond(id),
            AnchorId::Substituent(id) => EntityId::Substituent(id),
            AnchorId::Molecule(id) => EntityId::Molecule(id),
        }
    }
}
