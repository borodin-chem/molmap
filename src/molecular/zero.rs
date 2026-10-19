// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    BondType, Element, Map, MolMapError, MolMapResult, Pseudoelement, ViewMut,
    atomic::{AtomCore, AtomGraph, AtomMap},
    entities::*,
    traits::{Core, Graph, Stores},
};

use super::{MolCore, MolGraph, MolMap};

/// A map in which the atomic entities and bonds are organized into substituents
/// and molecules, in a pure-graph representation.
#[derive(Clone, Debug, Default)]
pub struct MolMap0 {
    graph: MolGraph,
}

impl MolMap0 {
    pub fn new() -> Self {
        Self {
            graph: MolGraph::new(),
        }
    }

    pub fn contains<E: Entity>(&self, entity: E) -> bool {
        self.graph.contains(entity)
    }
}

impl Core for MolMap0 {
    type Graph = MolGraph;

    fn graph(&self) -> &MolGraph {
        &self.graph
    }

    fn graph_mut(&mut self) -> &mut MolGraph {
        &mut self.graph
    }
}

impl Map for MolMap0 {
    fn contains<E: Entity>(&self, entity: E) -> bool {
        self.contains(entity)
    }
}

impl AtomCore for MolMap0 {
    fn atom_graph(&self) -> &AtomGraph {
        self.graph.atom_graph()
    }

    fn atom_graph_mut(&mut self) -> &mut AtomGraph {
        self.graph.atom_graph_mut()
    }
}

impl AtomMap for MolMap0 {}

impl MolCore for MolMap0 {
    fn mol_graph(&self) -> &MolGraph {
        &self.graph
    }

    fn mol_graph_mut(&mut self) -> &mut MolGraph {
        &mut self.graph
    }
}

impl MolMap for MolMap0 {}
