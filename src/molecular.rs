// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Maps in which the atomic entities and bonds are organized into substituents
//! and molecules.

mod common;
mod graph;
mod spatial;

pub(crate) mod data;
#[expect(unused)]
pub(crate) use spatial::MolecularPositions;

// ----------
// Public API
// ----------
// The module should act like a prelude for the molecular layer - a user can
// glob import and get basically everything they need to work with a MolMap
pub use crate::bonding::BondType;
pub use crate::elements::{Element, Pseudoelement};
pub use common::Molecular;
pub use graph::MolMap;
pub use spatial::{MolMap2, MolMap3, SpatialMolMap};

/// Entity types and traits relevant when using a `MolMap` or `SpatialMolMap`.
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

/// Views into a `MolMap` or `SpatialMolMap`.
pub mod views {
    use crate::view::{View, ViewMut};

    use super::{entities::*, *};

    /// Views into a `MolMap`.
    pub mod graph {
        use super::*;

        macro_rules! generate_views {
        ($kind:ident) => {
            paste::paste! {
                #[doc = concat!("An immutable view of an individual ", stringify!([<$kind:lower>]), " in a [`MolMap`].")]
                pub type [<$kind View>]<'m> = View<'m, MolMap, $kind>;

                #[doc = concat!("A mutable view of an individual ", stringify!([<$kind:lower>]), " in a [`MolMap`].")]
                pub type [<$kind ViewMut>]<'m> = ViewMut<'m, MolMap, $kind>;
            }
        };
    }
        generate_views!(Atom);
        generate_views!(Bond);
        generate_views!(Pseudoatom);
        generate_views!(Substituent);
        generate_views!(Molecule);
    }

    /// Views into a `SpatialMolMap`.
    pub mod spatial {
        use super::*;

        macro_rules! generate_views {
            ($kind:ident) => {
                paste::paste! {
                    #[doc = concat!("An immutable view of an individual ", stringify!([<$kind:lower>]), " in a [`SpatialMolMap`].")]
                    pub type [<$kind View>]<'m, const D: usize, U> = View<'m, SpatialMolMap<D, U>, $kind>;

                    #[doc = concat!("A mutable view of an individual ", stringify!([<$kind:lower>]), " in a [`SpatialMolMap`].")]
                    pub type [<$kind ViewMut>]<'m, const D: usize, U> = ViewMut<'m, SpatialMolMap<D, U>, $kind>;
                }
            };
        }
        generate_views!(Atom);
        generate_views!(Bond);
        generate_views!(Pseudoatom);
        generate_views!(Substituent);
        generate_views!(Molecule);
    }
}
