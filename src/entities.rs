// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Definitions of the basic kinds of entity.

use std::iter::FusedIterator;

use slotmap::{Key, SlotMap, new_key_type};

use crate::{error::*, id::Id};

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

impl EntityKind {
    /// Returns the corresponding variant if `value` is a valid discriminant, or
    /// `None` otherwise.
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Atom),
            0x01 => Some(Self::Bond),
            0x02 => Some(Self::Pseudoatom),
            0x10 => Some(Self::Substituent),
            0x1F => Some(Self::Molecule),
            _ => None,
        }
    }
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
        Self::from_u8(value).ok_or(MolMapError::UnknownEntityKind(value))
    }
}

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
/// This trait is sealed and cannot be implemented outside of the crate.
pub trait Entity: Copy + Clone + Eq {
    // It is important that this trait remains sealed! Any kinds of entity that
    // molmap doesn't know about will lead to problems. It being sealed is also
    // relied upon by traits that have this as a supertrait e.g. the entity
    // category traits (Bondable, Atomlike etc.).
    //
    // What makes this trait sealed currently is the fact that Id is not
    // nameable by other crates, so foreign types cannot implement new_unchecked
    // or into_inner and therefore cannot implement the trait. It is therefore
    // crucial that that remains the case i.e. the id module remains private and
    // Id is not publicly re-exported anywhere.
    //
    // It's also very important that downstream code cannot create an entity of
    // a specific kind from a generic Id without the discriminant being
    // checked, so it is *essential* that new_unchecked not just cannot be
    // *implemented* but also cannot be *called*. As long as Id stays
    // unnameable, this is the case.
    //
    // It is fine for into_inner to be callable downstream and for an Id
    // to be obtained, as long as nothing can be done with that Id.
    /// Creates a new ID for the requested kind of entity without checking that
    /// the discriminant of the ID is correct for that kind.
    fn new_unchecked(id: Id) -> Self;

    fn into_inner(self) -> Id;

    /// Returns the corresponding kind of the entity.
    fn kind(&self) -> EntityKind {
        self.into_inner().kind()
    }

    fn as_entity(self) -> AnyEntity {
        AnyEntity(self.into_inner())
    }

    /// Returns the specific entity type wrapped in an enum variant, for exhaustive matching.
    fn as_tagged_entity(self) -> TaggedEntity {
        match self.kind() {
            EntityKind::Atom => TaggedEntity::Atom(Atom::new_unchecked(self.into_inner())),
            EntityKind::Bond => TaggedEntity::Bond(Bond::new_unchecked(self.into_inner())),
            EntityKind::Pseudoatom => {
                TaggedEntity::Pseudoatom(Pseudoatom::new_unchecked(self.into_inner()))
            }
            EntityKind::Substituent => {
                TaggedEntity::Substituent(Substituent::new_unchecked(self.into_inner()))
            }
            EntityKind::Molecule => {
                TaggedEntity::Molecule(Molecule::new_unchecked(self.into_inner()))
            }
        }
    }
}

/// An entity that may be of any kind.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct AnyEntity(pub(crate) Id);

impl Entity for AnyEntity {
    fn new_unchecked(id: Id) -> Self {
        Self(id)
    }

    fn into_inner(self) -> Id {
        self.0
    }
}

/// An entity of a kind, tagged to show which specific kind.
///
/// Matching on this enum is exhaustive for all the possible kinds of entity.
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
//#[non_exhaustive]
pub enum TaggedEntity {
    Atom(Atom) = EntityKind::Atom as u8,
    Bond(Bond) = EntityKind::Bond as u8,
    Pseudoatom(Pseudoatom) = EntityKind::Pseudoatom as u8,
    Substituent(Substituent) = EntityKind::Substituent as u8,
    Molecule(Molecule) = EntityKind::Molecule as u8,
}

