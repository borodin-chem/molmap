// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{AtomId, Atomlike, BondId, IdError, MolMap, SubstituentId, PseudoatomId};

// Entities that are connected by *visible* bonds are called Substituents here
// These are the subunits that bonds are drawn between in a skeletal formula
// Bonds that are *displayed* are always between (the central atoms of) nodes
// They *always* have a central Atom or Pseudoatom
// Internally they also have their own network of Atoms, Pseudoatoms, and Bonds
#[derive(Debug)]
pub struct Substituent {
    pub id: SubstituentId,
    pub centre: Atomlike,
    pub atoms: Vec<AtomId>,
    pub pseudoatoms: Vec<PseudoatomId>,
    pub bonds: Vec<BondId>,
}

impl Substituent {
    pub fn new(id: SubstituentId, centre: Atomlike) -> Self {
        Self {
            id,
            centre,
            atoms: Vec::new(),
            pseudoatoms: Vec::new(),
            bonds: Vec::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct NodeView<'a> {
    pub molmap: &'a MolMap,
    pub id: SubstituentId,
}

impl<'a> From<NodeView<'a>> for SubstituentId {
    fn from(view: NodeView<'a>) -> Self {
        view.id
    }
}

impl<'a> NodeView<'a> {
    fn inner(&self) -> &'a Substituent {
        self.molmap.nodes.get(self.id).unwrap()
    }
    
    pub fn centre(&self) -> Atomlike {
        self.inner().centre
    }
}

pub struct NodeViewMut<'a> {
    pub molmap: &'a mut MolMap,
    pub id: SubstituentId,
}

impl<'a> From<NodeViewMut<'a>> for SubstituentId {
    fn from(view: NodeViewMut<'a>) -> Self {
        view.id
    }
}

impl<'a> NodeViewMut<'a> {
    fn as_ref(&self) -> NodeView<'_> {
        NodeView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    fn inner(&mut self) -> &mut Substituent {
        self.molmap.nodes.get_mut(self.id).unwrap()
    }
}
