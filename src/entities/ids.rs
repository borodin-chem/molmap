// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt::{Debug, Formatter};
use std::num::{IntErrorKind, NonZeroU16, NonZeroU32, NonZeroU64};

use slotmap::{Key, KeyData};

use crate::{MolMapError, MolMapResult};

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

/// An ID for a single kind of entity or a set of kinds of entity.
///
/// Implementors are required to ensure that they convert to the correct variant of
/// [`EntityId`], and that only the correct variants convert to the `Id` type.
pub trait Id: Into<EntityId> + TryFrom<EntityId> {
    /// The tagged key form of the ID that wraps an actual key in an enum.
    ///
    /// The tagged key form is inherently larger than the ID form but allows convenient
    /// matching on the kind while holding the underlying key.
    type Tagged;

    /// Returns the tagged key form of the ID that wraps the actual key in an enum.
    ///
    /// The tagged key form is inherently larger than the ID form but allows convenient
    /// matching on the kind while holding the underlying key.
    fn to_tagged(self) -> Self::Tagged;

    /// Returns the kind of entity that the ID represents, amongst all possible
    /// kinds of entity.
    fn kind(&self) -> EntityKind;
}

/// A key ID as a tagged enum.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(u8)]
//#[non_exhaustive]
pub enum TaggedEntity {
    Atom(AtomId) = EntityKind::Atom as u8,
    Pseudoatom(PseudoatomId) = EntityKind::Pseudoatom as u8,
    Bond(BondId) = EntityKind::Bond as u8,
    Substituent(SubstituentId) = EntityKind::Substituent as u8,
    Molecule(MoleculeId) = EntityKind::Molecule as u8,
}

// We use composite "category" IDs, not traits, to classify entities and narrow
// functionality. The strategy originally pursued was to wrap the basic ID types
// in enums, but this makes the category ID types 12 bytes vs 8, and as they
// are widely used, that just means a lot of unnecessary memory use.
//
// Instead, we convert the SlotMap keys to the raw `u64` using the `as_ffi`
// method and use the high 8 bits of the index field to store a discriminant,
// and the whole thing is then the ID.
//
// The version of a SlotMap key wraps, so even if 2^31 deletion-addition cycles
// occur at the same slot, there will be no issues; the only situation in which
// a stale key will spuriously refer to something other than it originally did
// is if *exactly* 2^31 deletion-addition cycles have occurred at the same slot
// since the original key was generated and access is attempted with the stale
// original key. This is exceptionally unlikely.
//
// If the bits for the discriminant are stolen from the version field, the
// version has to be hard capped at the maximum value of the remaining bits i.e.
// 2^27, as anything higher will not survive a round trip.
//
// As such, it is better to take the bits from the index field, as that already
// has a hard cap (we just reduce it). This means the number of atoms is limited
// to 2^24, as is the number of bonds (or indeed any individual entity).
//
// As the discriminant is encoded by 8 bits, the number of different entity
// types is limited to 256. Only those in the range of 0 to 127 are used in the
// core `MolGraph`; the rest are left for use by extensions that wish to add
// additional entity types but still use unified IDs.

/// The ID of any kind of entity.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct EntityId(NonZeroU64);

// Layout:
// - Bits 63–32 = version (`version` field of `KeyData`)
// - Bits 31–24 = discriminant/kind
// - Bits 23–0  = index (`idx` field of `KeyData` truncated to 28 bits)

impl EntityId {
    const DISC_OFFSET: u64 = 24;
    const DISC_MASK: u64 = 0xFF << EntityId::DISC_OFFSET;
    // Like for KeyData, the maximum index is reserved for use as the null value
    const MAX_IDX: u32 = 0x0FFFFFFF;

    /// Extracts the discriminant field.
    #[inline]
    fn discriminant(&self) -> u8 {
        ((u64::from(self.0) & Self::DISC_MASK) >> Self::DISC_OFFSET) as u8
    }

    /// Extracts the version field.
    #[inline]
    fn version(&self) -> u32 {
        (u64::from(self.0) >> 32) as u32
    }

