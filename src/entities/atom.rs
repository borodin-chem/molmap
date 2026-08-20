// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use slotmap::basic::Keys;

use crate::*;

/// Smallest particle still characterizing a chemical element.
#[derive(Copy, Clone, Debug)]
pub struct Atom;

impl Entity for Atom {}

impl KeyEntity for Atom {
    fn kind() -> EntityKind {
        EntityKind::Atom
    }
}

/// The core data of an atom entity.
#[derive(Clone, Debug)]
pub(crate) struct AtomData {
    pub(crate) element: Element,
    pub(crate) bonds: Vec<Id<Bond>>,
}

impl AtomData {
    pub(crate) fn new(element: Element) -> Self {
        Self {
            element,
            bonds: Vec::new(),
        }
    }
}

pub type AtomView<'a, M> = View<'a, M, Atom>;

impl<'a, M: MolMap> View<'a, M, Atom> {
    fn core(&self) -> &AtomData {
        self.map.core().atoms.get(self.id).unwrap()
    }

    pub fn element(&self) -> Element {
        self.core().element
    }

    pub fn symbol(&self) -> &str {
        self.core().element.symbol()
    }

    pub fn bonds(&self) -> &[Id<Bond>] {
        &self.core().bonds
    }
}
