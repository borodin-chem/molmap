// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{Entity, EntityKind, Id, entities::*};

/// A category encompassing multiple kinds of entity with shared behaviour.
pub trait Category: Entity {
    const KINDS: &[EntityKind];

    /// Returns `true` if the given kind of entity falls in this category.
    fn valid_kind(kind: EntityKind) -> bool {
        Self::KINDS.contains(&kind)
    }
}

macro_rules! define_category {
    (
        $(#[$doc:meta])*
        $category:ident {
            $($kind:ident),+ $(,)?
        }
    ) => {
        paste::paste! {
            #[allow(non_snake_case)]

            $(#[$doc])*
            #[derive(Copy, Clone, Debug)]
            pub struct $category;

            impl Entity for $category {}

            impl Category for $category {
                const KINDS: &[EntityKind] = &[
                    $(EntityKind::$kind,)+
                ];
            }

            // Infallible conversion from each category member to the category
            $(
                impl From<$kind> for $category {
                    fn from(kind: $kind) -> Self {
                        Self
                    }
                }
            )+

            // Likewise, infallible conversion from each category member's ID
            $(
                impl From<Id<$kind>> for Id<$category> {
                    fn from(id: Id<$kind>) -> Self {
                        Self::new_unchecked(id.into_inner())
                    }
                }
            )+

            // A corresponding tagged ID type for each category for exhaustive matching
            $(#[$doc])*
            #[derive(Copy, Clone, Debug)]
            pub enum [<Tagged $category>] {
                $($kind(Id<$kind>),)+
            }

            // Conversion of the type-erased ID to the tagged ID
            impl From<Id<$category>> for [<Tagged $category>] {
                fn from(id: Id<$category>) -> Self {
                    match id.kind() {
                        $(EntityKind::$kind => Self::$kind(Id::<$kind>::new_unchecked(id.into_inner())),)+
                        _ => unreachable!(),
                    }
                }
            }

            impl Id<$category> {
                pub fn to_tagged(self) -> [<Tagged $category>] {
                    self.into()
                }
            }
        }
    };
}

define_category! {
    /// An atom or something that behaves like one (a pseudoatom).
    ///
    /// Atomlikes are the true nodes of the molecular graph.
    Atomlike {
        Atom,
        Pseudoatom,
    }
}

define_category! {
    /// An entity that does not group other entities.
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

define_category! {
    /// An aggregation of fundamental entities.
    Collection {
        Substituent,
        Molecule,
    }
}

define_category! {
    /// An entity that can form bonds.
    Bondable {
        Atom,
        Pseudoatom,
        //Bond,
    }
}

define_category! {
    /// An entity that an `Object` can be attached to.
    Anchor {
        Atom,
        Pseudoatom,
        Bond,
        Substituent,
        Molecule,
    }
}

// Some additional overlaps

impl From<Atomlike> for Fundamental {
    fn from(atomlike: Atomlike) -> Fundamental {
        Fundamental
    }
}

impl From<Id<Atomlike>> for Id<Fundamental> {
    fn from(id: Id<Atomlike>) -> Id<Fundamental> {
        Id::new_unchecked(id.into_inner())
    }
}

impl From<Atomlike> for Bondable {
    fn from(atomlike: Atomlike) -> Bondable {
        Bondable
    }
}

impl From<Id<Atomlike>> for Id<Bondable> {
    fn from(id: Id<Atomlike>) -> Id<Bondable> {
        Id::new_unchecked(id.into_inner())
    }
}

//#[cfg(test)]
//mod tests {
//    use std::num::NonZeroU64;
//
//    use crate::id::EntityId;
//
//    use super::*;
//
//    // Some raw keys for use when testing
//    const ATOM_NULL_RAW: u64 = 0x0000001_00_FFFFFF; // The null atom key
//    const KD_NULL_RAW: u64 = 0x0000001_FF_FFFFFF; // What KeyData considers to be null
//
//    const BOND_RAW: NonZeroU64 = NonZeroU64::new(0x1_01_000008).unwrap(); // version: 1, kind: Bond, idx: 8
//    const ATOM_RAW: NonZeroU64 = NonZeroU64::new(0x3_00_000010).unwrap(); // version: 3, kind: Atom, idx: 16, (version always odd for occupied slots)
//    const PSEUDOATOM_RAW: NonZeroU64 = NonZeroU64::new(0x1_03_00000A).unwrap(); // version: 1, kind: Pseudoatom, idx: 10
//    const MOL_RAW: NonZeroU64 = NonZeroU64::new(0x1_1F_000001).unwrap(); // version: 1, kind: Molecule, idx: 1
//
//    const BOND: Id<Bond> = Id::new_unchecked(EntityId(BOND_RAW));
//    const ATOM: Id<Atom> = Id::new_unchecked(EntityId(ATOM_RAW));
//    const PSEUDOATOM: Id<Pseudoatom> = Id::new_unchecked(EntityId(PSEUDOATOM_RAW));
//    const MOL: Id<Molecule> = Id::new_unchecked(EntityId(MOL_RAW));
//
//    #[test]
//    fn category_kind() {
//        let atomlike = Id::<Atomlike>::new_unchecked(EntityId(ATOM_RAW));
//        let fundamental = Id::<Fundamental>::new_unchecked(EntityId(BOND_RAW));
//        let collection = Id::<Collection>::new_unchecked(EntityId(MOL_RAW));
//        assert_eq!(atomlike.kind(), EntityKind::Atom);
//        assert_eq!(fundamental.kind(), EntityKind::Bond);
//        assert_eq!(collection.kind(), EntityKind::Molecule);
//    }
//
//    #[test]
//    fn convert_key_to_category() {
//        let atom = ATOM;
//        // Conversion to an Atomlike is infallible
//        let atomlike: Id<Atomlike> = atom.into();
//        // ID stays the same
//        assert_eq!(atom.into_inner(), atomlike.into_inner());
//        // Underlying key is the same
//        assert_eq!(
//            atom.into_inner().to_raw_key(),
//            atomlike.into_inner().to_raw_key()
//        );
//        // Conversion via the Entity works too
//        //assert_eq!(
//        //    Id<Atomlike>::try_from(EntityId::from(atom)).unwrap(),
//        //    atomlike
//        //);
//    }
//
//    #[test]
//    fn convert_key_to_category_fails() {
//        // TODO Consider whether to remove this test or to reinstate the TryFrom impl
//        //
//        // Conversion of a Bond to an Atomlike is forbidden
//        // There's simply no From implementation
//        // It can be attempted via the Entity, but it should fail
//        //let bond = BOND;
//        //let attempt = Id<Atomlike>::try_from(EntityId::from(bond));
//        //assert!(Id<Atomlike>::try_from(EntityId::from(bond)).is_err());
//    }
//
//    #[test]
//    fn convert_category_to_key() {
//        let atom: Id<Fundamental> = ATOM.into();
//        let bond: Id<Fundamental> = BOND.into();
//        // Conversion should work when the attempted conversion aligns with the kind
//        assert!(Id::<Atom>::try_from(atom).is_ok());
//        assert!(Id::<Bond>::try_from(bond).is_ok());
//        // But not otherwise
//        assert!(Id::<Bond>::try_from(atom).is_err());
//        assert!(Id::<Atom>::try_from(bond).is_err());
//        assert!(Id::<Pseudoatom>::try_from(atom).is_err());
//        assert!(Id::<Pseudoatom>::try_from(bond).is_err());
//    }
//
//    #[test]
//    fn convert_key_cat_key_round_trip() {
//        // Bond to Fundamental works, as does round trip
//        let bond = BOND;
//        assert_eq!(
//            Id::<Fundamental>::from(bond),
//            Id::<Fundamental>::new_unchecked(EntityId(BOND_RAW))
//        );
//        assert_eq!(
//            Id::<Bond>::try_from(Id::<Fundamental>::from(bond)).unwrap(),
//            bond
//        );
//        // Molecule to Collection to Entity to Molecule should all work
//        let mol = MOL;
//        let col: Id<Collection> = mol.into();
//        let ent: EntityId = col.into();
//        let recovered: Id<Molecule> = ent.try_into().unwrap();
//        assert_eq!(ent, mol.0);
//        assert_eq!(recovered, mol);
//    }
//
//    #[test]
//    fn convert_between_categories() {
//        let atom = Id::<Atomlike>::from_raw_unchecked(ATOM_RAW);
//        let pseudoatom = Id::<Atomlike>::from_raw_unchecked(PSEUDOATOM_RAW);
//        assert_eq!(
//            Id::<Fundamental>::from(atom),
//            Id::<Fundamental>::from_raw_unchecked(ATOM_RAW)
//        );
//        assert_eq!(
//            Id::<Fundamental>::from(pseudoatom),
//            Id::<Fundamental>::from_raw_unchecked(PSEUDOATOM_RAW)
//        );
//        assert_eq!(
//            Id::<Bondable>::from(atom),
//            Id::<Bondable>::from_raw_unchecked(ATOM_RAW)
//        );
//        assert_eq!(
//            Id::<Bondable>::from(pseudoatom),
//            Id::<Bondable>::from_raw_unchecked(PSEUDOATOM_RAW)
//        );
//    }
//}
