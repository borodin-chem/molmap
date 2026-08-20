// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt::{Debug, Formatter};
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::num::{IntErrorKind, NonZeroU16, NonZeroU32, NonZeroU64};

use slotmap::{Key, KeyData};

use crate::*;

// Each kind of entity has a corresponding SlotMap in a MolGraph and therefore
// its own slotmap::Key type. These could be used as IDs for the entities.
// However, in various places it is necessary to store polymorphic IDs (i.e. IDs
// of multiple kinds of entity) in a single collection, which thus requires some
// sort of type erasure.
//
// The strategy originally pursued was to wrap the keys in "category ID" enums,
// but this makes the category ID types 12 bytes vs 8 for the the entity key
// types, and as they are widely used, that just means a lot of unnecessary
// memory use.
//
// So, instead, we convert the entity keys to universal "entity IDs". These are
// identical to the raw `u64` representations of the keys (obtained using the
// `as_ffi` method) but with the high 8 bits of the index field co-opted to
// store a discriminant that indicates the entity kind.
//
// In this way, an ID can be cast from the "key ID" type, which is specific to
// the kind of entity, to a "category ID" type, which could be one of a number
// of different kinds, at zero cost (because the same underlying representation
// is kept), and with the kind of entity it corresponds to recoverable
// dynamically, despite no increase in size.
//
// It also means that all IDs are globally unique.
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
// types is limited to 256. Only those in the range of 0 to 127 are reserved for
// use in the core `MolGraph`; the rest are left for use by extensions that wish
// to add additional entity types but still use unified IDs.

/// The ID of one of any kind of entity, where the kind of entity is encoded by
/// an 8-bit discriminant.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) struct EntityId(pub(crate) NonZeroU64);

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
    pub(crate) fn to_raw_key(self) -> u64 {
        u64::from(self.0) & !Self::DISC_MASK
    }

    /// Wraps a non-zero integer to create an ID.
    #[inline]
    pub(crate) fn from_raw(n: u64) -> Option<Self> {
        Some(Self(NonZeroU64::new(n)?))
    }

    /// Returns the inner value with the discriminant as a normal `u64`.
    #[inline]
    pub(crate) fn to_raw(self) -> u64 {
        u64::from(self.0)
    }

    /// Checks if an ID represents the null key.
    ///
    /// Following `slotmap`, the null key is the one with the maximum index.
    #[inline]
    fn is_null(&self) -> bool {
        self.index() == Self::MAX_IDX
    }

    /// Returns the kind of entity that the ID represents, amongst all possible
    /// kinds of entity.
    fn kind(&self) -> EntityKind {
        EntityKind::try_from(self.discriminant()).expect("The discriminant should never be invalid")
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

/// A key ID as a tagged enum.
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
//#[non_exhaustive]
pub enum TaggedId {
    Atom(Id<Atom>) = EntityKind::Atom as u8,
    Bond(Id<Bond>) = EntityKind::Bond as u8,
    Pseudoatom(Id<Pseudoatom>) = EntityKind::Pseudoatom as u8,
    Substituent(Id<Substituent>) = EntityKind::Substituent as u8,
    Molecule(Id<Molecule>) = EntityKind::Molecule as u8,
}

/// The ID of an entity.
#[derive(Copy, Clone)]
pub struct Id<E: Entity>(EntityId, PhantomData<E>);

impl<E: Entity> Id<E> {
    /// Creates a new ID for the requested kind of entity without checking that
    /// the discriminant of the ID is correct for that kind.
    pub(crate) const fn new_unchecked(id: EntityId) -> Self {
        Self(id, PhantomData)
    }

    pub(crate) fn into_inner(self) -> EntityId {
        self.0
    }

    pub(crate) fn from_raw_unchecked(n: u64) -> Self {
        Self::new_unchecked(EntityId::from_raw(n).unwrap())
    }

    /// Returns the kind of entity that the ID represents, amongst all possible
    /// kinds of entity.
    pub fn kind(&self) -> EntityKind {
        self.0.kind()
    }

    ///// Returns a tagged form of the ID that wraps the actual key in an enum.
    /////
    ///// The tagged key form is inherently larger than the ID form but allows convenient
    ///// matching on the kind while holding the underlying key.
    //pub fn to_tagged(self) -> TaggedId {
    //    match self.kind() {
    //        EntityKind::Atom => TaggedId::Atom(Id::<Atom>::new_unchecked(self.0)),
    //        EntityKind::Bond => TaggedId::Bond(Id::<Bond>::new_unchecked(self.0)),
    //        EntityKind::Pseudoatom => TaggedId::Pseudoatom(Id::<Pseudoatom>::new_unchecked(self.0)),
    //        EntityKind::Substituent => {
    //            TaggedId::Substituent(Id::<Substituent>::new_unchecked(self.0))
    //        }
    //        EntityKind::Molecule => TaggedId::Molecule(Id::<Molecule>::new_unchecked(self.0)),
    //    }
    //}
}

impl<E: Entity> Debug for Id<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Id")
            .field(&self.0)
            .field(&self.kind())
            .finish()
    }
}

