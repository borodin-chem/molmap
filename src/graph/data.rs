// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The data held for the entities in the core molecular graph and the view
//! methods to access it.
//!
//! # A note on crate organization
//!
//! By the very nature of the MolMap types, the data and functionality pertaining
//! to a particular kind of entity is implemented across many different types.
//! Some data is contained in the MolGraph, in the structs defined below, while
//! some (e.g. positional information) is held by higher map types.
//! Some of the internal functions that handle the entities are defined on
//! MolGraph, some in the MolMap trait, some on the higher maps such as MolMap0
//! or SpatialMolMap, some on the entities, and a very small number of them on
//! the data structs themselves.
//! Because Rust is compiled by crate, we are presented with two good options for
//! organizing the code.
//!
//! Firstly, it would actually be possible to collect *everything* related to a
//! single entity type together, e.g. put all code for atoms in a single `atom`
//! module, and do likewise for each of the entity types. This would put the
//! definition of the corresponding Entity type, the core data struct, any
//! additional data, the methods on the different maps, the methods on the
//! different views, etc. all in a single file. General things, such as the
//! Entity trait or the macro to define a new Kind, would remain at the top level.
//!
//! The other good option, and the one currently taken, is to organize the code
//! into modules by the thing being implemented e.g. the core data, spatial data,
//! methods on MolGraph, on MolMap, on SpatialMolMap, on the MolGraph views, on
//! the general MolMap views, on SpatialMolMap views, etc.
//! If desired this approach can be further split up e.g. different modules for
//! the different kinds of functionality of a SpatialMolMap.
//!
//! The first approach would make it easier to see and change the behaviour of a
//! single kind of entity all at once, and to have a better overview of what is
//! implemented for it at each level.
//! However, it makes parallel implementation of the same or similar things for
//! multiple kinds of entity much harder.
//! It also creates a lot of circular dependencies, which while not technically a
//! problem due to the crate-level compilation do still feel kind of bad.
//! The second option also fits better the way the crate is actually used - you
//! are more likely to want to work with (and thus import) the Entity types for
//! all kinds of entity at once, or all spatial stuff for all kinds at once.
//!
//! As such, this module contains the **core data structs** for *all* entity
//! kinds in a single file, as well as any **methods on completely generic views**
//! for access and manipulation **of the core data**.
//! As the maps are generally expected to want side-effects to occur when the
//! core data is changed, the methods implemented completely generically are
//! on the whole (currently, exclusively) confined to *access* via the immutable
//! views.
//!
//! It also currently contains many types necessary for the definitions of the
//! core structs e.g. [`BondType`] and [`SubstituentCentre`], but these may well
//! be moved in the future.

use std::collections::HashSet;

use crate::{Element, MolMap, Pseudoelement, categories::*, entities::*, view::*};

/// The core data of an atom entity.
#[derive(Clone, Debug)]
pub struct AtomData {
    pub(crate) element: Element,
    pub(crate) bonds: Vec<Bond>,
}

impl AtomData {
    pub(crate) fn new(element: Element) -> Self {
        Self {
            element,
            bonds: Vec::new(),
        }
    }
}

impl<'m, M: MolMap> View<'m, M, Atom> {
    pub fn element(&self) -> Element {
        self.data().element
    }

    pub fn symbol(&self) -> &str {
        self.data().element.symbol()
    }

    pub fn bonds(&self) -> &[Bond] {
        &self.data().bonds
    }
}

/// The core data of a pseudoatom entity.
#[derive(Clone, Debug)]
pub struct PseudoatomData {
    pub(crate) pseudoelement: Pseudoelement,
    pub(crate) bonds: Vec<Bond>,
}

impl PseudoatomData {
    pub(crate) fn new(pseudoelement: Pseudoelement) -> Self {
        Self {
            pseudoelement,
            bonds: Vec::new(),
        }
    }
}

impl<'m, M: MolMap> View<'m, M, Pseudoatom> {
    pub fn bonds(&self) -> &[Bond] {
        &self.data().bonds
    }
}