    /// Extracts the index field.
    #[inline]
    fn index(&self) -> u32 {
        (u64::from(self.0) & !Self::DISC_MASK) as u32
    }

    /// Wraps the key data of a key to create an ID.
    ///
    /// # Panics
    ///
    /// Panics if the key's index equals or exceeds [`EntityId::MAX_IDX`] = 2<sup>28</sup> − 1,
    /// in which case the `MolMap` is full.
    #[inline]
    fn from_key_data(kind: EntityKind, key_data: KeyData) -> Self {
        let ffi = key_data.as_ffi();
        // Max index indicates the null key, which is fine
        if (ffi as u32 >= Self::MAX_IDX) && (ffi as u32 != u32::MAX) {
            panic!("MolMap is full!");
        }
        // The version is non-zero and therefore the FFI representation is too,
        // so this is safe to do
        EntityId(unsafe {
            NonZeroU64::new_unchecked(
                ((kind as u64) << Self::DISC_OFFSET) | (ffi & !Self::DISC_MASK),
            )
        })
    }

    /// Returns the equivalent key data.
    #[inline]
    fn to_key_data(self) -> KeyData {
        let ffi = self.to_raw_key();
        KeyData::from_ffi(ffi)
    }

    /// Returns the inner value without the discriminant.
    #[inline]
    fn to_raw_key(self) -> u64 {
        u64::from(self.0) & !Self::DISC_MASK
    }

    /// Returns the inner value with the discriminant as a normal `u64`.
    #[inline]
    fn to_raw(self) -> u64 {
        u64::from(self.0)
    }

    /// Checks if an ID represents the null key.
    ///
    /// Following `slotmap`, the null key is the one with the maximum index.
    #[inline]
    fn is_null(&self) -> bool {
        self.index() == Self::MAX_IDX
    }
}

impl Debug for EntityId {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:#04X}/{}.{}",
            self.discriminant(),
            self.index(),
            self.version()
        )
    }
}

impl Id for EntityId {
    type Tagged = TaggedEntity;

    #[inline]
    fn kind(&self) -> EntityKind {
        EntityKind::try_from(self.discriminant())
            .expect("Discriminant should always be a valid value")
    }

    fn to_tagged(self) -> TaggedEntity {
        match self.kind() {
            EntityKind::Atom => TaggedEntity::Atom(AtomId(self)),
            EntityKind::Pseudoatom => TaggedEntity::Pseudoatom(PseudoatomId(self)),
            EntityKind::Bond => TaggedEntity::Bond(BondId(self)),
            EntityKind::Substituent => TaggedEntity::Substituent(SubstituentId(self)),
            EntityKind::Molecule => TaggedEntity::Molecule(MoleculeId(self)),
        }
    }
}

/// An ID for a kind of entity that is also a key for the corresponding `SlotMap`.
///
/// Like category ID types, they wrap an [`EntityId`] (i.e. a `u64`), but key ID
/// types are also an actual [`SlotMap::Key`].
pub(crate) trait KeyId: slotmap::Key + Id {
    /// The kind of entity that the ID represents.
    const KIND: EntityKind;
}

// First we define the different key IDs, which are all just SlotMap keys.

