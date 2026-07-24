// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    Element, MolMapError, MolMapResult,
    ids::{AtomId, BondId},
    traits::MolMap,
};

/// The core data of an atom entity.
#[derive(Clone, Debug)]
pub(crate) struct Atom {
    pub(crate) element: Element,
    pub(crate) bonds: Vec<BondId>,
}

impl Atom {
    pub(crate) fn new(element: Element) -> Self {
        Self {
            element,
            bonds: Vec::new(),
        }
    }
}

/// An immutable view over a specific atom entity in a specific `MolMap`.
#[derive(Copy, Clone, Debug)]
pub struct AtomView<'a, M: MolMap> {
    pub molmap: &'a M,
    pub id: AtomId,
}

impl<'a, M: MolMap> From<AtomView<'a, M>> for AtomId {
    fn from(view: AtomView<'a, M>) -> Self {
        view.id
    }
}

impl<'a, M: MolMap> AtomView<'a, M> {
    /// Returns the corresponding data from the core `MolGraph`.
    fn core(&self) -> &'a Atom {
        self.molmap.core().atoms.get(self.id).unwrap()
    }

    pub fn element(&self) -> Element {
        self.core().element
    }

    pub fn symbol(&self) -> &str {
        self.core().element.symbol()
    }

    pub fn bonds(&self) -> &[BondId] {
        &self.core().bonds
    }
}

/// A mutable view over a specific atom entity in a specific `MolMap`.
#[derive(Debug)]
pub struct AtomViewMut<'a, M: MolMap> {
    pub molmap: &'a mut M,
    pub id: AtomId,
}

impl<'a, M: MolMap> From<AtomViewMut<'a, M>> for AtomId {
    fn from(view: AtomViewMut<'a, M>) -> Self {
        view.id
    }
}

impl<'a, M: MolMap> AtomViewMut<'a, M> {
    /// Returns the corresponding data from the core `MolGraph`.
    fn core(&mut self) -> &mut Atom {
        self.molmap.core_mut().atoms.get_mut(self.id).unwrap()
    }

    /// Returns an immutable view over the same atom.
    fn as_ref(&self) -> AtomView<'_, M> {
        AtomView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    // Public methods, which should consume the view

    /// Set the element of the atom without any additional effects.
    pub fn set_element(mut self, element: Element) {
        self.core().element = element
    }

    /// Removes the atom from the map, as well as any bonds to it.
    pub fn delete(mut self) {
        self.molmap.core_mut().delete_atom(self.id);
    }
}