/// The type of a bond e.g. covalent, ionic, hydrogen.
///
/// As in chemical structure depictions, this classification is to a large extent
/// only formal and indicates primarily how the electron distribution is to be
/// interpreted, rather than being a perfect description of the quantum chemical
/// reality.
///
/// The point at which a stablizing interaction between two entities becomes a bond can
/// be difficult to define, especially as the strength of interactions is often affected
/// by the environment. What is referred to as a "bond" and what as merely an
/// "interaction" is mostly based on the range of strengths of the class of interaction
/// rather than the strength of any individual example. Hydrogen bonds, for example, are
/// usually referred to as bonds, and while indeed they can be very strong – stronger
/// than some covalent bonds – most hydrogen bonds are quite weak interactions.
/// Meanwhile, various kinds of non-covalent interaction are almost never considered
/// bonds.
///
/// `molmap` avoids making a distinction between bonding and non-bonding interactions.
/// In `molmap`, a `Bond` entity may represent any interaction that it is desirable to
/// indicate or depict explicitly. A `Bond` may even be a hypothetical one that is yet
/// to be realized.
///
/// However, in most contexts it does not make sense to consider weak non-covalent
/// interactions as bonds. To aid this, the possible bond types are categorized as
/// "strong" and "weak", with only covalent/dipolar, metallic, and ionic considered
/// strong (as well as the catch-all `OtherStrong` variant). The [`is_strong`] method is
/// provided to test this predicate.
/// Generally, only bonds with "strong" bond types are considered edges in the graph.
///
/// In other contexts it may be preferable to only consider bonds classed as covalent
/// (which includes dipolar bonds), for which the [`is_covalent`] method is provided.
/// Only covalent/dipolar bonds affect the calculation of electron counts and formal
/// charges.
#[derive(Copy, Clone, Debug)]
pub enum BondType {
    /// A bond formed by the sharing of electrons.
    Covalent { order: f32 },
    /// A covalent bond where the bonding electrons are said to have come from
    /// only one of the two bonding partners.
    ///
    /// Obsolete terms for such a bond include "coordinate covalent" and "dative".
    ///
    /// This is the usual bonding mode in a coordination complex.
    ///
    /// The classification of a bond as dipolar is purely formal and a bond with
    /// this bond type should generally be treated the same as a covalent bond.
    ///
    /// Dipolar bonds may be depicted as an arrow pointing in the direction of
    /// the "donation", or as a normal covalent bond with the appropriate formal
    /// charges as a charge-separated structure, or – most commonly – simply as
    /// a normal covalent bond (with apparently incorrect formal charges).
    ///
    /// The Lewis-basic bonding partner that provides the bonding electrons should
    /// be `start`.
    Dipolar { order: f32 },
    /// An attractive interaction due to the electrostatic force between metal
    /// ions and surrounding delocalized electrons.
    Metallic,
    /// A strong attraction between a cation and anion where no or very little
    /// sharing of electrons occurs and the interaction is primarily electrostatic.
    Ionic,
    /// A strong bonding interaction not described adequately by any of the other
    /// categories.
    OtherStrong,
    /// An interaction between a Lewis-basic species and a hydrogen atom attached to a
    /// second, relatively electronegative atom.
    ///
    /// The term hydrogen bond is used to describe both weak, non-covalent interactions
    /// and strong bonding interactions, in some cases even with significant covalency.
    /// In `molmap` no distinction is made, but any bond with this bond type is
    /// considered "weak", regardless of actual strength.
    Hydrogen,
    /// An attractive interaction between a Lewis-basic species and a Lewis-acidic host
    /// atom bonded to a neighbour by a σ-bond.
    ///
    /// Halogen bonds are the most famous form of σ-hole interaction.
    SigmaHole,
    /// An electrostatic interaction between an ion (most commonly, a cation) and the
    /// face of a π-system.
    IonPi,
    /// An electrostatic interaction between two π-systems.
    PiPi,
    /// An electrostatic interaction between a π-system and some species other
    /// than an ion or another π-system.
    OtherPi,
    /// An electrostatic interaction between two ions that is not strong enough to be
    /// considered an ionic bond.
    IonPair,
    /// An electrostatic interaction between an ion and a permanent dipole.
    IonDipole,
    /// An electrostatic interaction between two permanent dipoles.
    DipoleDipole,
    /// A catch-all for any interactions with an induced dipole, including London
    /// dispersion forces.
    InducedDipole,
    /// An explicitly indicated non-covalent interaction not described adequately by any
    /// of the other categories.
    OtherNonCovalent,
    /// A weak interaction not described adequately by any of the other categories.
    OtherWeak,
    /// A bond that does not currently exist but could or will.
    Hypothetical,
}

impl BondType {
    /// Returns `true` if the bond type is covalent/dipolar, ionic, metallic, or
    /// the `OtherStrong` variant.
    ///
    /// Note that this excludes hydrogen bonds and σ-hole interactions, even though
    /// these can in some cases have considerable strength.
    pub fn is_strong(&self) -> bool {
        match self {
            BondType::Covalent { order } => true,
            BondType::Dipolar { order } => true,
            BondType::Metallic => true,
            BondType::Ionic => true,
            BondType::OtherStrong => true,
            _ => false,
        }
    }

    /// Returns `true` if the bond type is covalent or dipolar.
    pub fn is_covalent(&self) -> bool {
        match self {
            BondType::Covalent { order } | BondType::Dipolar { order } => true,
            _ => false,
        }
    }
}

/// The core data of a bond entity.
#[derive(Clone, Debug)]
pub struct BondData {
    pub(crate) bond_type: BondType,
    pub(crate) start: AnyBondable,
    pub(crate) end: AnyBondable,
}

impl BondData {
    pub(crate) fn new(bond_type: BondType, start: AnyBondable, end: AnyBondable) -> Self {
        Self {
            bond_type,
            start,
            end,
        }
    }
}

impl<'m, M: MolMap> View<'m, M, Bond> {
    pub fn bond_type(&self) -> BondType {
        self.data().bond_type
    }

