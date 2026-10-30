// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The data held by an `AtomGraph` for the fundamental entities.

use crate::{Element, Pseudoelement, entities::*};

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

impl Stored for Atom {
    type Data = AtomData;
}

/// The core data of a pseudoatom entity.
#[derive(Clone, Debug)]
pub struct PseudoatomData {
    #[allow(unused)]
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

impl Stored for Pseudoatom {
    type Data = PseudoatomData;
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
        matches!(
            self,
            BondType::Covalent { order: _ }
                | BondType::Dipolar { order: _ }
                | BondType::Metallic
                | BondType::Ionic
                | BondType::OtherStrong
        )
    }

    /// Returns `true` if the bond type is covalent or dipolar.
    pub fn is_covalent(&self) -> bool {
        matches!(
            self,
            BondType::Covalent { order: _ } | BondType::Dipolar { order: _ }
        )
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

impl Stored for Bond {
    type Data = BondData;
}
