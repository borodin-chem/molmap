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
    Map, Point, Spatial, Vector,
    atomic::AtomMap,
    entities::*,
    units::{Angstrom, Relative, Unit},
};

use super::{MolMap, Molecular};

/// A map in which the atomic entities and bonds are organized into substituents
/// and molecules, with positions in `D`-dimensional space.
#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub struct SpatialMolMap<const D: usize, U> {
    graph: MolMap,
    atom_positions: SecondaryMap<AtomKey, Point<D>>,
    pseudoatom_positions: SecondaryMap<PseudoatomKey, Point<D>>,
    unit: PhantomData<U>,
}

impl<const D: usize, U> SpatialMolMap<D, U> {
    pub fn new() -> Self {
        Self {
            graph: MolMap::new(),
            atom_positions: SecondaryMap::new(),
            pseudoatom_positions: SecondaryMap::new(),
            unit: PhantomData,
        }
    }
}

impl<const D: usize, U> Map for SpatialMolMap<D, U> {
    type Graph = MolMap;

    fn into_graph(self) -> Self::Graph {
        self.graph
    }

    fn as_graph(&self) -> &Self::Graph {
        &self.graph
    }

    fn into_atomic(self) -> AtomMap {
        self.graph.into_atomic()
    }

    fn as_atomic(&self) -> &AtomMap {
        self.graph.as_atomic()
    }

    fn contains<E: Entity>(&self, entity: E) -> bool {
        self.as_graph().contains(entity)
    }
}

impl<const D: usize, U> Molecular for SpatialMolMap<D, U> {
    fn into_molecular(self) -> MolMap {
        self.graph
    }

    fn as_molecular(&self) -> &Self::Graph {
        &self.graph
    }
}

impl<const D: usize, U: Unit> Spatial for SpatialMolMap<D, U> {
    type Unit = U;

    type DimName = na::Const<D>;

    type Point = Point<D>;

    type Vector = Vector<D>;
}

impl<const D: usize, U> SpatialMolMap<D, U> {
    // /// Calculates the unweighted geometric centre of the substituent,
    // /// or `None` if the molecule is empty.
    // ///
    // /// If `centres_only` is `true`, the centroid of the substituent's centre(s) will be
    // /// returned; in the typical case where substituent a only has one centre, this will
    // /// simply be the position of the central atomlike.
    // ///
    // /// Only the positions of the constituent atoms and pseudoatoms are taken into
    // /// consideration, not the bonds, nor any other member fundamentals.
    // ///
    // /// # Panics
    // ///
    // /// Panics if the substituent is not in the map.
    // pub(crate) fn substituent_centroid(
    //     &self,
    //     substituent: Substituent,
    //     centres_only: bool,
    // ) -> Option<Point<D>> {
    //     let data = self
    //         .graph
    //         .data(substituent)
    //         .expect("Caller is required to ensure that the ID is valid");
    //     if centres_only {
    //         match &data.centre() {
    //             // Early return if substituent is empty/has no centre (should be the same thing)
    //             SubstituentCentre::None => None,
    //             // Early return if single centre (no need to take average)
    //             SubstituentCentre::Single(centre) => Some(self.atomlike_position(*centre)).copied(),
    //             // Only if there are multiple centres do we need to proceed to find the centroid
    //             SubstituentCentre::Multiple(centres) => {
    //                 let positions = centres.iter().map(|&x| self.atomlike_position(x));
    //                 Self::mean_point(positions)
    //             }
    //         }
    //     } else {
    //         let positions = data
    //             .members
    //             .iter()
    //             .filter_map(|&x| AnyAtomlike::try_from(x).ok())
    //             .map(|x| self.atomlike_position(x));
    //         Self::mean_point(positions)
    //     }
    // }
    //
    //    /// Returns an iterator over the positions of the constituent atoms and
    //    /// pseudoatoms of a molecule.
    //    ///
    //    /// Only the positions of the constituent atomlikes are included,
    //    /// not the bonds, nor any other member fundamentals.
    //    ///
    //    /// # Panics
    //    ///
    //    /// Panics if the molecule is not in the map.
    //    pub(crate) fn molecule_member_positions(
    //        &self,
    //        molecule: Molecule,
    //    ) -> impl Iterator<Item = &Point<f64, D>> {
    //        let members = self
    //            .core
    //            .data(molecule)
    //            .expect("Caller is required to ensure that the ID is valid")
    //            .members
    //            .iter();
    //        let atomlikes = members.filter_map(|&x| AnyAtomlike::try_from(x).ok());
    //        atomlikes.map(|x| self.atomlike_position(x))
    //    }
    //
    //    /// Calculates the unweighted geometric centre of the molecule,
    //    /// or `None` if the molecule is empty.
    //    ///
    //    /// Only the positions of the constituent atoms and pseudoatoms are taken into
    //    /// consideration, not the bonds, nor any other member fundamentals.
    //    ///
    //    /// # Panics
    //    ///
    //    /// Panics if the molecule is not in the map.
    //    pub(crate) fn molecule_centroid(&self, molecule: Molecule) -> Option<Point<f64, D>> {
    //        let positions = self.molecule_member_positions(molecule);
    //        // Centroid is unweighted average of all these positions
    //        Self::mean_point(positions)
    //    }
}

