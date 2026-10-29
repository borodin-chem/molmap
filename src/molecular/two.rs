// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::units::Relative;

use super::spatial::SpatialMolMap;

/// A map holding atomic entities and bonds organized into substituents
/// and molecules, with positions in two-dimensional space.
pub type MolMap2 = SpatialMolMap<2, Relative>;
