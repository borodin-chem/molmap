// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Definition of the entity ID type and its relationship to slotmap::Key types.
//!
//! Each kind of entity has a corresponding SlotMap in a MolGraph and therefore
//! its own slotmap::Key type. These could be used as IDs for the entities.
//! However, in various places it is necessary to store polymorphic IDs (i.e. IDs
//! of multiple kinds of entity) in a single collection, which thus requires some
//! sort of type erasure.
//!
//! The strategy originally pursued was to wrap the keys in "category ID" enums,
//! but this makes the category ID types 12 bytes vs 8 for the the entity key
//! types, and as they are widely used, that just means a lot of unnecessary
//! memory use.
//!
//! So, instead, we convert the entity keys to universal "entity IDs". These are
//! identical to the raw `u64` representations of the keys (obtained using the
//! `as_ffi` method) but with the high 8 bits of the index field co-opted to
//! store a discriminant that indicates the entity kind.
//!
//! In this way, an ID can be cast from the "key ID" type, which is specific to
//! the kind of entity, to a "category ID" type, which could be one of a number
//! of different kinds, at zero cost (because the same underlying representation
//! is kept), and with the kind of entity it corresponds to recoverable
//! dynamically, despite no increase in size.
//!
//! It also means that all IDs are globally unique.
//!
//! The version of a SlotMap key wraps, so even if 2^31 deletion-addition cycles
//! occur at the same slot, there will be no issues; the only situation in which
//! a stale key will spuriously refer to something other than it originally did
//! is if *exactly* 2^31 deletion-addition cycles have occurred at the same slot
//! since the original key was generated and access is attempted with the stale
//! original key. This is exceptionally unlikely.
//!
//! If the bits for the discriminant are stolen from the version field, the
//! version has to be hard capped at the maximum value of the remaining bits i.e.
//! 2^27, as anything higher will not survive a round trip.
//!
//! As such, it is better to take the bits from the index field, as that already
//! has a hard cap (we just reduce it). This means the number of atoms is limited
//! to 2^24, as is the number of bonds (or indeed any individual entity).
//!
//! As the discriminant is encoded by 8 bits, the number of different entity
//! types is limited to 256. Only those in the range of 0 to 127 are reserved for
//! use in the core `MolGraph`; the rest are left for use by extensions that wish
//! to add additional entity types but still use IDs that are compatible (i.e.
//! non-conflicting) with these IDs.

use std::fmt::{Debug, Formatter};
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::num::{IntErrorKind, NonZeroU16, NonZeroU32, NonZeroU64};

use slotmap::{Key, KeyData};

use crate::entities::*;

/// The ID of one of any kind of entity, where the kind of entity is encoded by
/// an 8-bit discriminant.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
// It is crucial that this module remains private and that Id is not re-exported
// as pub, as its unnamability is what ensures that Entity is a sealed trait!
pub struct Id(pub(crate) NonZeroU64);

// Layout:
// - Bits 63–32 = version (`version` field of `KeyData`)
// - Bits 31–24 = discriminant/kind
// - Bits 23–0  = index (`idx` field of `KeyData` truncated to 28 bits)

impl Id {
    const DISC_OFFSET: u64 = 24;
    const DISC_MASK: u64 = 0xFF << Id::DISC_OFFSET;
    // Like for KeyData, the maximum index is reserved for use as the null value
    const MAX_IDX: u32 = 0x0FFFFFFF;

    /// Extracts the discriminant field.
    #[inline]
    const fn discriminant(&self) -> u8 {
        ((self.0.get() & Self::DISC_MASK) >> Self::DISC_OFFSET) as u8
    }

    /// Extracts the version field.
    #[inline]
    const fn version(&self) -> u32 {
        (self.0.get() >> 32) as u32
    }

    /// Extracts the index field.
    #[inline]
    const fn index(&self) -> u32 {
        (self.0.get() & !Self::DISC_MASK) as u32
    }

