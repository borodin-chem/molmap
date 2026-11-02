// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Functionality generic over any spatial `MolMap`.

use std::marker::PhantomData;

use slotmap::SecondaryMap;

use crate::{
    Map, MolMapResult, Point, Spatial, Vector,
    atomic::AtomMap,
    entities::*,
    spatial::SpatialCommon,
    units::{Angstrom, Relative, Unit},
};

use super::{MolMap, Molecular, data::SubstituentCentre};

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

impl<const D: usize, U: Unit> Spatial<D> for SpatialMolMap<D, U> {
    type Unit = U;
}

impl<const D: usize, U: Unit> SpatialCommon<D> for SpatialMolMap<D, U> {
    fn atom_position(&self, atom: Atom) -> Option<&Point<D>> {
        // When an entity is removed from the map, any spatial data is not deleted.
        // If the slot has not since been overwritten, it is possible to obtain a stale
        // position from the SecondaryMap using the stale ID, so we must first confirm
        // the ID's validity.
        if self.contains(atom) {
            self.atom_positions.get(atom.into())
        } else {
            None
        }
    }

    fn set_atom_position(&mut self, atom: Atom, new: Point<D>) -> Option<Point<D>> {
        // Here, slotmap's insert method takes care of the checking for us
        // The promised behaviour of set_position matches that of slotmap::insert
        self.atom_positions.insert(atom.into(), new)
    }

    fn pseudoatom_position(&self, pseudoatom: Pseudoatom) -> Option<&Point<D>> {
        if self.contains(pseudoatom) {
            self.pseudoatom_positions.get(pseudoatom.into())
        } else {
            None
        }
    }

    fn set_pseudoatom_position(
        &mut self,
        pseudoatom: Pseudoatom,
        new: Point<D>,
    ) -> Option<Point<D>> {
        self.pseudoatom_positions.insert(pseudoatom.into(), new)
    }
}

/// Methods for accessing and calculating positions.
impl<const D: usize, U: Unit> SpatialMolMap<D, U> {
    /// Calculates the unweighted geometric centre of the substituent.
    ///
    /// If `centres_only` is `true`, the centroid of the substituent's centre(s) will be
    /// returned; in the typical case where substituent a only has one centre, this will
    /// simply be the position of the central atomlike.
    ///
    /// Only the positions of the constituent atoms and pseudoatoms are taken into
    /// consideration, not the bonds, nor any other member fundamentals.
    ///
    /// # Errors
    ///
    /// Fails if the substituent is not in the map.
    pub(crate) fn substituent_centroid(
        &self,
        substituent: Substituent,
        centres_only: bool,
    ) -> MolMapResult<Point<D>> {
        let centroid = if centres_only {
            match self.graph.substituent_centres(substituent)? {
                // Early return if substituent is empty/has no centre (should be the same thing)
                SubstituentCentre::None => unreachable!("Substituent should always have a centre"),
                // Early return if single centre (no need to take average)
                SubstituentCentre::Single(centre) => *self
                    .atomlike_position(centre)
                    .expect("Substituent's centre should always exist"),
                // Only if there are multiple centres do we need to proceed to find the centroid
                SubstituentCentre::Multiple(centres) => self
                    .centroid(centres.iter().copied())
                    .expect("All substituent members should exist"),
            }
        } else {
            let members = self
                .graph
                .substituent_members(substituent)?
                .filter_map(|x| AnyAtomlike::try_from(x).ok());
            self.centroid(members).unwrap()
        };
        Ok(centroid)
    }

    /// Calculates the unweighted geometric centre of the molecule.
    ///
    /// Only the positions of the constituent atoms and pseudoatoms are taken into
    /// consideration, not the bonds, nor any other member fundamentals.
    ///
    /// # Errors
    ///
    /// Fails if the molecule is not in the map.
    pub(crate) fn molecule_centroid(&self, molecule: Molecule) -> MolMapResult<Point<D>> {
        let members = self
            .graph
            .molecule_members(molecule)?
            .filter_map(|x| AnyAtomlike::try_from(x).ok());
        // Centroid is unweighted average of the atomlike positions
        Ok(self.centroid(members).expect(
            "All molecule members should exist and molecule should have at least one member",
        ))
    }
}

/// View methods specific to `SpatialMolMap`.
mod views {
    use crate::View;

    use super::*;

    impl<'m, const D: usize, U: Unit> View<'m, SpatialMolMap<D, U>, Atom> {
        /// Returns the position of the atom.
        pub fn position(&self) -> &Point<D> {
            self.map.atom_position(self.id).unwrap()
        }
    }

    impl<'m, const D: usize, U: Unit> View<'m, SpatialMolMap<D, U>, Pseudoatom> {
        /// Returns the position of the pseudoatom.
        pub fn position(&self) -> &Point<D> {
            self.map.pseudoatom_position(self.id).unwrap()
        }
    }

    impl<'m, const D: usize, U: Unit> View<'m, SpatialMolMap<D, U>, Bond> {
        /// Returns the position of the [`Bondable`] from which the bond starts.
        pub fn origin(&self) -> &Point<D> {
            self.map.bond_origin(self.id).unwrap()
        }

        /// Returns the position of the [`Bondable`] at which the bond ends.
        pub fn terminus(&self) -> &Point<D> {
            self.map.bond_terminus(self.id).unwrap()
        }

        /// Calculates the midpoint of the bond.
        pub fn midpoint(&self) -> Point<D> {
            self.map.bond_midpoint(self.id).unwrap()
        }

        /// Calculates the vector that the bond follows from its origin to its terminus.
        pub fn vector(&self) -> Vector<D> {
            self.map.bond_vector(self.id).unwrap()
        }

        /// Calculates the distance from the bond's origin to its terminus.
        pub fn length(&self) -> f64 {
            self.vector().magnitude()
        }
    }

    impl<'m, const D: usize, U: Unit> View<'m, SpatialMolMap<D, U>, Substituent> {
        /// Calculates the unweighted geometric centre of the substituent.
        ///
        /// If `centres_only` is `true`, the centroid of the substituent's centre(s) will be
        /// returned; in the typical case where substituent a only has one centre, this will
        /// simply be the position of the central atomlike.
        ///
        /// Only the positions of the constituent atoms and pseudoatoms are taken into
        /// consideration, not the bonds, nor any other member fundamentals.
        pub fn centroid(&self, centres_only: bool) -> Point<D> {
            self.map
                .substituent_centroid(self.id, centres_only)
                .unwrap()
        }
    }

    impl<'m, const D: usize, U: Unit> View<'m, SpatialMolMap<D, U>, Molecule> {
        /// Calculates the unweighted geometric centre of the molecule.
        ///
        /// Only the positions of the constituent atoms and pseudoatoms are taken into
        /// consideration, not the bonds, nor any other member fundamentals.
        pub fn centroid(&self) -> Point<D> {
            self.map.molecule_centroid(self.id).unwrap()
        }
    }
}

/// A map holding atomic entities and bonds organized into substituents
/// and molecules, with positions in two-dimensional space.
pub type MolMap2 = SpatialMolMap<2, Relative>;

/// A map holding atomic entities and bonds organized into substituents
/// and molecules, with positions in three-dimensional space.
pub type MolMap3 = SpatialMolMap<3, Angstrom>;
