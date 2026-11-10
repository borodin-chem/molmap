// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Maps in which the atomic entities and bonds are organized into substituents
//! and molecules.

mod graph;
mod spatial;

/// Traits specific to the molecular layer.
mod traits {
    use super::MolMap;
    use crate::Map;

    /// A map in which the atomic entities and bonds are organized into substituents
    /// and molecules.
    pub trait Molecular: Map {
        /// Returns the molecular graph without any spatial positions.
        fn into_molecular(self) -> MolMap;

        /// Provides read-only access to the molecular graph.
        fn as_molecular(&self) -> &MolMap;
    }
}

pub(crate) mod data;
#[expect(unused)]
pub(crate) use spatial::SpatialMolecularCommon;

// ----------
// Public API
// ----------
// The module should act like a prelude for the molecular layer - a user can
// glob import and get basically everything they need to work with a MolMap
pub use crate::bonding::BondType;
pub use crate::elements::{Element, Pseudoelement};
pub use graph::MolMap;
pub use spatial::{MolMap2, MolMap3, SpatialMolMap};
pub use traits::Molecular;

pub mod views;

/// Entity types and traits relevant when using a [`MolMap`].
pub mod entities {
    // In-line entity kinds so that the mod doc page gives a neat overview of them
    #[doc(inline)]
    pub use crate::entities::{Atom, Bond, Molecule, Pseudoatom, Substituent};

    // Likewise for traits
    #[doc(inline)]
    pub use crate::entities::{
        Atomlike, Bondable, Category, Collection, Entity, Fundamental, Kind,
    };

    // Everything else can be listed as a re-export
    pub use crate::entities::{AnyAtomlike, AnyBondable, AnyCollection, AnyEntity, AnyFundamental};
    pub use crate::entities::{
        ResolvedAtomlike, ResolvedBondable, ResolvedCollection, ResolvedEntity, ResolvedFundamental,
    };
}
