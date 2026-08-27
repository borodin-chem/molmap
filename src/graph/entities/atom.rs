// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.;

use crate::{Element, MolMap, categories::*, entities::*, view::*};

/// The core data of an atom entity.
#[derive(Clone, Debug)]
pub struct AtomData {
    pub(crate) element: Element,
    pub(crate) bonds: Vec<Bond>,
}

impl AtomData {
    pub(crate) fn new(element: Element) -> Self {
        Self {
            element,
            bonds: Vec::new(),
        }
    }
}

impl<'m, M: MolMap> View<'m, M, Atom> {
    pub fn element(&self) -> Element {
        self.data().element
    }

    pub fn symbol(&self) -> &str {
        self.data().element.symbol()
    }

    pub fn bonds(&self) -> &[Bond] {
        &self.data().bonds
    }
}
