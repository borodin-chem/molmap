// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Maps holding fundamental atomic entities connected by a network of bonds.

mod common;
mod graph;
mod spatial;

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
pub use common::Atomic;
pub use graph::AtomMap;
pub use spatial::{AtomMap2, AtomMap3, SpatialAtomMap};

/// Entity types and traits relevant when using an `AtomMap` or `SpatialAtomMap`.
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

/// Views into an `AtomMap` or `SpatialAtomMap`.
pub mod views {
    use crate::view::{View, ViewMut};

    use super::{entities::*, *};

    /// Views into an [`AtomMap`].
    pub mod graph {
        use super::*;

        macro_rules! generate_views {
        ($kind:ident) => {
            paste::paste! {
                #[doc = concat!("An immutable view of an individual ", stringify!([<$kind:lower>]), " in an [`AtomMap`].")]
                pub type [<$kind View>]<'m> = View<'m, AtomMap, $kind>;

                #[doc = concat!("A mutable view of an individual ", stringify!([<$kind:lower>]), " in an [`AtomMap`].")]
                pub type [<$kind ViewMut>]<'m> = ViewMut<'m, AtomMap, $kind>;
            }
        };
    }
        generate_views!(Atom);
        generate_views!(Bond);
        generate_views!(Pseudoatom);
    }

    /// Views into a `SpatialAtomMap`.
    pub mod spatial {
        use super::*;

        macro_rules! generate_views {
            ($kind:ident) => {
                paste::paste! {
                    #[doc = concat!("An immutable view of an individual ", stringify!([<$kind:lower>]), " in a [`SpatialAtomMap`].")]
                    pub type [<$kind View>]<'m, const D: usize, U> = View<'m, SpatialAtomMap<D, U>, $kind>;

                    #[doc = concat!("A mutable view of an individual ", stringify!([<$kind:lower>]), " in a [`SpatialAtomMap`].")]
                    pub type [<$kind ViewMut>]<'m, const D: usize, U> = ViewMut<'m, SpatialAtomMap<D, U>, $kind>;
                }
            };
        }
        generate_views!(Atom);
        generate_views!(Bond);
        generate_views!(Pseudoatom);
    }
}
