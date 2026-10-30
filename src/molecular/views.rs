// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! View methods that apply to all molecular maps.

use crate::{View, ViewMut, entities::*, molecular::Molecular, traits::Stores};

use super::data::*;

impl<'m, M> View<'m, M, Substituent>
where
    M: Molecular,
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
    M: Molecular,
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
    M: Molecular,
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
