// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashSet;

use slotmap::new_key_type;

use crate::{
    ids::{FundamentalId, MoleculeId},
    traits::MolMap,
};

/// The core data of a molecule entity.
#[derive(Clone, Debug)]
pub(crate) struct Molecule {
    pub(crate) members: HashSet<FundamentalId>,
}

impl Molecule {
    pub fn new() -> Self {
        Self {
            members: HashSet::new(),
        }
    }
}

/// An immutable view over a specific molecule entity in a specific `MolMap`.
#[derive(Copy, Clone, Debug)]
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

    /// Returns an iterator over the IDs of all constituent atoms, pseudoatoms, and bonds.
    pub fn members(&self) -> impl Iterator<Item = FundamentalId> {
        self.core().members.iter().copied()
    }

    /// Checks if the molecule contains the given atom, pseudoatom, or bond.
    pub fn contains(&self, fundamental: FundamentalId) -> bool {
        self.core().members.contains(&fundamental)
    }
}

/// A mutable view over a specific molecule entity in a specific `MolMap`.
#[derive(Debug)]
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
