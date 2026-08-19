// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use slotmap::basic::Keys;

use crate::{
    Element, MolMapError, MolMapResult,
    entities::macros::define_entity_views,
    ids::{AtomId, AtomIds, BondId},
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

define_entity_views!(Atom);

impl<'a, M: MolMap> AtomView<'a, M> {
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
