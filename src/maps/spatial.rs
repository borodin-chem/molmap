// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use nalgebra as na;
use nalgebra::Point;
use slotmap::SecondaryMap;

use crate::{
    MolMap,
    categories::*,
    entities::*,
    graph::{MolGraph, keys::*},
    maps::MolMapCore,
    view::View,
};

// The nalgebra vector type leaves the storage type generic
// This is a more convenient alias
/// A matrix with one column and D rows.
///
/// Note that `Vector<T, 2>` using this alias is just shorthand for
/// `nalgebra::Vector2<T>` (and similarly `Vector<T, 3>` is `nalgebra::Vector3<T>`).
pub type Vector<T, const D: usize> = na::Vector<T, na::Const<D>, na::ArrayStorage<T, D, 1>>;

/// A [`MolMap`] that also holds the spatial positions (with dimensionality `D`)
/// of its entities, but no further application-specific information.
#[derive(Clone, Debug)]
pub struct SpatialMolMap<const D: usize> {
    core: MolGraph,
    atom_positions: SecondaryMap<AtomKey, Point<f64, D>>,
    pseudoatom_positions: SecondaryMap<PseudoatomKey, Point<f64, D>>,
    ///// Bond positions are just the positions of their start and end bondable,
    ///// but the vectors of the bonds are cached due to their usefulness.
    //bonds: SecondaryMap<BondKey, Vector<f64, D>>,
}

/// A [`MolMap`] that also holds the positions of its entities in two dimensions,
/// but no further application-specific information.
pub type MolMap2 = SpatialMolMap<2>;

/// A [`MolMap`] that also holds the positions of its entities in three dimensions,
/// but no further application-specific information.
pub type MolMap3 = SpatialMolMap<3>;

impl<const D: usize> MolMapCore for SpatialMolMap<D> {
    #[inline]
    fn core(&self) -> &MolGraph {
        &self.core
    }

    #[inline]
    fn core_mut(&mut self) -> &mut MolGraph {
        &mut self.core
    }
}

impl<const D: usize> MolMap for SpatialMolMap<D> {
    fn new() -> Self {
        Self {
            core: MolGraph::new(),
            atom_positions: SecondaryMap::new(),
            pseudoatom_positions: SecondaryMap::new(),
        }
    }

    fn with_capacities(
        atoms: usize,
        pseudoatoms: usize,
        bonds: usize,
        substituents: usize,
        molecules: usize,
    ) -> Self {
        Self {
            core: MolGraph::with_capacities(atoms, pseudoatoms, bonds, substituents, molecules),
            atom_positions: SecondaryMap::with_capacity(atoms),
            pseudoatom_positions: SecondaryMap::with_capacity(pseudoatoms),
        }
    }
}

/// Methods for accessing and calculating positions.
impl<const D: usize> SpatialMolMap<D> {
    /// Returns the position of the given atom.
    ///
    /// # Panics
    ///
    /// Panics if the atom is not in the map.
    pub(crate) fn atom_position(&self, atom: Atom) -> &Point<f64, D> {
        self.atom_positions
            .get(atom.to_key())
            .expect("Caller is required to ensure that the ID is valid")
    }

    /// Returns the position of the given pseudoatom.
    ///
    /// # Panics
    ///
    /// Panics if the pseudoatom is not in the map.
    pub(crate) fn pseudoatom_position(&self, pseudoatom: Pseudoatom) -> &Point<f64, D> {
        self.pseudoatom_positions
            .get(pseudoatom.to_key())
            .expect("Caller is required to ensure that the ID is valid")
    }

    /// Returns the position of the given atom or pseudoatom.
    ///
    /// # Panics
    ///
    /// Panics if the atomlike is not in the map.
    pub(crate) fn atomlike_position<E: Atomlike>(&self, atomlike: E) -> &Point<f64, D> {
        match atomlike.as_tagged_atomlike() {
            TaggedAtomlike::Atom(atom) => self.atom_position(atom),
            TaggedAtomlike::Pseudoatom(pseudoatom) => self.pseudoatom_position(pseudoatom),
        }
    }