    /// Returns `true` if the bond type is covalent/dipolar, ionic, metallic, or
    /// the `OtherStrong` variant.
    ///
    /// Note that this excludes hydrogen bonds and σ-hole interactions, even though
    /// these can in some cases have considerable strength.
    pub fn is_strong(&self) -> bool {
        self.bond_type().is_strong()
    }

    /// Returns `true` if the bond type is covalent or dipolar.
    pub fn is_covalent(&self) -> bool {
        self.bond_type().is_covalent()
    }

    /// Returns the order of a covalent bond, or `None` otherwise.
    ///
    /// A dipolar bond is considered covalent.
    pub fn order(&self) -> Option<f32> {
        match self.data().bond_type {
            BondType::Covalent { order } => Some(order),
            BondType::Dipolar { order } => Some(order),
            _ => None,
        }
    }

    pub fn partners(&self) -> [AnyBondable; 2] {
        let inner = self.data();
        [inner.start, inner.end]
    }
}

#[derive(Clone, Debug)]
pub enum SubstituentCentre {
    None,
    Single(AnyAtomlike),
    Multiple(Box<Vec<AnyAtomlike>>),
}

/// The core data of a substituent entity.
#[derive(Clone, Debug)]
pub struct SubstituentData {
    pub(crate) centre: SubstituentCentre,
    pub(crate) members: Vec<AnyFundamental>,
}

impl SubstituentData {
    pub(crate) fn new(centre: AnyAtomlike, members: &[AnyFundamental]) -> Self {
        Self {
            centre: SubstituentCentre::Single(centre),
            members: members.to_vec(),
        }
    }
}

impl<'m, M: MolMap> View<'m, M, Substituent> {
    /// Returns details of the centre(s) of the substituent.
    pub fn centre(&self) -> &SubstituentCentre {
        &self.data().centre
    }

    /// Returns an iterator over the IDs of all constituent atoms, pseudoatoms, and bonds.
    pub fn members(&self) -> impl Iterator<Item = AnyFundamental> {
        self.data().members.iter().copied()
    }

    /// Checks if the substituent contains the given atom, pseudoatom, or bond.
    pub fn contains(&self, fundamental: impl Fundamental) -> bool {
        self.data().members.contains(&fundamental.as_fundamental())
    }
}

//impl<'m, M: MolMap> ViewMut<'m, M, Substituent> {
///// Attempts to change the centre of the substituent to the one requested.
/////
///// # Errors
/////
///// Fails if the requested centre is not already a member of the substituent,
///// or if there are already bonds to the current centre(s).
//pub fn change_centre(mut self, new: Atomlike>) -> MolMapResult<() {
//    // First confirm that `new` is actually a member of `self`
//    self.core()
//        .members
//        .contains(&new.into())
//        .then_some(())
//        .ok_or(MolMapError::Membership(new.into()))?;
//    // A closure that determines if an atom or pseudoatom has bonds already
//    let atomlike_has_bonds = |id: Atomlike>| - bool {
//        let bonds = match id.to_tagged() {
//            ResolvedAtomlike::Atom(id) => {
//                &self
//                    .map
//                    .core()
//                    .atoms
//                    .get(id.try_into().unwrap())
//                    .expect("Wouldn't be listed as the centre if it had been removed")
//                    .bonds
//            }
//            ResolvedAtomlike::Pseudoatom(id) => {
//                &self
//                    .map
//                    .core()
//                    .pseudoatoms
//                    .get(id)
//                    .expect("Wouldn't be listed as the centre if it had been removed")
//                    .bonds
//            }
//        };
//        !bonds.is_empty()
//    };
//    // Check that there aren't already bonds to the current centre
//    let already_bonded = match self.as_view().centre().clone() {
//        SubstituentCentre::None => false,
//        SubstituentCentre::Single(atomlike_id) => atomlike_has_bonds(atomlike_id),
//        SubstituentCentre::Multiple(atomlike_ids) => {
//            atomlike_ids.into_iter().any(atomlike_has_bonds)
//        }
//    };
//    if already_bonded {
//        Err(MolMapError::Disallowed(String::from(
//            "Substituent already has at least one bond to its centre(s)",
//        )))
//    } else {
//        self.core().centre = SubstituentCentre::Single(new.into());
//        Ok(())
//    }
//}
//}

/// The core data of a molecule entity.
#[derive(Clone, Debug)]
pub struct MoleculeData {
    pub(crate) members: HashSet<AnyFundamental>,
}

impl MoleculeData {
    pub(crate) fn new() -> Self {
        Self {
            members: HashSet::new(),
        }
    }
}

impl<'m, M: MolMap> View<'m, M, Molecule> {
    /// Returns an iterator over the IDs of all constituent atoms, pseudoatoms, and bonds.
    pub fn members(&self) -> impl Iterator<Item = AnyFundamental> {
        self.data().members.iter().copied()
    }

    /// Checks if the molecule contains the given atom, pseudoatom, or bond.
    pub fn contains(&self, fundamental: impl Fundamental) -> bool {
        self.data().members.contains(&fundamental.as_fundamental())
    }
}
