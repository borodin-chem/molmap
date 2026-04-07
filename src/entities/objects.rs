// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{Anchor, MolMap, ObjectId};

pub enum Object {
    Charge(Charge),
}

pub struct ObjectView<'a, M> {
    pub molmap: &'a MolMap<E>,
    pub id: ObjectId,
}

pub struct ObjectViewMut<'a, M> {
    pub molmap: &'a mut MolMap<E>,
    pub id: ObjectId,
}

pub struct Charge {
    pub id: ObjectId,
    pub charge: f32,
    pub anchor: Anchor,
}
