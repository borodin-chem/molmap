// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! View methods that apply to all molecular maps, and aliases for views into a [`MolMap`] or [`SpatialMolMap`].

use crate::{
    entities::*,
    traits::{Map, Stores},
    view::{View, ViewMut},
};

use super::{Molecular, data::*};

/// Aliases for views into a [`MolMap`].
pub mod graph {
    use super::super::MolMap;
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

/// Aliases for views into a [`SpatialMolMap`].
pub mod spatial {
    use super::super::SpatialMolMap;
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

impl<'m, M> View<'m, M, Substituent>
where
    M: Map + Molecular,
{
    /// Returns a reference to the data struct in the core graph.
    #[allow(unused)]
    fn data(&self) -> &SubstituentData {
        self.map.as_molecular().data(self.id).unwrap()
    }

    /// Checks if the substituent contains the given fundamental.
    pub fn contains(&self, fundamental: impl Fundamental) -> bool {
        self.map
            .as_molecular()
            .substituent_contains(self.id, fundamental)
            .expect("Existence of view guarantees ID validity")
    }

    /// Returns an iterator over the IDs of all constituent atoms, pseudoatoms, and bonds.
    pub fn members(&self) -> impl Iterator<Item = AnyFundamental> {
        self.map
            .as_molecular()
            .substituent_members(self.id)
            .expect("Existence of view guarantees ID validity")
    }

    /// Returns details of the centre(s) of the substituent.
    pub fn centres(&self) -> SubstituentCentre<'_> {
        self.map
            .as_molecular()
            .substituent_centres(self.id)
            .expect("Existence of view guarantees ID validity")
    }
}

impl<'m, M> ViewMut<'m, M, Substituent>
where
    M: Map + Molecular,
{
    ///// Attempts to change the centre of the substituent to the one requested.
    /////
    ///// # Errors
    /////
    ///// Fails if the requested centre is not already a member of the substituent,
    ///// or if there are already bonds to the current centre(s).
    //pub fn set_centre(self, new_centre: impl Atomlike) -> MolMapResult<()> {
    //    self.map
    //        .graph_mut()
    //        .set_substituent_centre(self.id, new_centre)
    //}

    ///// Makes the requested atomlike a centre of the substituent, in addition to any
    ///// already existing centres.
    /////
    ///// # Errors
    /////
    ///// Fails if the requested centre is not already a member of the substituent.
    //pub fn add_centre(self, new_centre: impl Atomlike) -> MolMapResult<()> {
    //    self.map
    //        .graph_mut()
    //        .add_substituent_centre(self.id, new_centre)
    //}
}

impl<'m, M> View<'m, M, Molecule>
where
    M: Map + Molecular,
{
    /// Returns a reference to the data struct in the core graph.
    #[allow(unused)]
    fn data(&self) -> &MoleculeData {
        self.map.as_molecular().data(self.id).unwrap()
    }

    /// Checks if the molecule contains the given atom, pseudoatom, or bond.
    pub fn contains(&self, fundamental: impl Fundamental) -> bool {
        self.map
            .as_molecular()
            .molecule_contains(self.id, fundamental)
            .expect("Existence of view guarantees ID validity")
    }

    /// Returns an iterator over the IDs of all constituent atoms, pseudoatoms, and bonds.
    pub fn members(&self) -> impl Iterator<Item = AnyFundamental> {
        self.map
            .as_molecular()
            .molecule_members(self.id)
            .expect("Existence of view guarantees ID validity")
    }
}
