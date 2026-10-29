// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::Map;

use super::AtomGraph;

#[allow(unreachable_pub)]
mod internal {
    use super::*;

    pub trait AtomCore {
        /// Provides access to the core graph.
        fn atom_graph(&self) -> &AtomGraph;

        /// Provides mutable access to the core graph.
        fn atom_graph_mut(&mut self) -> &mut AtomGraph;
    }
}
pub(crate) use internal::AtomCore;

/// A map holding fundamental atomic entities, and a network of bonds between them.
pub trait AtomMap: Map + AtomCore {}
