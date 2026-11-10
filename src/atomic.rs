// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Maps holding fundamental atomic entities connected by a network of bonds.

mod graph;
mod spatial;

/// Traits specific to the atomic layer.
mod traits {
    use crate::Map;

    use super::AtomMap;

    /// A map that holds fundamental atomic entities, and a network of bonds between them.
    ///
    /// This applies to all maps, and so all map types in the crate implement this trait.
    pub trait Atomic: Map {
        /// Returns the the core network of atoms and bonds without any spatial positions.
        fn into_atomic(self) -> AtomMap;

        /// Provides read-only access to the core network of atoms and bonds.
        fn as_atomic(&self) -> &AtomMap;
    }
}

// ------------------
// Crate-internal API
// ------------------
pub(crate) mod data;
pub(crate) use spatial::SpatialAtomicCommon;

// ----------
// Public API
// ----------
// The module should act a bit like a prelude for the atomic layer - a user can
// glob import and get basically everything they need to work with an AtomMap
pub use crate::bonding::BondType;
pub use crate::elements::{Element, Pseudoelement};
pub use graph::AtomMap;
pub use spatial::{AtomMap2, AtomMap3, SpatialAtomMap};
pub use traits::Atomic;

pub mod views;

/// Entity types and traits relevant when using an [`AtomMap`].
pub mod entities {
    // In-line entity kinds so that the mod doc page gives a neat overview of them
    #[doc(inline)]
    pub use crate::entities::{Atom, Bond, Pseudoatom};

    // Likewise for traits
    #[doc(inline)]
    pub use crate::entities::{Atomlike, Bondable, Category, Entity, Fundamental, Kind};

    // Everything else can be listed as a re-export
    pub use crate::entities::{AnyAtomlike, AnyBondable, AnyEntity, AnyFundamental};
    pub use crate::entities::{
        ResolvedAtomlike, ResolvedBondable, ResolvedEntity, ResolvedFundamental,
    };
}
