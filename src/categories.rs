// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::entities::*;
use crate::id::Id;

macro_rules! define_category {
    (
        $(#[$doc:meta])*
        $category:ident {
            $($kind:ident),+ $(,)?
        }
    ) => {
        paste::paste! {

            // This macro defines three things:
            //
            // 1. A trait, which is implemented by Entity types to indicate
            //    that they have a particular property
            // 2. An Entity struct (i.e. an ID) that represents any entity with that property
            //    but with the concrete type erased (a bit like a trait object)
            // 3. An enum that indicates the kind of entity and wraps the concrete Entity
            //    type, which can be obtained from (2), or a trait object, or an
            //    [anonymous/abstract type](https://doc.rust-lang.org/reference/types/impl-trait.html),
            //    in order to recover the kind of entity and the true underlying Entity type
            //
            // These are equivalent to the triad of Entity/AnyEntity/TaggedEntity for
            // specific subsets.

            // 1. The trait, to be implemented by entity types

            $(#[$doc])*
            pub trait $category: Entity {
                #[doc = "Erases the specific entity type."]
                #[doc = ""]
                #[doc = "The kind of the entity remains encoded in the ID and can be recovered at runtime using [`Entity::kind`] or [`to_tagged`]."]
                fn [<as_ $category:lower>](self) -> [<Any $category>] {
                    [<Any $category>](self.into_inner())
                }

                #[doc = "Returns the specific entity type wrapped in an enum variant, for exhaustive matching."]
                fn [<as_tagged_ $category:lower>](self) -> [<Tagged $category>];
            }


            // 2. A union entity type for representing a type-erased entity

            #[doc = concat!("An entity that may be of any kind that implements the [`", stringify!([<$category>]), "`] trait.")]
            #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
            pub struct [<Any $category>](pub(crate) Id);

            impl Entity for [<Any $category>] {
                fn new_unchecked(id: Id) -> Self {
                    Self(id)
                }

                fn into_inner(self) -> Id {
                    self.0
                }
            }

            // 3. The corresponding tagged ID type for exhaustive matching
            // The enum has a variant for each kind that implements the trait

            #[doc = concat!("An entity of a kind that implements the [`", stringify!([<$category>]), "`] trait, tagged to show which specific kind.")]
            #[doc = ""]
            #[doc = "Matching on this enum is exhaustive for all the possible kinds of entity that it could be."]
            #[derive(Copy, Clone, Debug)]
            pub enum [<Tagged $category>] {
                $($kind($kind),)+
            }

            // Now, implement the trait for each kind of entity specified

            $(
                impl $category for $kind {
                    fn [<as_tagged_ $category:lower>](self) -> [<Tagged $category>] {
                        // Unlike conversion of the union type, this is trivial and
                        // low-cost because we know what the kind is based on the type
                        [<Tagged $category>]::$kind(self)
                    }
                }
            )+

            // Also implement it for the union type for consistency

            impl $category for [<Any $category>] {
                fn [<as_tagged_ $category:lower>](self) -> [<Tagged $category>] {
                    match self.0.kind() {
                        $(EntityKind::$kind => [<Tagged $category>]::$kind($kind::new_unchecked(self.0)),)+
                        _ => unreachable!(),
                    }
                }
            }

            // Infallible conversion with From for use by users.
            // These just replicate the conversions available via the trait's
            // `as_Trait` and `to_tagged` methods.
            // Can't be a blanket implementation due to the orphan rule, but we
            // can do it on a kind-by-kind basis.
            // This means the user can feel assured that they can call `into()` on
            // an ID of any entity that implements the trait - it's just that the
            // _compiler_ doesn't know that.

            $(
                impl From<$kind> for [<Any $category>] {
                    fn from(entity: $kind) -> Self {
                        Self::new_unchecked(entity.into_inner())
                    }
                }

                impl From<$kind> for [<Tagged $category>] {
                    fn from(entity: $kind) -> Self {
                        $category::[<as_tagged_ $category:lower>](entity)
                    }
                }
            )+

            impl From<[<Any $category>]> for [<Tagged $category>] {
                fn from(entity: [<Any $category>]) -> Self {
                    $category::[<as_tagged_ $category:lower>](entity)
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

impl From<AnyAtomlike> for AnyFundamental {
    fn from(entity: AnyAtomlike) -> Self {
        Self::new_unchecked(entity.into_inner())
    }
}

impl From<AnyAtomlike> for AnyBondable {
    fn from(entity: AnyAtomlike) -> Self {
        Self::new_unchecked(entity.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use crate::id::Id;

    use super::*;

    // Some raw keys for use when testing
    const ATOM_NULL_RAW: u64 = 0x0000001_00_FFFFFF; // The null atom key
    const KD_NULL_RAW: u64 = 0x0000001_FF_FFFFFF; // What KeyData considers to be null

    const BOND_RAW: u64 = 0x1_01_000008; // version: 1, kind: Bond, idx: 8
    const ATOM_RAW: u64 = 0x3_00_000010; // version: 3, kind: Atom, idx: 16, (version always odd for occupied slots)
    const PSEUDOATOM_RAW: u64 = 0x1_03_00000A; // version: 1, kind: Pseudoatom, idx: 10
    const MOLECULE_RAW: u64 = 0x1_1F_000001; // version: 1, kind: Molecule, idx: 1

    const BOND: Bond = Bond(Id(NonZeroU64::new(BOND_RAW).unwrap()));
    const ATOM: Atom = Atom(Id(NonZeroU64::new(ATOM_RAW).unwrap()));
    const PSEUDOATOM: Pseudoatom = Pseudoatom(Id(NonZeroU64::new(PSEUDOATOM_RAW).unwrap()));
    const MOLECULE: Molecule = Molecule(Id(NonZeroU64::new(MOLECULE_RAW).unwrap()));

    #[test]
    fn category_kind() {
        let atomlike = AnyAtomlike::new_unchecked(ATOM.into_inner());
        let fundamental = AnyFundamental::new_unchecked(BOND.into_inner());
        let collection = AnyCollection::new_unchecked(MOLECULE.into_inner());
        assert_eq!(atomlike.kind(), EntityKind::Atom);
        assert_eq!(fundamental.kind(), EntityKind::Bond);
        assert_eq!(collection.kind(), EntityKind::Molecule);
    }

    #[test]
    fn convert_key_to_category() {
        // Can convert via the trait
        let _: AnyAtomlike = ATOM.as_atomlike();
        let _: AnyAtomlike = PSEUDOATOM.as_atomlike();
        let _: AnyBondable = ATOM.as_bondable();
        // Conversion to an Atomlike is infallible
        let atomlike: AnyAtomlike = ATOM.into();
        // ID stays the same
        assert_eq!(ATOM.into_inner(), atomlike.into_inner());
        // Can be converted back to keyed ID form without issue, still the same
        assert_eq!(Atom::new_unchecked(atomlike.into_inner()), ATOM);
    }

    //#[test]
    //fn convert_key_to_category_fails() {
    //    // TODO Consider whether to remove this test or to reinstate the TryFrom impl
    //    //
    //    // Conversion of a Bond to an Atomlike is forbidden
    //    // There's simply no From implementation
    //    // It can be attempted via the Entity, but it should fail
    //    //let bond = BOND;
    //    //let attempt = Atomlike::try_from(Id::from(bond));
    //    //assert!(Atomlike::try_from(Id::from(bond)).is_err());
    //}

    //#[test]
    //fn convert_category_to_key() {
    //    let atom: AnyFundamental = ATOM.into();
    //    let bond: AnyFundamental = BOND.into();
    //    // Conversion should work when the attempted conversion aligns with the kind
    //    assert!(Id::<Atom>::try_from(atom).is_ok());
    //    assert!(Id::<Bond>::try_from(bond).is_ok());
    //    // But not otherwise
    //    assert!(Id::<Bond>::try_from(atom).is_err());
    //    assert!(Id::<Atom>::try_from(bond).is_err());
    //    assert!(Id::<Pseudoatom>::try_from(atom).is_err());
    //    assert!(Id::<Pseudoatom>::try_from(bond).is_err());
    //}

    //#[test]
    //fn convert_key_cat_key_round_trip() {
    //    // Bond to Fundamental works, as does round trip
    //    let bond = BOND;
    //    assert_eq!(
    //        Id::<Fundamental>::from(bond),
    //        Id::<Fundamental>::new_unchecked(Id(BOND_RAW))
    //    );
    //    assert_eq!(
    //        Id::<Bond>::try_from(Id::<Fundamental>::from(bond)).unwrap(),
    //        bond
    //    );
    //    // Molecule to Collection to Entity to Molecule should all work
    //    let mol = MOL;
    //    let col: Collection = mol.into();
    //    let ent: Id = col.into();
    //    let recovered: Molecule = ent.try_into().unwrap();
    //    assert_eq!(ent, mol.0);
    //    assert_eq!(recovered, mol);
    //}

    #[test]
    fn convert_between_categories() {
        let atom = AnyAtomlike::new_unchecked(ATOM.into_inner());
        let pseudoatom = AnyAtomlike::new_unchecked(PSEUDOATOM.into_inner());
        assert_eq!(
            AnyFundamental::from(atom),
            AnyFundamental::new_unchecked(ATOM.into_inner())
        );
        assert_eq!(
            AnyFundamental::from(pseudoatom),
            AnyFundamental::new_unchecked(PSEUDOATOM.into_inner())
        );
        assert_eq!(
            AnyBondable::from(atom),
            AnyBondable::new_unchecked(ATOM.into_inner())
        );
        assert_eq!(
            AnyBondable::from(pseudoatom),
            AnyBondable::new_unchecked(PSEUDOATOM.into_inner())
        );
    }

    #[test]
    fn tagged() {
        // Mostly want to check the ergonomics of getting a tagged representation
        let tagged: TaggedAtomlike = ATOM.as_tagged_atomlike();
        match tagged {
            TaggedAtomlike::Atom(_) => (),
            TaggedAtomlike::Pseudoatom(_) => panic!(),
        }
    }
}
