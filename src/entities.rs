// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::iter::FusedIterator;

use slotmap::basic::Keys;

use crate::{MolMapError, MolMapResult};

mod id;
use id::Id;

/// The kind of an entity.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(u8)]
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
    const fn from_u8(value: u8) -> Option<Self> {
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

mod private {
    /// A type to seal trait methods that should only be used within the module.
    #[derive(Copy, Clone)]
    pub struct Token;
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
pub trait Entity: Copy + Clone {
    // It is important that this trait remains sealed! Any kinds of entity that
    // molmap doesn't know about will lead to problems. It being sealed is also
    // relied upon by traits that have this as a supertrait e.g. the entity
    // category traits (Bondable, Atomlike etc.).
    //
    // What makes this trait sealed currently:
    // 1. `Id` is not nameable by other crates
    // 2. `private::Token` is not nameable by other crates and it is not possible
    //    to obtain one by any means other than construction
    //
    // Foreign types cannot implement new_unchecked or into_inner and therefore
    // cannot implement the trait. External code is also not able to access the
    // inner Id in any way.

    /// Creates a new ID for the requested kind of entity without checking that
    /// the discriminant of the ID is correct for that kind.
    #[doc(hidden)]
    fn new_unchecked(id: Id, _: private::Token) -> Self;

    /// Returns the underlying unique 64-bit ID of the entity.
    #[doc(hidden)]
    fn into_inner(self, _: private::Token) -> Id;

    /// Returns the corresponding kind of the entity.
    fn kind(&self) -> EntityKind {
        self.into_inner(private::Token).kind()
    }

    /// Upcasts the specific entity type to a dynamic type representing any entity.
    ///
    /// The kind of the entity remains encoded in the ID itself and can be recovered
    /// dynamically at runtime using [`Entity::kind`] or [`resolve`].
    fn as_entity(self) -> AnyEntity {
        AnyEntity(self.into_inner(private::Token))
    }

    /// Returns the concrete type appropriate for the specific kind of entity, wrapped
    /// in an enum where the variant corresponds to its kind.
    ///
    /// This method achieves the same as `self.as_entity().resolve()`, but may have a
    /// performance advantage for concrete entity types (i.e. those that implement
    /// [`Kind`]), as they do not require any dynamic resolution of the entity's kind,
    /// while conversion of the intermediate [`AnyEntity`] involves a runtime bitfield
    /// check.
    #[inline]
    fn to_resolved(self) -> ResolvedEntity {
        // By default go via the erased form - implementors can override if they
        // can do it more efficiently
        self.as_entity().resolve()
    }
}

/// An entity that may be of any kind.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct AnyEntity(Id);

impl AnyEntity {
    pub fn resolve(self) -> ResolvedEntity {
        match self.kind() {
            EntityKind::Atom => ResolvedEntity::Atom(Atom(self.0)),
            EntityKind::Bond => ResolvedEntity::Bond(Bond(self.0)),
            EntityKind::Pseudoatom => ResolvedEntity::Pseudoatom(Pseudoatom(self.0)),
            EntityKind::Substituent => ResolvedEntity::Substituent(Substituent(self.0)),
            EntityKind::Molecule => ResolvedEntity::Molecule(Molecule(self.0)),
        }
    }
}

impl Entity for AnyEntity {
    fn new_unchecked(id: Id, _: private::Token) -> Self {
        Self(id)
    }

    fn into_inner(self, _: private::Token) -> Id {
        self.0
    }
}

impl Category for AnyEntity {}

/// An entity of any kind, but tagged to show which specific kind.
///
/// Matching on this enum is exhaustive for all the possible kinds of entity.
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
//#[non_exhaustive]
pub enum ResolvedEntity {
    Atom(Atom) = EntityKind::Atom as u8,
    Bond(Bond) = EntityKind::Bond as u8,
    Pseudoatom(Pseudoatom) = EntityKind::Pseudoatom as u8,
    Substituent(Substituent) = EntityKind::Substituent as u8,
    Molecule(Molecule) = EntityKind::Molecule as u8,
}

/// A union type for iterators over entity IDs from various sources.
enum EntitiesSource<'m, E: Kind> {
    Keys(Keys<'m, E::Key, E::Data>),
    Vec(std::vec::IntoIter<E>),
}

impl<'m, E: Kind> Iterator for EntitiesSource<'m, E> {
    type Item = E;

    fn next(&mut self) -> Option<E> {
        match self {
            Self::Keys(keys) => keys.next().map(E::from_key),
            Self::Vec(ids) => ids.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Keys(keys) => keys.size_hint(),
            Self::Vec(ids) => ids.size_hint(),
        }
    }
}

impl<'m, E: Kind> ExactSizeIterator for EntitiesSource<'m, E> {}

impl<'m, E: Kind> FusedIterator for EntitiesSource<'m, E> {}

/// An iterator over entity IDs.
pub struct Entities<'m, E: Kind>(EntitiesSource<'m, E>);

impl<'m, E: Kind> Entities<'m, E> {
    pub(crate) fn from_keys(keys: Keys<'m, E::Key, E::Data>) -> Self {
        Self(EntitiesSource::Keys(keys))
    }

    pub(crate) fn from_vec(vec: Vec<E>) -> Self {
        Self(EntitiesSource::Vec(vec.into_iter()))
    }
}

impl<'m, E: Kind> Iterator for Entities<'m, E> {
    type Item = E;

    fn next(&mut self) -> Option<E> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'m, E: Kind> ExactSizeIterator for Entities<'m, E> {}

impl<'m, E: Kind> FusedIterator for Entities<'m, E> {}

/// A selected set or subset of entities of a given kind.
pub enum Selection<'m, E: Kind> {
    All,
    Iter(Box<dyn Iterator<Item = E> + 'm>),
}

impl<'m, E, I> From<I> for Selection<'m, E>
where
    E: Kind,
    I: IntoIterator<Item = E>,
    I::IntoIter: 'm,
{
    fn from(iterable: I) -> Self {
        Selection::Iter(Box::new(iterable.into_iter()))
    }
}

/// Items that must remain unnameable from outside the crate.
mod internal {
    use slotmap::new_key_type;

    use super::*;

    /// A kind of entity whose ID is a key in a backing `SlotMap`.
    pub trait Keyed: Entity {
        type Key: slotmap::Key + 'static;

        const KIND: EntityKind;

        fn from_key(key: Self::Key) -> Self {
            Self::new_unchecked(
                Id::from_key_data(Self::KIND, slotmap::Key::data(&key)),
                private::Token,
            )
        }

        fn to_key(self) -> Self::Key {
            Self::Key::from(self.into_inner(private::Token).to_key_data())
        }
    }

    /// A kind of entity with a corresponding struct type to hold its core data.
    pub trait Stored: Keyed {
        type Data: 'static;
    }

    new_key_type! { pub struct AtomKey; }
    new_key_type! { pub struct BondKey; }
    new_key_type! { pub struct PseudoatomKey; }
    new_key_type! { pub struct SubstituentKey; }
    new_key_type! { pub struct MoleculeKey; }
}
pub(crate) use internal::*;

/// A basic, concrete kind of entity in a `MolMap`, backed by its own storage.
///
/// All entity types implement [`Entity`] and one of either `Kind` or [`Category`],
/// according to whether the kind is known statically or only obtainable dynamically.
pub trait Kind: Entity + Keyed + Stored {}

// Kind is essentially the public-facing form of Keyed/Stored
// Blanket impl for whatever fulfils the conditions
impl<E: Entity + Keyed + Stored> Kind for E {}

macro_rules! new_entity_kind {
    (
        $(#[$doc:meta])*
        $vis:vis struct $kind:ident;
    ) => {
        paste::paste! {
            $(#[$doc])*
            #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
            pub struct $kind(pub(crate) Id);

            impl Keyed for $kind {
                type Key = [<$kind Key>];

                const KIND: EntityKind = EntityKind::$kind;
            }

            impl From<$kind> for [<$kind Key>] {
                fn from(id: $kind) -> Self {
                    id.to_key()
                }
            }

            impl From<[<$kind Key>]> for $kind {
                fn from(key: [<$kind Key>]) -> Self {
                    $kind::from_key(key)
                }
            }

            impl Entity for $kind {
                fn new_unchecked(id: Id, _: private::Token) -> Self {
                    Self(id)
                }

                fn into_inner(self, _: private::Token) -> Id {
                    self.0
                }

                fn kind(&self) -> EntityKind {
                    Self::KIND
                }

                #[inline]
                fn to_resolved(self) -> ResolvedEntity {
                    ResolvedEntity::$kind($kind(self.0))
                }
            }

            // Conversion to and from AnyEntity

            impl From<$kind> for AnyEntity {
                fn from(entity: $kind) -> AnyEntity {
                    entity.as_entity()
                }
            }

            impl TryFrom<AnyEntity> for $kind {
                type Error = MolMapError;

                fn try_from(entity: AnyEntity) -> Result<Self, Self::Error> {
                    match entity.kind() {
                        EntityKind::$kind => Ok(Self(entity.0)),
                        _ => {
                            Err(crate::error::MolMapError::IncorrectEntityKind(
                                entity.kind(),
                                entity,
                            ))
                        },
                    }
                }
            }
        }
    };
}

new_entity_kind!(
    /// Smallest particle still characterizing a chemical element.
    pub struct Atom;
);

new_entity_kind!(
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

new_entity_kind!(
    /// A pseudoatom: something that forms bonds and can be represented by an
    /// "element symbol" like a normal atom but represents something else.
    ///
    /// It may have an unknown composition like R, or a known structure like Ph.
    pub struct Pseudoatom;
);

new_entity_kind!(
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

new_entity_kind!(
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

/// A dynamic ID type representing any kind of entity that implements the corresponding trait.
///
/// All entity types implement [`Entity`] and one of either [`Kind`] or `Category`,
/// according to whether the kind is known statically or only obtainable dynamically.
pub trait Category: Entity {
    /// Attempts to convert the dynamic entity type to a concrete one, failing if the
    /// entity is not of the corresponding kind.
    fn downcast<E: Kind>(self) -> MolMapResult<E> {
        if self.kind() == E::KIND {
            Ok(E::new_unchecked(
                self.into_inner(private::Token),
                private::Token,
            ))
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
                    [<Any $category>](self.into_inner(private::Token))
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
                        $(EntityKind::$kind => [<Resolved $category>]::$kind($kind(self.0)),)+
                        _ => unreachable!(),
                    }
                }
            }

            impl Entity for [<Any $category>] {
                fn new_unchecked(id: Id, _: private::Token) -> Self {
                    Self(id)
                }

                fn into_inner(self, _: private::Token) -> Id {
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
                        $(Self::$kind(concrete) => concrete.0,)+
                    };
                    [<Any $category>](inner)
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
                        Self(entity.0)
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
                        $(EntityKind::$kind => Ok(Self(entity.0)),)+
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
                    Self(entity.0)
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
#[allow(unused)]
#[allow(clippy::unusual_byte_groupings)]
mod tests {
    use std::num::NonZeroU64;

    use slotmap::SlotMap;

    //use crate::graph::keys::BondKey;

    use super::*;

    // Some raw keys for use when testing
    const ATOM_NULL_RAW: u64 = 0x0000001_00_FFFFFF; // The null atom key
    const KD_NULL_RAW: u64 = 0x0000001_FF_FFFFFF; // What KeyData considers to be null

    const BOND_RAW: u64 = 0x1_01_000008; // version: 1, kind: Bond, idx: 8
    const ATOM_RAW: u64 = 0x3_00_000010; // version: 3, kind: Atom, idx: 16, (version always odd for occupied slots)
    const PSEUDOATOM_RAW: u64 = 0x1_02_00000A; // version: 1, kind: Pseudoatom, idx: 10
    const MOLECULE_RAW: u64 = 0x1_1F_000001; // version: 1, kind: Molecule, idx: 1

    const BOND: Bond = Bond(Id::from_raw(BOND_RAW).unwrap());
    const ATOM: Atom = Atom(Id::from_raw(ATOM_RAW).unwrap());
    const PSEUDOATOM: Pseudoatom = Pseudoatom(Id::from_raw(PSEUDOATOM_RAW).unwrap());
    const MOLECULE: Molecule = Molecule(Id::from_raw(MOLECULE_RAW).unwrap());

    #[test]
    fn key_kind() {
        let atom = ATOM;
        let bond = BOND;
        let mol = MOLECULE;
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

    use crate::entities::id::Id;

    use super::*;

    #[test]
    fn category_kind() {
        let atomlike = AnyAtomlike(ATOM.0);
        let fundamental = AnyFundamental(BOND.0);
        let collection = AnyCollection(MOLECULE.0);
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
        assert_eq!(ATOM.0, atomlike.0);
        // Can be converted back to keyed ID form without issue, still the same
        assert_eq!(Atom(atomlike.0), ATOM);
    }

    #[test]
    fn convert_key_to_category_fails() {
        // Conversion of a Bond to an AnyAtomlike is forbidden, because Bond isn't Atomlike
        // There's simply no From implementation
        // It can be attempted via the Entity, but it should fail
        assert!(AnyAtomlike::try_from(BOND.as_entity()).is_err());
    }

    #[test]
    fn convert_category_to_key() {
        let atom: AnyFundamental = ATOM.into();
        let bond: AnyFundamental = BOND.into();
        // Conversion should work when the attempted conversion aligns with the kind
        assert!(Atom::try_from(atom).is_ok());
        assert!(Bond::try_from(bond).is_ok());
        // But not otherwise
        assert!(Bond::try_from(atom).is_err());
        assert!(Atom::try_from(bond).is_err());
        assert!(Pseudoatom::try_from(atom).is_err());
        assert!(Pseudoatom::try_from(bond).is_err());
    }

    #[test]
    fn convert_key_cat_key_round_trip() {
        // Bond to Fundamental works, as does round trip
        let bond = BOND;
        assert_eq!(
            AnyFundamental::from(bond),
            AnyFundamental(Id::from_raw(BOND_RAW).unwrap())
        );
        assert_eq!(Bond::try_from(AnyFundamental::from(bond)).unwrap(), bond);
        // Molecule to Collection to Entity to Molecule should all work
        let col: AnyCollection = MOLECULE.into();
        let ent: AnyEntity = col.into();
        let recovered: Molecule = ent.try_into().unwrap();
        assert_eq!(recovered, MOLECULE);
        assert_eq!(ent, MOLECULE.as_entity());
    }

    #[test]
    fn convert_between_categories() {
        let atom: AnyAtomlike = ATOM.into();
        let pseudoatom: AnyAtomlike = PSEUDOATOM.into();
        assert_eq!(AnyFundamental::from(atom), AnyFundamental(ATOM.0));
        assert_eq!(
            AnyFundamental::from(pseudoatom),
            AnyFundamental(PSEUDOATOM.0)
        );
        assert_eq!(AnyBondable::from(atom), AnyBondable(ATOM.0));
        assert_eq!(AnyBondable::from(pseudoatom), AnyBondable(PSEUDOATOM.0));
    }

    #[test]
    fn convert_between_partially_overlapping() {
        let atom: AnyFundamental = ATOM.into();
        let pseudoatom: AnyFundamental = PSEUDOATOM.into();
        let bond: AnyFundamental = BOND.into();
        // Conversion succeeds when the resolved kind is in both categories
        assert!(AnyAtomlike::try_from(atom).is_ok());
        assert!(AnyAtomlike::try_from(pseudoatom).is_ok());
        // Fails otherwise
        assert!(AnyAtomlike::try_from(bond).is_err());
    }

    #[test]
    fn resolve() {
        // Mostly want to check the ergonomics of getting a tagged representation
        match Atomlike::to_resolved(ATOM) {
            ResolvedAtomlike::Atom(atom) => assert_eq!(atom, ATOM),
            ResolvedAtomlike::Pseudoatom(_) => panic!(),
        }
    }
}
