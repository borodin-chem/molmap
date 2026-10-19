// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Functionality generic over any spatial `MolMap`.

use std::marker::PhantomData;

use nalgebra as na;
use slotmap::SecondaryMap;

use crate::{
    BondType, Element, Map, MolMapError, MolMapResult, Point, Pseudoelement, Spatial, Vector, View,
    ViewMut,
    atomic::{AtomCore, AtomGraph, AtomMap},
    entities::*,
    geometry,
    traits::{Core, Graph, Stores, StoresPosition},
    units::Unit,
};

use super::{MolCore, MolGraph, MolMap};

/// A map in which the atomic entities and bonds are organized into substituents
/// and molecules, with positions in `D`-dimensional space.
#[derive(Clone, Debug, Default)]
pub struct SpatialMolMap<const D: usize, U> {
    graph: MolGraph,
    atom_positions: SecondaryMap<AtomKey, Point<D>>,
    pseudoatom_positions: SecondaryMap<PseudoatomKey, Point<D>>,
    unit: PhantomData<U>,
}

impl<const D: usize, U> SpatialMolMap<D, U> {
    pub fn new() -> Self {
        Self {
            graph: MolGraph::new(),
            atom_positions: SecondaryMap::new(),
            pseudoatom_positions: SecondaryMap::new(),
            unit: PhantomData,
        }
    }

    pub fn contains<E: Entity>(&self, entity: E) -> bool {
        self.graph.contains(entity)
    }
}

impl<const D: usize, U> Core for SpatialMolMap<D, U> {
    type Graph = MolGraph;

    fn graph(&self) -> &Self::Graph {
        &self.graph
    }

    fn graph_mut(&mut self) -> &mut Self::Graph {
        &mut self.graph
    }
}

impl<const D: usize, U> Map for SpatialMolMap<D, U> {
    fn contains<E: Entity>(&self, entity: E) -> bool {
        self.contains(entity)
    }
}

impl<const D: usize, U> AtomCore for SpatialMolMap<D, U> {
    fn atom_graph(&self) -> &AtomGraph {
        self.graph.atom_graph()
    }

    fn atom_graph_mut(&mut self) -> &mut AtomGraph {
        self.graph.atom_graph_mut()
    }
}

impl<const D: usize, U> AtomMap for SpatialMolMap<D, U> {}

impl<const D: usize, U> MolCore for SpatialMolMap<D, U> {
    fn mol_graph(&self) -> &MolGraph {
        &self.graph
    }

    fn mol_graph_mut(&mut self) -> &mut MolGraph {
        &mut self.graph
    }
}

impl<const D: usize, U> MolMap for SpatialMolMap<D, U> {}

impl<const D: usize, U: Unit> Spatial for SpatialMolMap<D, U> {
    type Unit = U;

    type DimName = na::Const<D>;

    type Point = Point<D>;

    type Vector = Vector<D>;
}