    /// Calculates the vector of the line between two atomlikes, from `a` to `b`.
    ///
    /// # Panics
    ///
    /// Panics if either atomlike is not in the map.
    pub(crate) fn interatomlike_line<A: Atomlike, B: Atomlike>(
        &self,
        a: A,
        b: B,
    ) -> Vector<f64, D> {
        self.atomlike_position(b) - self.atomlike_position(a)
    }

    /// Returns the position of the [`Bondable`] from which the bond starts.
    ///
    /// # Panics
    ///
    /// Panics if the bond is not in the map.
    pub(crate) fn bond_origin(&self, bond: Bond) -> &Point<f64, D> {
        match self.core().data(bond).start.as_tagged_bondable() {
            TaggedBondable::Atom(atom) => self.atom_position(atom),
            TaggedBondable::Pseudoatom(pseudoatom) => self.pseudoatom_position(pseudoatom),
        }
    }

    /// Returns the position of the [`Bondable`] at which the bond ends.
    ///
    /// # Panics
    ///
    /// Panics if the bond is not in the map.
    pub(crate) fn bond_terminus(&self, bond: Bond) -> &Point<f64, D> {
        match self.core().data(bond).end.as_tagged_bondable() {
            TaggedBondable::Atom(atom) => self.atom_position(atom),
            TaggedBondable::Pseudoatom(pseudoatom) => self.pseudoatom_position(pseudoatom),
        }
    }

    /// Calculates the midpoint of the bond.
    ///
    /// # Panics
    ///
    /// Panics if the bond is not in the map.
    pub(crate) fn bond_midpoint(&self, bond: Bond) -> Point<f64, D> {
        na::center(self.bond_origin(bond), self.bond_terminus(bond))
    }

    /// Calculates the vector that the bond follows from its origin to its terminus.
    ///
    /// # Panics
    ///
    /// Panics if the bond is not in the map.
    pub(crate) fn bond_vector(&self, bond: Bond) -> Vector<f64, D> {
        //self.positions()
        //    .bonds
        //    .get(bond.to_key())
        //    .expect("Caller is required to ensure that the ID is valid")
        self.bond_terminus(bond) - self.bond_origin(bond)
    }

    ///// Calculates the unweighted geometric centre of the collection.
    /////
    ///// Only the positions of the constituent atoms and pseudoatoms are taken into
    ///// consideration, not the bonds, nor any other member fundamentals.
    //pub(crate) fn collection_centroid<E: Collection>(&self, collection: E) -> Point<f64, D> {
    //    let members: Vec<AnyFundamental> = match collection.as_tagged_collection() {
    //        TaggedCollection::Substituent(substituent) => self
    //            .core
    //            .data(substituent)
    //            .members
    //            .iter()
    //            .filter(|f| f.kind())
    //                //.map(|f| self.atomlike_position(atomlike)),
    //        TaggedCollection::Molecule(molecule) => {
    //            self.core.data(molecule).members.iter().copied().collect()
    //        }
    //    };
    //}
}

impl<'m, const D: usize> View<'m, SpatialMolMap<D>, Atom> {
    /// Returns the position of the atom.
    pub fn position(&self) -> &Point<f64, D> {
        self.map.atom_position(self.id)
    }
}

impl<'m, const D: usize> View<'m, SpatialMolMap<D>, Pseudoatom> {
    /// Returns the position of the pseudoatom.
    pub fn position(&self) -> &Point<f64, D> {
        self.map.pseudoatom_position(self.id)
    }
}

impl<'m, const D: usize> View<'m, SpatialMolMap<D>, Bond> {
    /// Returns the position of the [`Bondable`] from which the bond starts.
    pub fn origin(&self) -> &Point<f64, D> {
        self.map.bond_origin(self.id)
    }

    /// Returns the position of the [`Bondable`] at which the bond ends.
    pub fn terminus(&self) -> &Point<f64, D> {
        self.map.bond_terminus(self.id)
    }

    /// Calculates the midpoint of the bond.
    pub fn midpoint(&self) -> Point<f64, D> {
        self.map.bond_midpoint(self.id)
    }

    /// Calculates the vector that the bond follows from its origin to its terminus.
    pub fn vector(&self) -> Vector<f64, D> {
        self.map.bond_vector(self.id)
    }

    /// Calculates the distance from the bond's origin to its terminus.
    pub fn length(&self) -> f64 {
        self.vector().magnitude()
    }
}