    /// Wraps a (non-zero) integer to create an ID.
    ///
    /// Returns `None` if `n` is zero or if the discriminant is an invalid value.
    #[inline]
    pub(crate) const fn from_raw(n: u64) -> Option<Self> {
        if let Some(non_zero) = NonZeroU64::new(n) {
            let id = Self(non_zero);
            if EntityKind::from_u8(id.discriminant()).is_some() {
                Some(id)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns the inner value with the discriminant as a normal `u64`.
    #[inline]
    pub(crate) const fn to_raw(self) -> u64 {
        self.0.get()
    }

    /// Wraps the FFI representation of a `slotmap::Key` to create an ID.
    ///
    /// # Panics
    ///
    /// Panics if the key's index equals or exceeds [`Id::MAX_IDX`] = 2<sup>28</sup> − 1,
    /// in which case the `MolMap` is full.
    #[inline]
    const fn from_raw_key(kind: EntityKind, ffi: u64) -> Self {
        // Max index indicates the null key, which is fine
        if (ffi as u32 >= Self::MAX_IDX) && (ffi as u32 != u32::MAX) {
            panic!("MolMap is full!");
        }
        // The version is non-zero and therefore the FFI representation is too,
        // so this is safe to do
        Id(unsafe {
            NonZeroU64::new_unchecked(
                ((kind as u64) << Self::DISC_OFFSET) | (ffi & !Self::DISC_MASK),
            )
        })
    }

    /// Returns the inner value without the discriminant (equivalent to the FFI representation of the `slotmap::Key`).
    #[inline]
    const fn to_raw_key(self) -> u64 {
        self.0.get() & !Self::DISC_MASK
    }

    /// Wraps the key data of a `slotmap::Key` to create an ID.
    ///
    /// # Panics
    ///
    /// Panics if the key's index equals or exceeds [`Id::MAX_IDX`] = 2<sup>28</sup> − 1,
    /// in which case the `MolMap` is full.
    #[inline]
    pub(crate) fn from_key_data(kind: EntityKind, key_data: KeyData) -> Self {
        let ffi = key_data.as_ffi();
        Self::from_raw_key(kind, ffi)
    }

    /// Returns the equivalent key data.
    #[inline]
    pub(crate) fn to_key_data(self) -> KeyData {
        let ffi = self.to_raw_key();
        KeyData::from_ffi(ffi)
    }

    /// Checks if an ID represents the null key.
    ///
    /// Following `slotmap`, the null key is the one with the maximum index.
    #[inline]
    const fn is_null(&self) -> bool {
        self.index() == Self::MAX_IDX
    }

    /// Returns the kind of entity that the ID represents, amongst all possible
    /// kinds of entity.
    pub(crate) const fn kind(&self) -> EntityKind {
        EntityKind::from_u8(self.discriminant()).expect("The discriminant should never be invalid")
    }
}

impl Debug for Id {
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

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use slotmap::{DefaultKey, KeyData, SlotMap, new_key_type};

    use super::*;

    // Some raw keys for use when testing, so that they only need updating in
    // one place if anything changes
    const ATOM_NULL_RAW: u64 = 0x0000001_00_FFFFFF; // The null atom key
    const KD_NULL_RAW: u64 = 0x0000001_FF_FFFFFF; // What KeyData considers to be null

    const BOND_RAW: u64 = 0x1_01_000008; // version: 1, kind: Bond, idx: 8
    const ATOM_RAW: u64 = 0x3_00_000010; // version: 3, kind: Atom, idx: 16, (version always odd for occupied slots)
    const PSEUDOATOM_RAW: u64 = 0x1_02_00000A; // version: 1, kind: Pseudoatom, idx: 10
    const MOLECULE_RAW: u64 = 0x1_1F_000001; // version: 1, kind: Molecule, idx: 1

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
        assert_eq!(first.data().as_ffi(), 0x00000001_00_000001);
        let second = sm.insert(2); // idx: 2, version: 1
        assert_eq!(second.data().as_ffi(), 0x00000001_00_000002);
        // Remove a key then insert a new one, should reuse the index
        sm.remove(first); // idx 1 now free
        let third = sm.insert(3); // idx: 1, version: 3
        assert_eq!(third.data().as_ffi(), 0x00000003_00_000001);
    }

    #[test]
    fn entity_getters() {
        let e = Id::from_raw(ATOM_RAW).unwrap();
        assert_eq!(e.discriminant(), 0);
        assert_eq!(e.version(), 3);
        assert_eq!(e.index(), 16);
    }

    #[test]
    fn from_raw() {
        for n in [0x1_00_000001, 0x1_00_000002, 0x1_01_000003] {
            assert_eq!(Id::from_raw(n).unwrap(), Id(NonZeroU64::new(n).unwrap()))
        }
    }

    #[test]
    fn from_raw_invalid() {
        // Shouldn't be able to create an Id from 0
        assert!(Id::from_raw(0).is_none());
        // Shouldn't be able to create an Id with an invalid discriminant
        assert!(Id::from_raw(0x1_06_000001).is_none());
        assert!(Id::from_raw(0x1_FF_000001).is_none());
    }

    #[test]
    fn from_key_data() {
        let k = KeyData::from_ffi(0x1_00_000001); // idx: 1, version: 1
        let id = Id::from_key_data(EntityKind::Atom, k);
        // Atom has discriminant of 0
        assert_eq!(id.to_raw(), 0x1_00_000001);
        new_key_type! { struct AtomKey; }
        let mut sm: SlotMap<AtomKey, usize> = SlotMap::with_key();
        let first = sm.insert(1); // idx: 1, version: 1
        assert_eq!(
            Id::from_key_data(EntityKind::Atom, first.data()),
            Id::from_raw(0x1_00_000001).unwrap()
        );
        let second = sm.insert(2); // idx: 2, version: 1
        assert_eq!(
            Id::from_key_data(EntityKind::Atom, second.data()),
            Id::from_raw(0x1_00_000002).unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn panics_on_overflow() {
        let overflowed = KeyData::from_ffi(0x1_10000000);
        let _ = Id::from_key_data(EntityKind::Molecule, overflowed);
    }

    #[test]
    fn key_data_round_trip() {
        // Round trip is survived by any valid key that hasn't overflowed
        let kd = KeyData::from_ffi(0x1_00_123456); // idx: 123456, version: 1
        let atom = Id::from_key_data(EntityKind::Atom, kd);
        let recovered = atom.to_key_data();
        assert_eq!(kd, recovered);
        // Unfortunately, a null key doesn't survive a round trip
        //let null = KeyData::default();
        //let atom_null: Atom = null.into();
        //let recovered_null = atom_null.data();
        //assert_eq!(null, recovered_null);
    }

    #[test]
    fn different_discriminants_same_keys() {
        // First assigned key in two different slotmaps
        let atom = Id::from_raw(0x1_02_000001).unwrap();
        let bond = Id::from_raw(0x1_01_000001).unwrap();
        assert_eq!(atom.to_raw_key(), bond.to_raw_key());
    }

    #[test]
    fn entity_kind() {
        let atom = Id::from_raw(ATOM_RAW).unwrap();
        let bond = Id::from_raw(BOND_RAW).unwrap();
        let mol = Id::from_raw(MOLECULE_RAW).unwrap();
        assert_eq!(atom.kind(), EntityKind::Atom);
        assert_eq!(bond.kind(), EntityKind::Bond);
        assert_eq!(mol.kind(), EntityKind::Molecule);
    }
}
