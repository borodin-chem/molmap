// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use slotmap::new_key_type;

use crate::{ids::FundamentalId, traits::MolMap};

new_key_type! {
    /// An ID corresponding to a specific molecule entity in a `MolMap`.
    pub struct MoleculeId;
}

/// The core data of a molecule entity.
#[derive(Debug)]
pub(crate) struct Molecule {
    pub(crate) members: Vec<FundamentalId>,
}

impl Molecule {
    pub(crate) fn new() -> Self {
        Self {
            members: Vec::new(),
        }
    }
}

/// An immutable view over a specific molecule entity in a specific `MolMap`.
#[derive(Clone, Copy)]
pub struct MoleculeView<'a, M: MolMap> {
    pub molmap: &'a M,
    pub id: MoleculeId,
}

impl<'a, M: MolMap> From<MoleculeView<'a, M>> for MoleculeId {
    fn from(view: MoleculeView<'a, M>) -> Self {
        view.id
    }
}

impl<'a, M: MolMap> MoleculeView<'a, M> {
    /// Returns the corresponding data from the core `MolGraph`.
    fn core(&self) -> &'a Molecule {
        self.molmap.core().molecules.get(self.id).unwrap()
    }

    pub fn members(&self) -> &[FundamentalId] {
        &self.core().members
    }

    /// Checks if the molecule contains the given atom, pseudoatom, or bond.
    pub fn contains(&self, fundamental: FundamentalId) -> bool {
        self.members().contains(&fundamental)
    }
}

/// A mutable view over a specific molecule entity in a specific `MolMap`.
pub struct MoleculeViewMut<'a, M: MolMap> {
    pub molmap: &'a mut M,
    pub id: MoleculeId,
}

impl<'a, M: MolMap> From<MoleculeViewMut<'a, M>> for MoleculeId {
    fn from(view: MoleculeViewMut<'a, M>) -> Self {
        view.id
    }
}

impl<'a, M: MolMap> MoleculeViewMut<'a, M> {
    /// Returns the corresponding data from the core `MolGraph`.
    fn core(&mut self) -> &mut Molecule {
        self.molmap.core_mut().molecules.get_mut(self.id).unwrap()
    }

    /// Returns an immutable view over the same molecule.
    fn as_ref(&self) -> MoleculeView<'_, M> {
        MoleculeView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    // Public methods, which should consume the view

    /// Removes the molecule from the map, as well as all of its members.
    pub fn delete(mut self) {
        self.molmap.core_mut().delete_molecule(self.id);
    }
}
