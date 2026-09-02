// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Definitions and implementations of the entity category traits.

use crate::entities::*;
use crate::error::{MolMapError, MolMapResult};
use crate::id::Id;

pub use crate::entities::{AnyEntity, Entity, ResolvedEntity};

/// A dynamic ID type representing any kind of entity that implements the corresponding trait.
///
/// All entity types implement [`Entity`] and one of either [`Kind`] or `Category`,
/// according to whether the kind is known statically or only obtainable dynamically.
pub trait Category: Entity {
    /// Attempts to convert the dynamic entity type to a concrete one, failing if the
    /// entity is not of the corresponding kind.
    fn downcast<E: Kind>(self) -> MolMapResult<E> {
        if self.kind() == E::KIND {
            Ok(E::new_unchecked(self.into_inner()))
        } else {
            Err(crate::error::MolMapError::IncorrectEntityKind(
                self.kind(),
                self.as_entity(),
            ))
        }
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
            // These are equivalent to the triad of Entity/AnyEntity/ResolvedEntity for
            // specific subsets. Indeed, Entity should act like any other Category.

            // 1. The trait, to be implemented by entity types

            $(#[$doc])*
            pub trait $category: Entity {
                #[doc = concat!("Upcasts the specific entity type to a dynamic type representing any kind of [`", stringify!([<$category>]), "`] entity.")]
                #[doc = ""]
                #[doc = "The kind of the entity remains encoded in the ID itself and can be recovered dynamically at runtime using [`Entity::kind`] or [`resolve`]."]
                fn [<as_ $category:lower>](self) -> [<Any $category>] {
                    [<Any $category>](self.into_inner())
                }

                #[doc = "Returns the appropriate concrete entity type wrapped in an enum where the variant corresponds to its kind."]
                #[doc = ""]
                #[doc = concat!("This method achieves the same as `self.", stringify!([<as_ $category:lower>]), "().resolve()`, but may have a ")]
                #[doc = "performance advantage for concrete entity types (i.e. those that implement "]
                #[doc = "[`Kind`]), as they do not require any dynamic resolution of the entity's kind, "]
                #[doc = concat!("while conversion of the intermediate [`", stringify!([<Any $category>]), "`] involves a runtime bitfield ")]
                #[doc = "check."]
                #[inline]
                fn to_resolved(self) -> [<Resolved $category>] {
                    // By default go via the erased form - implementors can override if they
                    // can do it more efficiently
                    self.[<as_ $category:lower>]().resolve()
                }
            }

            // 2. A category struct for representing a kind-erased entity of the category

            #[doc = concat!("An entity that may be of any kind that implements the [`", stringify!([<$category>]), "`] trait.")]
            #[doc = ""]
            #[doc = "The kind of the entity remains encoded in the ID itself and can be recovered dynamically at runtime using [`Entity::kind`] or [`resolve`]."]
            #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
            pub struct [<Any $category>](pub(crate) Id);

            impl [<Any $category>] {
                pub fn resolve(self) -> [<Resolved $category>] {
                    match self.kind() {
                        $(EntityKind::$kind => [<Resolved $category>]::$kind($kind::new_unchecked(self.0)),)+
                        _ => unreachable!(),
                    }
                }
            }

            impl Entity for [<Any $category>] {
                fn new_unchecked(id: Id) -> Self {
                    Self(id)
                }

                fn into_inner(self) -> Id {
                    self.0
                }
            }

            impl Category for [<Any $category>] {}

            // 3. The corresponding tagged ID type for exhaustive matching
            // The enum has a variant for each kind that implements the trait

            #[doc = concat!("An entity of any kind that implements the [`", stringify!([<$category>]), "`] trait, but tagged to show which specific kind.")]
            #[doc = ""]
            #[doc = "Matching on this enum is exhaustive for all the possible kinds of entity that it could be."]
            #[derive(Copy, Clone, Debug)]
            pub enum [<Resolved $category>] {
                $($kind($kind),)+
            }

            impl [<Resolved $category>] {
                #[doc = concat!("Reverses the resolution to afford the dynamic type representing any kind of [`", stringify!([<$category>]), "`] entity.")]
                pub fn [<as_ $category:lower>](self) -> [<Any $category>] {
                    let inner = match self {
                        $(Self::$kind(concrete) => concrete.into_inner(),)+
                    };
                    [<Any $category>]::new_unchecked(inner)
                }
            }

            // Now, implement the trait for each kind of entity specified

            $(
                impl $category for $kind {
                    fn to_resolved(self) -> [<Resolved $category>] {
                        // Unlike conversion of the union type, this is trivial and
                        // low-cost because we know what the kind is based on the type
                        [<Resolved $category>]::$kind(self)
                    }
                }
            )+

            // Also implement the trait for the erased struct type for consistency

            impl $category for [<Any $category>] {}

            // Infallible conversion with From for use by users.
            // These just replicate the conversions available via the trait's
            // `as_Trait` and `to_resolved` methods.
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

                impl From<$kind> for [<Resolved $category>] {
                    fn from(entity: $kind) -> Self {
                        $category::to_resolved(entity)
                    }
                }
            )+

            // Infallible conversion between struct and enum forms.

            impl From<[<Any $category>]> for [<Resolved $category>] {
                fn from(entity: [<Any $category>]) -> Self {
                    $category::to_resolved(entity)
                }
            }

            impl From<[<Resolved $category>]> for [<Any $category>] {
                fn from(resolved: [<Resolved $category>]) -> Self {
                    resolved.[<as_ $category:lower>]()
                }
            }

            // Fallible conversion with TryFrom for use by users.

            $(
                impl TryFrom<[<Any $category>]> for $kind {
                    type Error = MolMapError;

                    fn try_from(entity: [<Any $category>]) -> Result<Self, Self::Error> {
                        entity.downcast()
                    }
                }
            )+

            // Conversion to and from AnyEntity

            impl From<[<Any $category>]> for AnyEntity {
                fn from(entity: [<Any $category>]) -> AnyEntity {
                    entity.as_entity()
                }
            }

            impl TryFrom<AnyEntity> for [<Any $category>] {
                type Error = MolMapError;

                fn try_from(entity: AnyEntity) -> Result<Self, Self::Error> {
                    match entity.kind() {
                        $(EntityKind::$kind => Ok(Self::new_unchecked(entity.into_inner())),)+
                        _ => {
                            Err(crate::error::MolMapError::IncorrectEntityKind(
                                entity.kind(),
                                entity.into(),
                            ))
                        },
                    }
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

//define_category! {
//    /// An entity that an `Object` can be attached to.
//    Anchor {
//        Atom,
//        Pseudoatom,
//        Bond,
//        Substituent,
//        Molecule,
//    }
//}

// Some additional overlaps

/// Implements traits as appropriate for categories `A` and `B`, where `A` is a strict subset of `B`.
///
/// Any entity that is `A` is also `B`.
/// Reflecting this, any entity kind type that implements `A` should already
/// implement `B`.
///
/// However, as these are not done (and cannot be done) using blanket
/// implementations, the compiler does not know about this relationship.
/// Therefore, to assist with category narrowing and broadening, this macro
/// implements the following additional traits:
///
/// 1. `impl B for AnyA` (anything that is `A` is also `B`)
/// 2. `impl From<AnyA> for AnyB` (infallible conversion to reflect that fact)
/// 3. `impl TryFrom<AnyB> for AnyA` (fallible conversion, as there are kinds of
///    entity that are `B` but not `A`, but a useful one due to the overlap)
macro_rules! impl_subset {
    ($A:ident < $B:ident) => {
        paste::paste! {
            impl $B for [<Any $A>] {}

            impl From<[<Any $A>]> for [<Any $B>] {
                fn from(entity: [<Any $A>]) -> Self {
                    Self::new_unchecked(entity.into_inner())
                }
            }

            impl TryFrom<[<Any $B>]> for [<Any $A>] {
                type Error = MolMapError;

                fn try_from(entity: [<Any $B>]) -> Result<Self, Self::Error> {
                    [<Any $A>]::try_from(entity.as_entity())
                }
            }
        }
    };
}

impl_subset!(Atomlike < Fundamental);

impl_subset!(Atomlike < Bondable);

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
    fn resolve() {
        // Mostly want to check the ergonomics of getting a tagged representation
        let tagged: ResolvedAtomlike = Atomlike::to_resolved(ATOM);
        match tagged {
            ResolvedAtomlike::Atom(_) => (),
            ResolvedAtomlike::Pseudoatom(_) => panic!(),
        }
    }
}
