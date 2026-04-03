// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{AtomId, Atomlike, BondId, FragmentId, Fundamental, IdError, MolMap, MolMapExt, PseudoatomId};

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
    pub(crate) centres: Vec<Atomlike>,
    pub(crate) members: Vec<Fundamental>,
    pub(crate) bonds: Vec<BondId>,
}

impl Fragment {
    pub(crate) fn new(members: &[Fundamental]) -> Self {
        Self {
            centres: Vec::new(),
            members: members.to_vec(),
            bonds: Vec::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct FragmentView<'a, E: MolMapExt> {
    pub molmap: &'a MolMap<E>,
    pub id: FragmentId,
}

impl<'a, E: MolMapExt> From<FragmentView<'a, E>> for FragmentId {
    fn from(view: FragmentView<'a, E>) -> Self {
        view.id
    }
}

impl<'a, E: MolMapExt> FragmentView<'a, E> {
    fn inner(&self) -> &'a Fragment {
        self.molmap.fragments.get(self.id).unwrap()
    }
}

pub struct FragmentViewMut<'a, E: MolMapExt> {
    pub molmap: &'a mut MolMap<E>,
    pub id: FragmentId,
}

impl<'a, E: MolMapExt> From<FragmentViewMut<'a, E>> for FragmentId {
    fn from(view: FragmentViewMut<'a, E>) -> Self {
        view.id
    }
}

impl<'a, E: MolMapExt> FragmentViewMut<'a, E> {
    fn as_ref(&self) -> FragmentView<'_, E> {
        FragmentView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    fn inner(&mut self) -> &mut Fragment {
        self.molmap.fragments.get_mut(self.id).unwrap()
    }
}
