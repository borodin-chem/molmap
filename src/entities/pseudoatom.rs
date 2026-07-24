// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use slotmap::new_key_type;

use crate::{
    Pseudoelement,
    ids::{BondId, KeyId, PseudoatomId},
    traits::MolMap,
};

/// The core data of a pseudoatom entity.
///
/// A pseudoatom is something that has a "symbol" like a normal atom but
/// represents something else.
/// It may have an unknown composition like R, or a known structure like Ph.
#[derive(Clone, Debug)]
pub(crate) struct Pseudoatom {
    pub(crate) pseudoelement: Pseudoelement,
    pub(crate) bonds: Vec<BondId>,
}

impl Pseudoatom {
    pub fn new(pseudoelement: Pseudoelement) -> Self {
        Self {
            pseudoelement,
            bonds: Vec::new(),
        }
    }
}

/// An immutable view over a specific pseudoatom entity in a specific `MolMap`.
#[derive(Copy, Clone, Debug)]
pub struct PseudoatomView<'a, M: MolMap> {
    pub molmap: &'a M,
    pub id: PseudoatomId,
}

impl<'a, M: MolMap> From<PseudoatomView<'a, M>> for PseudoatomId {
    fn from(view: PseudoatomView<'a, M>) -> Self {
        view.id
    }
}

impl<'a, M: MolMap> PseudoatomView<'a, M> {
    /// Returns the corresponding data from the core `MolGraph`.
    fn core(&self) -> &'a Pseudoatom {
        self.molmap.core().pseudoatoms.get(self.id).unwrap()
    }

    pub fn bonds(&self) -> &[BondId] {
        &self.core().bonds
    }
}

/// A mutable view over a specific pseudoatom entity in a specific `MolMap`.
#[derive(Debug)]
pub struct PseudoatomViewMut<'a, M: MolMap> {
    pub molmap: &'a mut M,
    pub id: PseudoatomId,
}

impl<'a, M: MolMap> From<PseudoatomViewMut<'a, M>> for PseudoatomId {
    fn from(view: PseudoatomViewMut<'a, M>) -> Self {
        view.id
    }
}

impl<'a, M: MolMap> PseudoatomViewMut<'a, M> {
    /// Returns the corresponding data from the core `MolGraph`.
    fn core(&mut self) -> &mut Pseudoatom {
        self.molmap.core_mut().pseudoatoms.get_mut(self.id).unwrap()
    }

    /// Returns an immutable view over the same pseudoatom.
    fn as_ref(&self) -> PseudoatomView<'_, M> {
        PseudoatomView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    // Public methods, which should consume the view

    /// Set the pseudoelement of the pseudoatom without any additional effects.
    pub fn set_pseudoelement(mut self, pseudoelement: Pseudoelement) {
        self.core().pseudoelement = pseudoelement
    }

    /// Removes the pseudoatom from the map, as well as any bonds to it.
    pub fn delete(mut self) {
        self.molmap.core_mut().delete_pseudoatom(self.id);
    }
}
