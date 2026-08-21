// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{Entity, EntityKind, Id, entities::*};

///// A category encompassing multiple kinds of entity with shared behaviour.
//pub trait Category: Entity {
//    const KINDS: &[EntityKind];
//
//    /// Returns `true` if the given kind of entity falls in this category.
//    fn valid_kind(kind: EntityKind) -> bool {
//        Self::KINDS.contains(&kind)
//    }
//}

// Trait for infallible conversion from an ID of any entity that implements
// the $category trait to the type-erased ID (`Id<$categoryEntity>`) via a
// blanket implementation
//
// Can't blanket implement From as $categoryEntity itself also implements $category,
// so would cause a conflict with std's blanket `impl<T> From<T> for T`

pub(crate) trait IntoCategory<C: Entity>: Sized {
    type Tagged: From<Id<C>>;

    fn to_erased(self) -> Id<C>;

    fn to_tagged(self) -> Self::Tagged {
        self.to_erased().into()
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

            // A trait to be implemented by entity types

            $(#[$doc])*
            pub trait $category: Entity {}

            // Implement it for each kind of entity specified

            $(
                impl $category for $kind {}
            )+

            // A union marker type for representing a type-erased entity

            #[doc = concat!("A concrete type representing any entity that implements the [`", stringify!([<$category>]), "`] trait.")]
            #[derive(Copy, Clone, Debug)]
            pub struct [<$category Entity>];

            impl Entity for [<$category Entity>] {}

            // Implement the Category trait to mark that it is one of these special
            // union marker types
            // (Is this necessary though, since it doesn't implement KeyEntity anyway?)

            //impl Category for [<$category Entity>] {
            //    const KINDS: &[EntityKind] = &[
            //        $(EntityKind::$kind,)+
            //    ];
            //}

            // The trait is also implemented for the union marker type

            impl $category for [<$category Entity>] {}

            // Infallible conversion from each category member to the category type
            // NO LONGER NEEDED because Into<$category> is no longer the way of showing
            // that $kind "is" $category, instead $kind implements $category
            //$(
            //    impl From<$kind> for [<$category Entity>] {
            //        fn from(kind: $kind) -> Self {
            //            Self
            //        }
            //    }
            //)+

            // Trait for infallible conversion from an ID of any entity that implements
            // the $category trait to the type-erased ID (`Id<$categoryEntity>`) via a
            // blanket implementation
            //
            // Can't blanket implement From as $categoryEntity itself also implements $category,
            // so would cause a conflict with std's blanket `impl<T> From<T> for T`
            //
            // As EraseIdKind is a local trait, we have no such problem with a blanket impl for it

            impl<E: $category> IntoCategory<[<$category Entity>]> for Id<E> {
                type Tagged = [<Tagged $category>];

                fn to_erased(self) -> Id<[<$category Entity>]> {
                    Id::new_unchecked(self.into_inner())
                }
            }

            // Infallible conversion with From still implemented for use by users,
            // just not in a blanket fashion but on a kind-by-kind basis.
            // This means the user can feel assured that they can call `into()` on
            // an ID of any entity that implements the trait - it's just that the
            // _compiler_ doesn't know that, and that's what the EraseIdKind trait
            // is then for.

            $(
                impl From<Id<$kind>> for Id<[<$category Entity>]> {
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

            impl Id<[<$category Entity>]> {
                pub fn to_tagged(self) -> [<Tagged $category>] {
                    match self.kind() {
                        $(EntityKind::$kind => [<Tagged $category>]::$kind(Id::<$kind>::new_unchecked(self.into_inner())),)+
                        _ => unreachable!(),
                    }
                }
            }

            impl From<Id<[<$category Entity>]>> for [<Tagged $category>] {
                fn from(id: Id<[<$category Entity>]>) -> Self {
                    id.to_tagged()
                }
            }
        }
    };
}

define_category! {
    /// An atom or something that behaves like one (a pseudoatom).
    ///
    /// Atomlike entities are the true nodes of the molecular graph.
    Atomlike {
        Atom,
        Pseudoatom,
    }
}

define_category! {
    /// An entity that does not group other entities.
    ///
    /// Fundamental entities are the basic building blocks of a [`MolMap`].
    ///
    /// Atoms, pseudoatoms, and bonds are fundamental entities.
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

//impl<E: Atomlike> IntoCategory<FundamentalEntity> for Id<E> {
//    type Tagged = TaggedFundamental;
//
//    fn erase(self) -> Id<FundamentalEntity> {
//        Id::new_unchecked(self.into_inner())
//    }
//}

impl From<Id<AtomlikeEntity>> for Id<FundamentalEntity> {
    fn from(id: Id<AtomlikeEntity>) -> Self {
        Id::new_unchecked(id.into_inner())
    }
}

//impl<E: Atomlike> IntoCategory<BondableEntity> for Id<E> {
//    type Tagged = TaggedBondable;
//
//    fn erase(self) -> Id<BondableEntity> {
//        Id::new_unchecked(self.into_inner())
//    }
//}

impl From<Id<AtomlikeEntity>> for Id<BondableEntity> {
    fn from(id: Id<AtomlikeEntity>) -> Self {
        Id::new_unchecked(id.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use crate::id::EntityId;

    use super::*;

    // Some raw keys for use when testing
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

    //#[test]
    //fn tagged() {
    //    // Mostly want to test the ergonomics of getting a tagged representation
    //    let tagged: TaggedAtomlike = IntoCategory::to_tagged(ATOM);
    //    match tagged {
    //        TaggedAtomlike::Atom(_) => (),
    //        TaggedAtomlike::Pseudoatom(_) => panic!(),
    //    }
    //    match ATOM.to_tagged() {
    //        TaggedAtomlike::Atom(_) => (),
    //        TaggedAtomlike::Pseudoatom(_) => panic!(),
    //    }
    //}
}
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
