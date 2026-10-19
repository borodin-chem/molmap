// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The data held by a `MolGraph` for the basic collections and the view methods
//! to access it.

use std::collections::HashSet;

use crate::{Map, MolMapResult, View, ViewMut, entities::*, molecular::MolMap, traits::Stores};

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum SubstituentCentre<'a> {
    None,
    Single(AnyAtomlike),
    Multiple(&'a [AnyAtomlike]),
}

/// The core data of a substituent entity.
#[derive(Clone, Debug)]
pub struct SubstituentData {
    /// The number of central atomlikes that the substituent has.
    pub(crate) centres: usize,
    /// All atoms and pseudoatoms that the substituent contains.
    ///
    /// The first `n` items are the centres of the substituent, where `n == self.centres`.
    pub(crate) atomlikes: Vec<AnyAtomlike>,
    /// All intra-substituent bonds between the substituent's atomlikes.
    pub(crate) bonds: Vec<Bond>,
}

impl SubstituentData {
    /// Creates the data for an empty substituent.
    pub(crate) fn new() -> Self {
        Self {
            centres: 0,
            atomlikes: Vec::new(),
            bonds: Vec::new(),
        }
    }

    /// Creates the data for a substituent with a single centre.
    pub(crate) fn with_centre(centre: AnyAtomlike) -> Self {
        Self {
            centres: 1,
            atomlikes: vec![centre.into()],
            bonds: Vec::new(),
        }
    }
}

impl Stored for Substituent {
    type Data = SubstituentData;
}

impl<'m, M> View<'m, M, Substituent>
where
    M: MolMap,
{
    /// Returns a reference to the data struct in the core graph.
    fn data(&self) -> &SubstituentData {
        self.map.mol_graph().data(self.id).unwrap()
    }

    /// Returns details of the centre(s) of the substituent.
    pub fn centres(&self) -> SubstituentCentre {
        match self.data().centres {
            0 => SubstituentCentre::None,
            1 => SubstituentCentre::Single(self.data().atomlikes.first().unwrap().to_owned()),
            2.. => SubstituentCentre::Multiple(&self.data().atomlikes[..self.data().centres]),
        }
    }

    /// Returns an iterator over the IDs of all constituent atoms, pseudoatoms, and bonds.
    pub fn members(&self) -> impl Iterator<Item = AnyFundamental> {
        let mut members: Vec<AnyFundamental> = self
            .data()
            .atomlikes
            .iter()
            .map(|x| x.as_fundamental())
            .collect();
        members.extend(self.data().bonds.iter().map(|x| x.as_fundamental()));
        members.into_iter()
    }

    /// Checks if the substituent contains the given fundamental.
    pub fn contains(&self, fundamental: impl Fundamental) -> bool {
        match Fundamental::to_resolved(fundamental) {
            ResolvedFundamental::Atom(atom) => self.data().atomlikes.contains(&atom.as_atomlike()),
            ResolvedFundamental::Pseudoatom(pseudoatom) => {
                self.data().atomlikes.contains(&pseudoatom.as_atomlike())
            }
            ResolvedFundamental::Bond(bond) => self.data().bonds.contains(&bond),
        }
    }
}

impl<'m, M> ViewMut<'m, M, Substituent>
where
    M: MolMap,
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

/// The core data of a molecule entity.
#[derive(Clone, Debug)]
pub struct MoleculeData {
    /// The substituents that the molecule contains.
    pub(crate) substituents: HashSet<Substituent>,
    /// All intramolecular, inter-substituent bonds that the molecule contains.
    pub(crate) bonds: HashSet<Bond>,
}

impl MoleculeData {
    /// Creates the data for an empty molecule.
    pub(crate) fn new() -> Self {
        Self {
            substituents: HashSet::new(),
            bonds: HashSet::new(),
        }
    }
}

impl Stored for Molecule {
    type Data = MoleculeData;
}

impl<'m, M> View<'m, M, Molecule>
where
    M: MolMap,
{
    /// Returns a reference to the data struct in the core graph.
    fn data(&self) -> &MoleculeData {
        self.map.mol_graph().data(self.id).unwrap()
    }

    /// Returns an iterator over the IDs of all constituent atoms, pseudoatoms, and bonds.
    pub fn members(&self) -> impl Iterator<Item = AnyFundamental> {
        let mut members: Vec<AnyFundamental> = self
            .data()
            .bonds
            .iter()
            .map(|x| x.as_fundamental())
            .collect();
        for sub in &self.data().substituents {
            members.extend(
                self.map
                    .mol_graph()
                    .data(*sub)
                    .unwrap()
                    .atomlikes
                    .iter()
                    .map(|x| x.as_fundamental()),
            );
            members.extend(
                self.map
                    .mol_graph()
                    .data(*sub)
                    .unwrap()
                    .bonds
                    .iter()
                    .map(|x| x.as_fundamental()),
            );
        }
        members.into_iter()
    }

    /// Checks if the molecule contains the given atom, pseudoatom, or bond.
    pub fn contains(&self, fundamental: impl Fundamental) -> bool {
        match Fundamental::to_resolved(fundamental) {
            ResolvedFundamental::Atom(atom) => self.data().substituents.iter().any(|x| {
                self.map
                    .mol_graph()
                    .data(*x)
                    .unwrap()
                    .atomlikes
                    .contains(&atom.as_atomlike())
            }),
            ResolvedFundamental::Pseudoatom(pseudoatom) => {
                self.data().substituents.iter().any(|x| {
                    self.map
                        .mol_graph()
                        .data(*x)
                        .unwrap()
                        .atomlikes
                        .contains(&pseudoatom.as_atomlike())
                })
            }
            ResolvedFundamental::Bond(bond) => {
                self.data().bonds.contains(&bond)
                    || self
                        .data()
                        .substituents
                        .iter()
                        .any(|x| self.map.mol_graph().data(*x).unwrap().bonds.contains(&bond))
            }
        }
    }
}
