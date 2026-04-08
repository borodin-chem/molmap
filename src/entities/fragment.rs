// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{AtomId, Atomlike, BondId, FragmentId, Fundamental, IdError, MolMap, MoleculeId, PseudoatomId};

#[derive(Debug)]
pub(crate) enum FragmentCentre {
    Ambiguous(Vec<BondId>),
    Single(Atomlike),
    Multiple(Vec<Atomlike>),
}

impl Default for FragmentCentre {
    /// Creates an ambiguous centre with an empty vector of bonds.
    fn default() -> Self {
        FragmentCentre::Ambiguous(Vec::new())
    }
}

// Fragments are the smallest grouping in a MolMap
// Fragments are conceptually equivalent to a non-hydrogen atom and "its" implicit
// hydrogen atoms in SMILES or in packages that work that way,
// or to the groups drawn together without explicit bonds in a skeletal formula
// e.g. –OH, –COOH, –CH3
// Fragments have an internal structure of Atoms, Pseudoatoms, and Bonds
// Fragments generally indicate one or more centres to which bonds can be made,
// but occasionally bonds are made to a fragment as a whole.
#[derive(Debug)]
pub(crate) struct Fragment {
    pub(crate) centre: FragmentCentre,
    pub(crate) members: Vec<Fundamental>,
}

impl Fragment {
    pub(crate) fn new(members: &[Fundamental]) -> Self {
        Self {
            centre: FragmentCentre::default(),
            members: members.to_vec(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct FragmentView<'a, M: MolMap> {
    pub molmap: &'a M,
    pub id: FragmentId,
}

impl<'a, M: MolMap> From<FragmentView<'a, M>> for FragmentId {
    fn from(view: FragmentView<'a, M>) -> Self {
        view.id
    }
}

impl<'a, M: MolMap> FragmentView<'a, M> {
    fn core(&self) -> &'a Fragment {
        self.molmap.core().fragments.get(self.id).unwrap()
    }
}

pub struct FragmentViewMut<'a, M: MolMap> {
    pub molmap: &'a mut M,
    pub id: FragmentId,
}

impl<'a, M: MolMap> From<FragmentViewMut<'a, M>> for FragmentId {
    fn from(view: FragmentViewMut<'a, M>) -> Self {
        view.id
    }
}

impl<'a, M: MolMap> FragmentViewMut<'a, M> {
    fn as_ref(&self) -> FragmentView<'_, M> {
        FragmentView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    fn core(&mut self) -> &mut Fragment {
        self.molmap.core_mut().fragments.get_mut(self.id).unwrap()
    }
}
