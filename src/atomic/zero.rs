// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    BondType, Element, Map, MolMapError, MolMapResult, Pseudoelement, ViewMut,
    entities::*,
    traits::{Core, Graph, Stores},
};

use super::{AtomCore, AtomGraph, AtomMap};

/// A map holding only fundamental entities in a pure-graph representation.
#[derive(Clone, Debug, Default)]
pub struct AtomMap0 {
    graph: AtomGraph,
}

impl AtomMap0 {
    pub fn new() -> Self {
        Self {
            graph: AtomGraph::new(),
        }
    }

    pub fn contains<E: Entity>(&self, entity: E) -> bool {
        self.graph.contains(entity)
    }
}

impl Core for AtomMap0 {
    type Graph = AtomGraph;

    fn graph(&self) -> &AtomGraph {
        &self.graph
    }

    fn graph_mut(&mut self) -> &mut AtomGraph {
        &mut self.graph
    }
}

impl Map for AtomMap0 {
    fn contains<E: Entity>(&self, entity: E) -> bool {
        self.contains(entity)
    }
}

impl AtomCore for AtomMap0 {
    fn atom_graph(&self) -> &AtomGraph {
        &self.graph
    }

    fn atom_graph_mut(&mut self) -> &mut AtomGraph {
        &mut self.graph
    }
}

impl AtomMap for AtomMap0 {}

/// Methods for entity addition.
impl AtomMap0 {
    /// Adds an atom to the map.
    pub fn add_atom(&mut self, element: Element) -> Atom {
        self.graph.add_atom(element)
    }

    /// Adds an atom to the map along with the requested number of hydrogen atoms
    /// (and single covalent bonds from the central atom to them).
    ///
    /// Returns the ID of the added central atom as well as a slice over the IDs
    /// of the new bonds.
    pub fn add_atom_with_hydrogen(&mut self, element: Element, n_hydrogen: u8) -> (Atom, &[Bond]) {
        let centre = self.add_atom(element);
        for _ in 0..n_hydrogen {
            let new_h = self.add_atom(Element::H);
            self.graph
                .add_bond(BondType::Covalent { order: 1.0 }, centre, new_h);
        }
        // Don't waste memory allocating a new Vec to hold the bond IDs, since
        // they are already stored on the new central atom – return a slice instead
        (
            centre,
            self.graph
                .data(centre)
                .expect("We just created this atom, so ID must be valid")
                .bonds
                .as_slice(),
        )
    }

    /// Adds an atom to the map along with the number of additional hydrogen atoms
    /// required to satisfy the most common valency for the central atom (and single
    /// covalent bonds from the central atom to them).
    ///
    /// Returns the ID of the added central atom as well as a slice over the IDs
    /// of the new bonds.
    pub fn add_atom_and_saturate(&mut self, element: Element) -> (Atom, &[Bond]) {
        let n_hydrogen = element.default_valency();
        self.add_atom_with_hydrogen(element, n_hydrogen)
    }

    /// Adds a pseudoatom to the map.
    pub fn add_pseudoatom(&mut self, pseudoelement: Pseudoelement) -> Pseudoatom {
        self.graph.add_pseudoatom(pseudoelement)
    }

    /// Creates a new bond between two bondable entities.
    ///
    /// # Errors
    ///
    /// Fails if either of `start` and `end` are invalid.
    pub fn add_bond<A, B>(&mut self, bond_type: BondType, start: A, end: B) -> MolMapResult<Bond>
    where
        A: Bondable,
        B: Bondable,
    {
        if !self.contains(start) {
            return Err(MolMapError::InvalidId(start.as_entity()));
        } else if !self.contains(end) {
            return Err(MolMapError::InvalidId(end.as_entity()));
        };
        Ok(self.graph.add_bond(bond_type, start, end))
    }

    /// Creates a new single covalent bond between two bondable entities.
    ///
    /// # Errors
    ///
    /// Fails if either of `start` and `end` are invalid.
    pub fn add_single_bond<A, B>(&mut self, start: A, end: B) -> MolMapResult<Bond>
    where
        A: Bondable,
        B: Bondable,
    {
        self.add_bond(BondType::Covalent { order: 1.0 }, start, end)
    }

    /// Creates a new double covalent bond between two bondable entities.
    ///
    /// # Errors
    ///
    /// Fails if either of `start` and `end` are invalid.
    pub fn add_double_bond<A, B>(&mut self, start: A, end: B) -> MolMapResult<Bond>
    where
        A: Bondable,
        B: Bondable,
    {
        self.add_bond(BondType::Covalent { order: 2.0 }, start, end)
    }
}

// Implement public API for deleting entities via mutable views

impl<'m> ViewMut<'m, AtomMap0, Atom> {
    /// Removes the atom from the map, as well as any bonds to it.
    pub fn delete(self) {
        self.map.graph_mut().delete_atom(self.id);
    }
}

impl<'m> ViewMut<'m, AtomMap0, Pseudoatom> {
    /// Removes the pseudoatom from the map, as well as any bonds to it.
    pub fn delete(self) {
        self.map.graph_mut().delete_pseudoatom(self.id);
    }
}

impl<'m> ViewMut<'m, AtomMap0, Bond> {
    /// Removes the bond from the map (but not its bonding partners).
    pub fn delete(self) {
        self.map.graph_mut().delete_bond(self.id);
    }
}