impl<E: Entity> PartialEq for Id<E> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<E: Entity> Eq for Id<E> {}

impl<E: Entity> PartialOrd for Id<E> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<E: Entity> Ord for Id<E> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<E: Entity> std::hash::Hash for Id<E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<E: Entity> From<Id<E>> for EntityId {
    #[inline]
    fn from(id: Id<E>) -> EntityId {
        id.0
    }
}

// IDs are keys for the slotmaps

impl<E: Entity + KeyEntity> Default for Id<E> {
    #[inline]
    fn default() -> Self {
        KeyData::default().into()
    }
}

impl<E: Entity + KeyEntity> From<KeyData> for Id<E> {
    #[inline]
    fn from(key_data: KeyData) -> Self {
        Self::new_unchecked(EntityId::from_key_data(E::kind(), key_data))
    }
}

unsafe impl<E: Entity + KeyEntity> Key for Id<E> {
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

// Don't think this needs to be a trait
// Just implement separately for kind/key IDs and category IDs

///// An ID type that may be for one specific kind of entity or for one in a
///// category of entity.
/////
///// Implementors are required to ensure that they convert to the correct variant of
///// [`EntityId`], and that only the correct variants convert to the `Id` type.
//pub trait TypedId: Into<EntityId> + TryFrom<EntityId> {
//    fn to_tagged(self) -> TaggedId;
//
//    fn kind(&self) -> EntityKind;
//}

/// An iterator over a set of IDs.
#[derive(Copy, Clone, Debug)]
pub struct IdIter<E, I>(pub(crate) I)
where
    E: Entity,
    I: Iterator<Item = Id<E>>;

impl<E, I> Iterator for IdIter<E, I>
where
    E: Entity,
    I: Iterator<Item = Id<E>>,
{
    type Item = Id<E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<E, I> ExactSizeIterator for IdIter<E, I>
where
    E: Entity,
    I: Iterator<Item = Id<E>> + ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<E, I> FusedIterator for IdIter<E, I>
where
    E: Entity,
    I: Iterator<Item = Id<E>> + FusedIterator,
{
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

    const BOND: Id<Bond> = Id::new_unchecked(EntityId(BOND_RAW));
    const ATOM: Id<Atom> = Id::new_unchecked(EntityId(ATOM_RAW));
    const PSEUDOATOM: Id<Pseudoatom> = Id::new_unchecked(EntityId(PSEUDOATOM_RAW));
    const MOL: Id<Molecule> = Id::new_unchecked(EntityId(MOL_RAW));

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
        let mut sm: SlotMap<Id<Atom>, usize> = SlotMap::with_key();
        let first = sm.insert(1); // idx: 1, version: 1
        assert_eq!(first.0.to_raw(), 0x1_00000001);
        let second = sm.insert(2); // idx: 2, version: 1
        assert_eq!(second.0.to_raw(), 0x1_00000002);
    }

    #[test]
    #[should_panic]
    fn panics_on_overflow() {
        let overflowed = KeyData::from_ffi(0x1_10000000);
        let _ = Id::<Molecule>::from(overflowed);
    }

    #[test]
    fn key_data_round_trip() {
        // Round trip is survived by any valid key that hasn't overflowed
        let kd = KeyData::from_ffi(0x1_00123456); // idx: 123456, version: 1
        let atom: Id<Atom> = kd.into();
        let recovered = atom.data();
        assert_eq!(kd, recovered);
        // Unfortunately, a null key doesn't survive a round trip
        //let null = KeyData::default();
        //let atom_null: Id<Atom> = null.into();
        //let recovered_null = atom_null.data();
        //assert_eq!(null, recovered_null);
    }

    #[test]
    fn key_id_slotmap_access() {
        let mut sm: SlotMap<Id<Bond>, usize> = SlotMap::with_key();
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
        let atom = Id::<Atom>::new_unchecked(EntityId::from_raw(0x1_02_000001).unwrap());
        let bond = Id::<Bond>::new_unchecked(EntityId::from_raw(0x1_01_000001).unwrap());
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
}
