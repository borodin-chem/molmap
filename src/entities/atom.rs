// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{AtomId, BondId, Bondable, Element, MolMap, SubstituentId, ObjectId};

#[derive(Debug)]
pub struct Atom {
    pub id: AtomId,
    pub element: Element,
    pub isotope: Option<u16>,
    pub annotations: Vec<ObjectId>,
}

impl Atom {
    pub fn new(id: AtomId, element: Element) -> Self {
        Self {
            id,
            element,
            isotope: None,
            annotations: Vec::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct AtomView<'a> {
    pub molmap: &'a MolMap,
    pub id: AtomId,
}

impl<'a> From<AtomView<'a>> for AtomId {
    fn from(view: AtomView<'a>) -> Self {
        view.id
    }
}

impl<'a> AtomView<'a> {
    fn inner(&self) -> &'a Atom {
        self.molmap.atoms.get(self.id).unwrap()
    }
    
    pub fn symbol(&self) -> &str {
        self.inner().element.symbol()
    }
}

pub struct AtomViewMut<'a> {
    pub molmap: &'a mut MolMap,
    pub id: AtomId,
}

impl<'a> From<AtomViewMut<'a>> for AtomId {
    fn from(view: AtomViewMut<'a>) -> Self {
        view.id
    }
}

impl<'a> AtomViewMut<'a> {
    fn as_ref(&self) -> AtomView<'_> {
        AtomView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    fn inner(&mut self) -> &mut Atom {
        self.molmap.atoms.get_mut(self.id).unwrap()
    }
}
