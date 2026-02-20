// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{Annotatable, MolMap, ObjectId};

pub enum Object {
    Charge(Charge),
}

pub enum ChargeStyle {
    Bare,
    Circled,
    PartialDelta,
    PartialNumeric,
}

pub struct Charge {
    pub id: ObjectId,
    pub charge: f32,
    pub charge_style: ChargeStyle,
    pub anchor: Annotatable,
}

pub struct ObjectView<'a> {
    pub molmap: &'a MolMap,
    pub id: ObjectId,
}

pub struct ObjectViewMut<'a> {
    pub molmap: &'a mut MolMap,
    pub id: ObjectId,
}
