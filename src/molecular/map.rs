// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{Map, atomic::AtomMap};

use super::MolGraph;

#[allow(unreachable_pub)]
mod internal {
    use super::*;

    pub trait MolCore {
        /// Provides access to the core graph.
        fn mol_graph(&self) -> &MolGraph;

        /// Provides mutable access to the core graph.
        fn mol_graph_mut(&mut self) -> &mut MolGraph;
    }
}
pub(crate) use internal::MolCore;

/// A map in which the atomic entities and bonds are organized into substituents
/// and molecules.
pub trait MolMap: Map + AtomMap + MolCore {}