/// Defines a key ID type: an [`Id`] that implements [`slotmap::Key`] and [`KeyId`].
///
/// There must already exist:
/// - A matching `EntityKind::$name` variant
///
/// As well as the key ID type itself, named `$nameId`, an enum named
/// `$nameKind` gets defined with a single variant, `$name`.
///
/// The following trait implementations are then defined:
/// - `slotmap::Key for $nameId`
/// - `Id for $nameId`
/// - `KeyId for $nameId`
/// - `From<$nameId> for EntityId` and `TryFrom<EntityId> for $nameId`
///
/// Requires ident concatenation, currently done using the `paste` crate (which,
/// though now marked as unmaintained, is at least stable, mature, and popular).
macro_rules! define_key_id {
    (
        $(#[$doc:meta])*
        $name:ident;
    ) => {
        paste::paste! {
            #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
            #[repr(u8)]
            pub enum [<Tagged $name>] {
                $name = EntityKind::$name as u8,
            }

            $(#[$doc])*
            #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
            pub struct [<$name Id>](EntityId);

            impl From<KeyData> for [<$name Id>] {
                #[inline]
                fn from(key_data: KeyData) -> Self {
                    Self(EntityId::from_key_data(EntityKind::$name, key_data))
                }
            }

            impl Default for [<$name Id>] {
                #[inline]
                fn default() -> Self {
                    KeyData::default().into()
                }
            }

            unsafe impl Key for [<$name Id>] {
                #[inline]
                fn data(&self) -> KeyData {
                    self.0.to_key_data()
                }

                // Have to override default is_null method because it goes via
                // the KeyData, and unfortunately the null key is not round-trippable
                #[inline]
                fn is_null(&self) -> bool {
                    self.0.is_null()
                }
            }

            impl Id for [<$name Id>] {
                type Tagged = [<Tagged $name>];

                #[inline]
                fn kind(&self) -> EntityKind {
                    EntityKind::$name
                }

                #[inline]
                fn to_tagged(self) -> [<Tagged $name>] {
                    [<Tagged $name>]::$name
                }
            }

            impl KeyId for [<$name Id>] {
                const KIND: EntityKind = EntityKind::$name;
            }

            impl From<[<$name Id>]> for EntityId {
                #[inline]
                fn from(id: [<$name Id>]) -> EntityId {
                    id.0
                }
            }

            impl TryFrom<EntityId> for [<$name Id>] {
                type Error = MolMapError;

                fn try_from(id: EntityId) -> MolMapResult<Self> {
                    if id.discriminant() == EntityKind::$name as u8 {
                        Ok(Self(id))
                    } else {
                        Err(MolMapError::IncorrectEntityKind(id.kind(), id))
                    }
                }
            }
        }
    };
}

define_key_id! {
    /// An ID corresponding to a specific atom entity in a `MolMap`.
    Atom;
}

define_key_id! {
    /// An ID corresponding to a specific pseudoatom entity in a `MolMap`.
    Pseudoatom;
}

define_key_id! {
    /// An ID corresponding to a specific bond entity in a `MolMap`.
    Bond;
}

define_key_id! {
    /// An ID corresponding to a specific substituent entity in a `MolMap`.
    Substituent;
}

define_key_id! {
    /// An ID corresponding to a specific molecule entity in a `MolMap`.
    Molecule;
}

// Now we define the different category IDs, which all just wrap an `EntityId`.
// It is important to note that any key ID can be converted into an `EntityId`
// trivially, and that any resulting `EntityId` can be wrapped to create any
// category ID type without an error occurring.
// Which kinds of entities can be converted to what is strictly controlled by
// the `From` and `TryFrom` implementations, so these should be implemented with
// thought and care.

/// Defines a category ID type wrapping [`EntityId`], covering the specified subset
/// of entity kinds.
///
/// For each `$variant` given, there must already exist:
/// - A matching `EntityKind::$variant` variant
/// - A key ID type named `$variantId` (e.g. `AtomId`) that is
///   `Into<EntityId>`
///
/// As well as the category ID type itself, named `$nameId`, an enum named
/// `$nameKind` gets defined with the provided variants and the same discriminant
/// values as for [`EntityKind`].
///
/// The following trait implementations are then defined:
/// - `Id for $nameId`
/// - `From<$nameId> for EntityId` and `TryFrom<EntityId> for $nameId`
/// - `From<$variantId> for $nameId` and `TryFrom<$nameId> for $variantId` for each `$variant`
///
/// Requires ident concatenation, currently done using the `paste` crate (which,
/// though now marked as unmaintained, is at least stable, mature, and popular).
///
/// # Usage
///
/// ```ignore
/// define_category_id! {
///     /// An ID of a category.
///     Category {
///         Atom,
///         Pseudoatom,
///         Bond,
///     }
/// }
/// ```
///
/// will define:
///
/// ```ignore
/// /// An ID of a category.
/// #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
/// pub struct CategoryId(EntityId);
///
/// /// The kinds of entity covered by [`CategoryId`].
/// #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
/// pub enum CategoryKind {
///     Atom = EntityId::Atom as u8,
///     Pseudoatom = EntityId::Pseudoatom as u8,
///     Bond = EntityId::Bond as u8,
/// }
/// ```
#[macro_export]
macro_rules! define_category_id {
    (
        $(#[$doc:meta])*
        $name:ident {
            $($variant:ident),+ $(,)?
        }
    ) => {
        #[allow(non_snake_case)]
        paste::paste! {
            $(#[$doc])*
            #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
            pub struct [<$name Id>](EntityId);

            #[doc = concat!(
                "The kinds of entity covered by [`", stringify!([<$name Id>]), "`]."
            )]
            #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
            #[repr(u8)]
            pub enum [<Tagged $name>] {
                $($variant([<$variant Id>]) = EntityKind::$variant as u8,)+
            }

            impl Id for [<$name Id>] {
                type Tagged = [<Tagged $name>];

                #[inline]
                fn kind(&self) -> EntityKind {
                    self.0.kind()
                }

                fn to_tagged(self) -> [<Tagged $name>] {
                    match self.0.kind() {
                        $(EntityKind::$variant => [<Tagged $name>]::$variant([<$variant Id>](self.0)),)+
                        _ => unreachable!()
                    }
                }
            }

            impl From<[<$name Id>]> for EntityId {
                #[inline]
                fn from(id: [<$name Id>]) -> EntityId {
                    id.0
                }
            }

            impl TryFrom<EntityId> for [<$name Id>] {
                type Error = MolMapError;

                fn try_from(id: EntityId) -> MolMapResult<Self> {
                    match id.kind() {
                        $(EntityKind::$variant => Ok(Self(id)),)+
                        _ => Err(MolMapError::IncorrectEntityKind(id.kind(), id)),
                    }
                }
            }

            $(
                impl From<[<$variant Id>]> for [<$name Id>] {
                    #[inline]
                    fn from(id: [<$variant Id>]) -> Self {
                        Self(id.into())
                    }
                }

                impl TryFrom<[<$name Id>]> for [<$variant Id>] {
                    type Error = MolMapError;

                    fn try_from(id: [<$name Id>]) -> MolMapResult<[<$variant Id>]> {
                        if id.0.discriminant() == (EntityKind::$variant as u8) {
                            Ok([<$variant Id>](id.0))
                        } else {
                            Err(MolMapError::IncorrectEntityKind(id.kind(), id.0))
                        }
                    }
                }
            )+
        }
    };
}

define_category_id! {
    /// An ID of an atom or something that behaves like one (a pseudoatom).
    ///
    /// Atomlikes are the true nodes of the molecular graph.
    Atomlike {
        Atom,
        Pseudoatom,
    }
}

define_category_id! {
    /// An ID of a fundamental entity, an entity that does not group other entities.
    ///
    /// Fundamentals are the basic building blocks of a [`MolMap`].
    ///
    /// Atoms, pseudoatoms, and bonds are fundamentals.
    Fundamental {
        Atom,
        Pseudoatom,
        Bond,
    }
}

define_category_id! {
    /// An ID of a collection, an aggregation of fundamental entities.
    Collection {
        Substituent,
        Molecule,
    }
}

define_category_id! {
    /// An ID of an entity that can form bonds.
    Bondable {
        Atom,
        Pseudoatom,
        //Bond,
    }
}

define_category_id! {
    /// An ID of an entity that an `Object` can be attached to.
    Anchor {
        Atom,
        Pseudoatom,
        Bond,
        Substituent,
        Molecule,
    }
}

// Finally, some additional inter-category conversions

impl From<AtomlikeId> for FundamentalId {
    fn from(id: AtomlikeId) -> FundamentalId {
        FundamentalId(id.0)
    }
}

impl From<AtomlikeId> for BondableId {
    fn from(id: AtomlikeId) -> BondableId {
        BondableId(id.0)
    }
}

#[cfg(test)]
mod tests {
    use slotmap::{DefaultKey, SlotMap, new_key_type};

    use super::*;

    // Some raw keys for use when testing, so that they only need updating in
    // one place if anything changes
    const ATOM_NULL_RAW: u64 = 0x0000001_00_FFFFFF; // The null atom key
    const KD_NULL_RAW: u64 = 0x0000001_FF_FFFFFF; // What KeyData considers to be null

    const BOND_RAW: NonZeroU64 = NonZeroU64::new(0x1_01_000008).unwrap(); // version: 1, kind: Bond, idx: 8
    const ATOM_RAW: NonZeroU64 = NonZeroU64::new(0x3_00_000010).unwrap(); // version: 3, kind: Atom, idx: 16, (version always odd for occupied slots)
    const PSEUDOATOM_RAW: NonZeroU64 = NonZeroU64::new(0x1_03_00000A).unwrap(); // version: 1, kind: Pseudoatom, idx: 10
    const MOL_RAW: NonZeroU64 = NonZeroU64::new(0x1_1F_000001).unwrap(); // version: 1, kind: Molecule, idx: 1

    const BOND: BondId = BondId(EntityId(BOND_RAW));
    const ATOM: AtomId = AtomId(EntityId(ATOM_RAW));
    const PSEUDOATOM: PseudoatomId = PseudoatomId(EntityId(PSEUDOATOM_RAW));
    const MOL: MoleculeId = MoleculeId(EntityId(MOL_RAW));

    #[test]
    fn slotmap_key_ffi_layout() {
        // Confirm the FFI representation is what we think it is and hasn't changed
        // (no guarantees are made by slotmap about it, so it could change without it
        // being a SemVer breaking change, but it would be a breaking change for us)
        // idx should be the lower 32 bits, version the higher 32
        let null = KeyData::default(); // idx: u32::MAX, version: 1
        assert_eq!(null.as_ffi(), KD_NULL_RAW);
        let mut sm: SlotMap<DefaultKey, usize> = SlotMap::new();
        let first = sm.insert(1); // idx: 1, version: 1
        assert_eq!(first.data().as_ffi(), 0x00000001_00000001);
        let second = sm.insert(2); // idx: 2, version: 1
        assert_eq!(second.data().as_ffi(), 0x00000001_00000002);
        // Remove a key then insert a new one, should reuse the index
        sm.remove(first); // idx 1 now free
        let third = sm.insert(3); // idx: 1, version: 3
        assert_eq!(third.data().as_ffi(), 0x00000003_00000001);
    }

    #[test]
    fn entity_getters() {
        let e = EntityId(ATOM_RAW);
        assert_eq!(e.discriminant(), 0);
        assert_eq!(e.version(), 3);
        assert_eq!(e.index(), 16);
    }

    #[test]
    fn from_key_data() {
        let k = KeyData::from_ffi(0x1_00000001); // idx: 1, version: 1
        let id = EntityId::from_key_data(EntityKind::Atom, k);
        // Atom has discriminant of 0
        assert_eq!(id.to_raw(), 0x1_00000001);
        let mut sm: SlotMap<AtomId, usize> = SlotMap::with_key();
        let first = sm.insert(1); // idx: 1, version: 1
        assert_eq!(first.0.to_raw(), 0x1_00000001);
        let second = sm.insert(2); // idx: 2, version: 1
        assert_eq!(second.0.to_raw(), 0x1_00000002);
    }

    #[test]
    #[should_panic]
    fn panics_on_overflow() {
        let overflowed = KeyData::from_ffi(0x1_10000000);
        let _ = MoleculeId::from(overflowed);
    }

    #[test]
    fn key_data_round_trip() {
        // Round trip is survived by any valid key that hasn't overflowed
        let kd = KeyData::from_ffi(0x1_00123456); // idx: 123456, version: 1
        let atom: AtomId = kd.into();
        let recovered = atom.data();
        assert_eq!(kd, recovered);
        // Unfortunately, a null key doesn't survive a round trip
        //let null = KeyData::default();
        //let atom_null: AtomId = null.into();
        //let recovered_null = atom_null.data();
        //assert_eq!(null, recovered_null);
    }

    #[test]
    fn key_id_slotmap_access() {
        let mut sm: SlotMap<BondId, usize> = SlotMap::with_key();
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

    #[test]
    fn different_discriminants_same_keys() {
        // First assigned key in two different slotmaps
        let atom = AtomId(EntityId(0x1_02_000001.try_into().unwrap()));
        let bond = BondId(EntityId(0x1_01_000001.try_into().unwrap()));
        assert_eq!(atom.0.to_raw_key(), bond.0.to_raw_key());
    }

    #[test]
    fn entity_kind() {
        let atom = EntityId(ATOM_RAW);
        let bond = EntityId(BOND_RAW);
        let mol = EntityId(MOL_RAW);
        assert_eq!(atom.kind(), EntityKind::Atom);
        assert_eq!(bond.kind(), EntityKind::Bond);
        assert_eq!(mol.kind(), EntityKind::Molecule);
    }

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
    fn category_kind() {
        let atomlike = AtomlikeId(EntityId(ATOM_RAW));
        let fundamental = FundamentalId(EntityId(BOND_RAW));
        let collection = CollectionId(EntityId(MOL_RAW));
        assert_eq!(atomlike.kind(), EntityKind::Atom);
        assert_eq!(fundamental.kind(), EntityKind::Bond);
        assert_eq!(collection.kind(), EntityKind::Molecule);
    }

    #[test]
    fn convert_key_to_category() {
        let atom = ATOM;
        // Conversion to an Atomlike is infallible
        let atomlike: AtomlikeId = atom.into();
        // ID stays the same
        assert_eq!(atom.0, atomlike.0);
        // Underlying key is the same
        assert_eq!(atom.0.to_raw_key(), atomlike.0.to_raw_key());
        // Conversion via the Entity works too
        assert_eq!(
            AtomlikeId::try_from(EntityId::from(atom)).unwrap(),
            atomlike
        );
    }

    #[test]
    fn convert_key_to_category_fails() {
        // Conversion of a Bond to an Atomlike is forbidden
        // There's simply no From implementation
        // It can be attempted via the Entity, but it should fail
        let bond = BOND;
        let attempt = AtomlikeId::try_from(EntityId::from(bond));
        assert!(AtomlikeId::try_from(EntityId::from(bond)).is_err());
    }

    #[test]
    fn convert_category_to_key() {
        let atom = FundamentalId(EntityId(ATOM_RAW));
        let bond = FundamentalId(EntityId(BOND_RAW));
        // Conversion should work when the attempted conversion aligns with the kind
        assert!(AtomId::try_from(atom).is_ok());
        assert!(BondId::try_from(bond).is_ok());
        // But not otherwise
        assert!(BondId::try_from(atom).is_err());
        assert!(AtomId::try_from(bond).is_err());
        assert!(PseudoatomId::try_from(atom).is_err());
        assert!(PseudoatomId::try_from(bond).is_err());
    }

    #[test]
    fn convert_key_cat_key_round_trip() {
        // Bond to Fundamental works, as does round trip
        let bond = BOND;
        assert_eq!(FundamentalId::from(bond), FundamentalId(EntityId(BOND_RAW)));
        assert_eq!(BondId::try_from(FundamentalId::from(bond)).unwrap(), bond);
        // Molecule to Collection to Entity to Molecule should all work
        let mol = MOL;
        let col: CollectionId = mol.into();
        let ent: EntityId = col.into();
        let recovered: MoleculeId = ent.try_into().unwrap();
        assert_eq!(ent, mol.0);
        assert_eq!(recovered, mol);
    }

    #[test]
    fn convert_between_categories() {
        let atom = AtomlikeId(EntityId(ATOM_RAW));
        let pseudoatom = AtomlikeId(EntityId(PSEUDOATOM_RAW));
        assert_eq!(FundamentalId::from(atom), FundamentalId(EntityId(ATOM_RAW)));
        assert_eq!(
            FundamentalId::from(pseudoatom),
            FundamentalId(EntityId(PSEUDOATOM_RAW))
        );
        assert_eq!(BondableId::from(atom), BondableId(EntityId(ATOM_RAW)));
        assert_eq!(
            BondableId::from(pseudoatom),
            BondableId(EntityId(PSEUDOATOM_RAW))
        );
    }
}
