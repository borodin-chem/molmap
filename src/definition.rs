// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::*;

/// A reusable definition of a group of fundamentals.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct Definition {
    pub(crate) symbol: String,
    pub(crate) members: Vec<Id<Fundamental>>,
}