macro_rules! new_keyed_entity {
    (
        $(#[$doc:meta])*
        $vis:vis struct $kind:ident;
    ) => {
        paste::paste! {
            $(#[$doc])*
            #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
            pub struct $kind(pub(crate) Id);

            impl Entity for $kind {
                fn new_unchecked(id: Id) -> Self {
                    Self(id)
                }

                fn into_inner(self) -> Id {
                    self.0
                }

                fn kind(&self) -> EntityKind {
                    EntityKind::$kind
                }

                #[inline]
                fn as_tagged_entity(self) -> TaggedEntity {
                    TaggedEntity::$kind($kind::new_unchecked(self.into_inner()))
                }
            }
        }
    };
}

new_keyed_entity!(
    /// Smallest particle still characterizing a chemical element.
    pub struct Atom;
);

new_keyed_entity!(
    /// A chemical bond: an attraction between molecular entities.
    ///
    /// > There is a chemical bond between two atoms or groups of atoms in the case
    /// > that the forces acting between them are such as to lead to the formation
    /// > of an aggregate with sufficient stability to make it convenient for the
    /// > chemist to consider it as an independent 'molecular species'.
    /// >
    /// > [_'bond' in IUPAC Compendium of Chemical Terminology, 5th ed. International Union of Pure and Applied Chemistry; 2025._](https://doi.org/10.1351/goldbook.B00697)
    pub struct Bond;
);

new_keyed_entity!(
    /// A pseudoatom: something that forms bonds and can be represented by an
    /// "element symbol" like a normal atom but represents something else.
    ///
    /// It may have an unknown composition like R, or a known structure like Ph.
    pub struct Pseudoatom;
);

new_keyed_entity!(
    /// A substituent: a group of atoms, bonded internally, identified as a unit and
    /// usually part of a larger molecule. Often synonymous with "functional group" or
    /// "moiety".
    ///
    /// Substituents are the smallest collections in a `MolMap` and represent the units
    /// that chemists tend to actually think in terms of, rather than individual atoms.
    /// For example, a substituent may be conceptually equivalent to:
    /// - a non-hydrogen atom and "its" implicit hydrogen atoms in SMILES or in packages
    /// that work that way (all hydrogen atoms are explicit in a MolMap)
    /// - the carbon atom and hydrogen atoms at a vertex in a skeletal formula
    /// - atoms drawn together as a group without explicit bonds in a skeletal formula
    ///   e.g. –OH, –COOH, –CH₃
    ///
    /// Substituents generally indicate one or more centres, so that bonds can be made
    /// "to" the centre. This allows molecules to be built up conveniently by adding and
    /// connecting substituents rather than individual atoms.
    pub struct Substituent;
);

new_keyed_entity!(
    /// A molecule: a discrete group of atoms held together by chemical bonds.
    ///
    /// > An electrically neutral entity consisting of more than one atom (_n_ > 1).
    /// > Rigorously, a molecule, in which n > 1 must correspond to a depression on the
    /// > potential energy surface that is deep enough to confine at least one
    /// > vibrational state.
    /// >
    /// > [_'molecule' in IUPAC Compendium of Chemical Terminology, 5th ed. International Union of Pure and Applied Chemistry; 2025._](https://doi.org/10.1351/goldbook.M04002)
    ///
    /// This definition from the IUPAC Gold Book restricts the meaning of "molecule" to
    /// electrically neutral species, but here, the typical practice is followed and no
    /// distinction is made based on charge.
    ///
    /// Note that the constituent atoms of a molecule are not actually required to be
    /// joined by bonds, and it is also not required that all bonds are covalent. The
    /// molecule need not have any bonds at all, or indeed any atoms (an empty molecule
    /// is also permitted). Do not rely on any of these things being true.
    pub struct Molecule;
);

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use slotmap::{DefaultKey, KeyData, SlotMap, new_key_type};

    use crate::graph::keys::BondKey;

    use super::*;

    // Some raw keys for use when testing, so that they only need updating in
    // one place if anything changes
    const ATOM_NULL_RAW: u64 = 0x0000001_00_FFFFFF; // The null atom key
    const KD_NULL_RAW: u64 = 0x0000001_FF_FFFFFF; // What KeyData considers to be null

    const BOND_RAW: NonZeroU64 = NonZeroU64::new(0x1_01_000008).unwrap(); // version: 1, kind: Bond, idx: 8
    const ATOM_RAW: NonZeroU64 = NonZeroU64::new(0x3_00_000010).unwrap(); // version: 3, kind: Atom, idx: 16, (version always odd for occupied slots)
    const PSEUDOATOM_RAW: NonZeroU64 = NonZeroU64::new(0x1_03_00000A).unwrap(); // version: 1, kind: Pseudoatom, idx: 10
    const MOL_RAW: NonZeroU64 = NonZeroU64::new(0x1_1F_000001).unwrap(); // version: 1, kind: Molecule, idx: 1

    const BOND: Bond = Bond(Id(BOND_RAW));
    const ATOM: Atom = Atom(Id(ATOM_RAW));
    const PSEUDOATOM: Pseudoatom = Pseudoatom(Id(PSEUDOATOM_RAW));
    const MOL: Molecule = Molecule(Id(MOL_RAW));

    #[test]
    fn key_kind() {
        let atom = ATOM;
        let bond = BOND;
        let mol = MOL;
        assert_eq!(atom.kind(), EntityKind::Atom);
        assert_eq!(bond.kind(), EntityKind::Bond);
        assert_eq!(mol.kind(), EntityKind::Molecule);
    }

    #[test]
    fn key_id_slotmap_access() {
        let mut sm: SlotMap<BondKey, usize> = SlotMap::with_key();
        let first = sm.insert(21); // idx: 1, version: 1
        assert_eq!(sm.get(first), Some(&21));
        let second = sm.insert(22); // idx: 2, version: 1
        assert_eq!(sm.get(second), Some(&22));
        sm.remove(first); // idx 1 now free
        let third = sm.insert(23); // idx: 1, version: 3 (version always odd for a filled slot)
        assert_eq!(sm.get(third), Some(&23));
        // Removed key should be invalid
        assert!(sm.get(first).is_none());
    }
}