/// View methods specific to `SpatialMolMap`.
mod views {
    //use super::*;

    //impl<'m, const D: usize> View<'m, SpatialMolMap<D>, Atom> {
    //    /// Returns the position of the atom.
    //    pub fn position(&self) -> &Point<f64, D> {
    //        self.map.atom_position(self.id)
    //    }
    //}
    //
    //impl<'m, const D: usize> View<'m, SpatialMolMap<D>, Pseudoatom> {
    //    /// Returns the position of the pseudoatom.
    //    pub fn position(&self) -> &Point<f64, D> {
    //        self.map.pseudoatom_position(self.id)
    //    }
    //}
    //
    //impl<'m, const D: usize> View<'m, SpatialMolMap<D>, Bond> {
    //    /// Returns the position of the [`Bondable`] from which the bond starts.
    //    pub fn origin(&self) -> &Point<f64, D> {
    //        self.map.bond_origin(self.id)
    //    }
    //
    //    /// Returns the position of the [`Bondable`] at which the bond ends.
    //    pub fn terminus(&self) -> &Point<f64, D> {
    //        self.map.bond_terminus(self.id)
    //    }
    //
    //    /// Calculates the midpoint of the bond.
    //    pub fn midpoint(&self) -> Point<f64, D> {
    //        self.map.bond_midpoint(self.id)
    //    }
    //
    //    /// Calculates the vector that the bond follows from its origin to its terminus.
    //    pub fn vector(&self) -> Vector<f64, D> {
    //        self.map.bond_vector(self.id)
    //    }
    //
    //    /// Calculates the distance from the bond's origin to its terminus.
    //    pub fn length(&self) -> f64 {
    //        self.vector().magnitude()
    //    }
    //}
    //
    //impl<'m, const D: usize> View<'m, SpatialMolMap<D>, Substituent> {
    //    /// Calculates the unweighted geometric centre of the substituent,
    //    /// or `None` if the molecule is empty.
    //    ///
    //    /// If `centres_only` is `true`, the centroid of the substituent's centre(s) will be
    //    /// returned; in the typical case where substituent a only has one centre, this will
    //    /// simply be the position of the central atomlike.
    //    ///
    //    /// Only the positions of the constituent atoms and pseudoatoms are taken into
    //    /// consideration, not the bonds, nor any other member fundamentals.
    //    pub fn centroid(&self, centres_only: bool) -> Option<Point<f64, D>> {
    //        self.map.substituent_centroid(self.id, centres_only)
    //    }
    //}
    //
    //impl<'m, const D: usize> View<'m, SpatialMolMap<D>, Molecule> {
    //    /// Calculates the unweighted geometric centre of the molecule,
    //    /// or `None` if the molecule is empty.
    //    ///
    //    /// Only the positions of the constituent atoms and pseudoatoms are taken into
    //    /// consideration, not the bonds, nor any other member fundamentals.
    //    pub fn centroid(&self) -> Option<Point<f64, D>> {
    //        self.map.molecule_centroid(self.id)
    //    }
    //}
    //
    //
    ///// Methods for setting positions and applying transformations.
    //impl<const D: usize> SpatialMolMap<D> {
    //    /// Sets the position of the given atom to the provided value.
    //    ///
    //    /// This does not panic – if the atom is not in the map, nothing happens.
    //    pub(crate) fn set_atom_position(&mut self, atom: Atom, position: Point<f64, D>) {
    //        self.atom_positions.insert(atom.to_key(), position);
    //    }
    //
    //    /// Sets the position of the given pseudoatom to the provided value.
    //    ///
    //    /// This does not panic – if the pseudoatom is not in the map, nothing happens.
    //    pub(crate) fn set_pseudoatom_position(
    //        &mut self,
    //        pseudoatom: Pseudoatom,
    //        position: Point<f64, D>,
    //    ) {
    //        self.pseudoatom_positions
    //            .insert(pseudoatom.to_key(), position);
    //    }
    //}
    //
    //impl<'m, const D: usize> ViewMut<'m, SpatialMolMap<D>, Atom> {
    //    /// Sets the position of the atom to the provided value.
    //    pub fn set_position(self, position: Point<f64, D>) {
    //        self.map.set_atom_position(self.id, position)
    //    }
    //}
    //
    //impl<'m, const D: usize> ViewMut<'m, SpatialMolMap<D>, Pseudoatom> {
    //    /// Sets the position of the pseudoatom to the provided value.
    //    pub fn set_position(self, position: Point<f64, D>) {
    //        self.map.set_pseudoatom_position(self.id, position)
    //    }
    //}
}

/// A map holding atomic entities and bonds organized into substituents
/// and molecules, with positions in two-dimensional space.
pub type MolMap2 = SpatialMolMap<2, Relative>;

/// A map holding atomic entities and bonds organized into substituents
/// and molecules, with positions in three-dimensional space.
pub type MolMap3 = SpatialMolMap<3, Angstrom>;
