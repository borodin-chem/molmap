// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Implementation of the core molecular graph, the public-facing pure-graph type,
//! and functions and methods for working on the graph.

pub(crate) mod data;
pub(crate) mod definition;
pub(crate) mod keys;
pub(crate) mod molgraph;
pub(crate) mod zero;

pub mod traversal;

pub(crate) use molgraph::MolGraph;

pub use zero::MolMap0;
