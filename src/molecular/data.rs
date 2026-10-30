// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The data held by a `MolGraph` for the basic collections.

use std::collections::HashSet;

use crate::entities::*;

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

#[expect(dead_code)]
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
            atomlikes: vec![centre],
            bonds: Vec::new(),
        }
    }
}

impl Stored for Substituent {
    type Data = SubstituentData;
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
