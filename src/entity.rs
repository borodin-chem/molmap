// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{MolMapError, MolMapResult};

/// A constituent member of a [`MolMap`] with associated data and relationships
/// to other entities.
///
/// The usage of the term "entity" in `molmap` only in some cases aligns with the
/// concept of a _molecular entity_:
///
/// > Any constitutionally or isotopically distinct atom, molecule, ion, ion pair,
/// > radical, radical ion, complex, conformer etc., identifiable as a separately
/// > distinguishable entity.
/// >
/// > [_'molecular entity' in IUPAC Compendium of Chemical Terminology, 5th ed. International Union of Pure and Applied Chemistry; 2025._](https://doi.org/10.1351/goldbook.M03986)
///
/// In other cases an entity in the `molmap` sense may be a part of another (such
/// as a repeating unit), or a grouping of other entities (such as a cluster), or
/// something that is not a physical object at all but rather a conceptual one (such
/// as a bond or a charge).
///
/// All entities have a unique ID.
///
/// This trait should not be implemented outside of the crate.
pub trait Entity: Copy + Clone {}

/// The kind of an entity.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(u8)]
//#[non_exhaustive]
pub enum EntityKind {
    Atom = 0x00,
    Bond = 0x01,
    Pseudoatom = 0x02,
    Substituent = 0x10,
    Molecule = 0x1F,
}

impl From<EntityKind> for u8 {
    #[inline]
    fn from(kind: EntityKind) -> Self {
        kind as u8
    }
}

impl TryFrom<u8> for EntityKind {
    type Error = MolMapError;

    fn try_from(value: u8) -> MolMapResult<Self> {
        match value {
            0x00 => Ok(Self::Atom),
            0x01 => Ok(Self::Bond),
            0x02 => Ok(Self::Pseudoatom),
            0x10 => Ok(Self::Substituent),
            0x1F => Ok(Self::Molecule),
            _ => Err(MolMapError::UnknownEntityKind(value)),
        }
    }
}

/// A fundamental kind of entity in a [`MolMap`], with a backing `SlotMap`.
///
/// This trait should not be implemented outside of the crate.
pub trait KeyEntity: Entity + Copy + Clone {
    /// Returns the corresponding kind of the entity.
    fn kind() -> EntityKind;
}
