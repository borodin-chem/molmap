// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{AtomId, BondId, Bondable, Element, FragmentId, MolMap, element::MassNumber};

#[derive(Debug)]
pub(crate) struct Atom {
    pub(crate) id: AtomId,
    pub(crate) element: Element,
    pub(crate) isotope: Option<MassNumber>,
    pub(crate) bonds: Vec<BondId>,
    //pub annotations: Vec<ObjectId>,
}

impl Atom {
    pub(crate) fn new(id: AtomId, element: Element) -> Self {
        Self {
            id,
            element,
            isotope: None,
            bonds: Vec::new(),
            //annotations: Vec::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct AtomView<'a, E> {
    pub molmap: &'a MolMap<E>,
    pub id: AtomId,
}

impl<'a, E> From<AtomView<'a, E>> for AtomId {
    fn from(view: AtomView<'a, E>) -> Self {
        view.id
    }
}

impl<'a, E> AtomView<'a, E> {
    fn inner(&self) -> &'a Atom {
        self.molmap.atoms.get(self.id).unwrap()
    }

    pub fn symbol(&self) -> &str {
        self.inner().element.symbol()
    }
}

pub struct AtomViewMut<'a, E> {
    pub molmap: &'a mut MolMap<E>,
    pub id: AtomId,
}

impl<'a, E> From<AtomViewMut<'a, E>> for AtomId {
    fn from(view: AtomViewMut<'a, E>) -> Self {
        view.id
    }
}

impl<'a, E> AtomViewMut<'a, E> {
    fn as_ref(&self) -> AtomView<'_, E> {
        AtomView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    fn inner(&mut self) -> &mut Atom {
        self.molmap.atoms.get_mut(self.id).unwrap()
    }
}
