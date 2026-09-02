// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use nalgebra::Point;
use nalgebra::{self as na, SVector};
use slotmap::SecondaryMap;

use crate::graph::entities::SubstituentCentre;
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
    pub(crate) fn atomlike_position(&self, atomlike: impl Atomlike) -> &Point<f64, D> {
        match Atomlike::to_resolved(atomlike) {
            ResolvedAtomlike::Atom(atom) => self.atom_position(atom),
            ResolvedAtomlike::Pseudoatom(pseudoatom) => self.pseudoatom_position(pseudoatom),
        }
    }

    /// Calculates the vector of the line between two atomlikes, from `a` to `b`.
    ///
    /// # Panics
    ///
    /// Panics if either atomlike is not in the map.
    pub(crate) fn interatomlike_line(&self, a: impl Atomlike, b: impl Atomlike) -> Vector<f64, D> {
        self.atomlike_position(b) - self.atomlike_position(a)
    }

    /// Returns the position of the [`Bondable`] from which the bond starts.
    ///
    /// # Panics
    ///
    /// Panics if the bond is not in the map.
    pub(crate) fn bond_origin(&self, bond: Bond) -> &Point<f64, D> {
        match self.core().data(bond).start.resolve() {
            ResolvedBondable::Atom(atom) => self.atom_position(atom),
            ResolvedBondable::Pseudoatom(pseudoatom) => self.pseudoatom_position(pseudoatom),
        }
    }

    /// Returns the position of the [`Bondable`] at which the bond ends.
    ///
    /// # Panics
    ///
    /// Panics if the bond is not in the map.
    pub(crate) fn bond_terminus(&self, bond: Bond) -> &Point<f64, D> {
        match self.core().data(bond).end.resolve() {
            ResolvedBondable::Atom(atom) => self.atom_position(atom),
            ResolvedBondable::Pseudoatom(pseudoatom) => self.pseudoatom_position(pseudoatom),
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

    /// Calculates the mean position from a set of positions, or `None` if the iterator is empty.
    pub(crate) fn mean_point<'a, I>(positions: I) -> Option<Point<f64, D>>
    where
        I: IntoIterator<Item = &'a Point<f64, D>>,
    {
        let mut count: u32 = 0;
        let mut sum: SVector<f64, D> = SVector::zeros();
        for pos in positions {
            count += 1;
            sum = sum + pos.coords;
        }
        if count == 0 {
            None
        } else {
            let avg = sum / f64::from(count);
            Some(Point::from(avg))
        }
    }

    /// Calculates the unweighted geometric centre of the substituent,
    /// or `None` if the molecule is empty.
    ///
    /// If `centres_only` is `true`, the centroid of the substituent's centre(s) will be
    /// returned; in the typical case where substituent a only has one centre, this will
    /// simply be the position of the central atomlike.
    ///
    /// Only the positions of the constituent atoms and pseudoatoms are taken into
    /// consideration, not the bonds, nor any other member fundamentals.
    pub(crate) fn substituent_centroid(
        &self,
        substituent: Substituent,
        centres_only: bool,
    ) -> Option<Point<f64, D>> {
        let data = self.core.data(substituent);
        if centres_only {
            match &data.centre {
                // Early return if substituent is empty/has no centre (should be the same thing)
                SubstituentCentre::None => return None,
                // Early return if single centre (no need to take average)
                SubstituentCentre::Single(centre) => {
                    return Some(self.atomlike_position(*centre)).copied();
                }
                // Only if there are multiple centres do we need to proceed to find the centroid
                SubstituentCentre::Multiple(centres) => {
                    let positions = centres.iter().map(|&x| self.atomlike_position(x));
                    Self::mean_point(positions)
                }
            }
        } else {
            let positions = data
                .members
                .iter()
                .filter_map(|&x| AnyAtomlike::try_from(x).ok())
                .map(|x| self.atomlike_position(x));
            Self::mean_point(positions)
        }
    }

    /// Returns an iterator over the positions of the constituent atoms and
    /// pseudoatoms of a molecule.
    ///
    /// Only the positions of the constituent atomlikes are included,
    /// not the bonds, nor any other member fundamentals.
    pub(crate) fn molecule_member_positions(
        &self,
        molecule: Molecule,
    ) -> impl Iterator<Item = &Point<f64, D>> {
        let members = self.core.data(molecule).members.iter();
        let atomlikes = members.filter_map(|&x| AnyAtomlike::try_from(x).ok());
        atomlikes.map(|x| self.atomlike_position(x))
    }

    /// Calculates the unweighted geometric centre of the molecule,
    /// or `None` if the molecule is empty.
    ///
    /// Only the positions of the constituent atoms and pseudoatoms are taken into
    /// consideration, not the bonds, nor any other member fundamentals.
    pub(crate) fn molecule_centroid(&self, molecule: Molecule) -> Option<Point<f64, D>> {
        let positions = self.molecule_member_positions(molecule);
        // Centroid is unweighted average of all these positions
        Self::mean_point(positions)
    }
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

impl<'m, const D: usize> View<'m, SpatialMolMap<D>, Substituent> {
    /// Calculates the unweighted geometric centre of the substituent,
    /// or `None` if the molecule is empty.
    ///
    /// If `centres_only` is `true`, the centroid of the substituent's centre(s) will be
    /// returned; in the typical case where substituent a only has one centre, this will
    /// simply be the position of the central atomlike.
    ///
    /// Only the positions of the constituent atoms and pseudoatoms are taken into
    /// consideration, not the bonds, nor any other member fundamentals.
    pub fn centroid(&self, centres_only: bool) -> Option<Point<f64, D>> {
        self.map.substituent_centroid(self.id, centres_only)
    }
}

impl<'m, const D: usize> View<'m, SpatialMolMap<D>, Molecule> {
    /// Calculates the unweighted geometric centre of the molecule,
    /// or `None` if the molecule is empty.
    ///
    /// Only the positions of the constituent atoms and pseudoatoms are taken into
    /// consideration, not the bonds, nor any other member fundamentals.
    pub fn centroid(&self) -> Option<Point<f64, D>> {
        self.map.molecule_centroid(self.id)
    }
}
