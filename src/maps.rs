// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Definitions of the various map types provided by `molmap` and their supporting
//! types and traits.

mod molmap;
mod spatial;
mod three;
mod two;
mod zero;

pub(crate) use molmap::MolMapCore;
pub(crate) use spatial::SpatialMolMap;

pub use molmap::MolMap;
pub use zero::MolMap0;

pub type MolMap2 = SpatialMolMap<2>;
pub type MolMap3 = SpatialMolMap<3>;
