// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Functionality generic over any spatial `AtomMap`.

use std::marker::PhantomData;

use nalgebra as na;
use slotmap::SecondaryMap;

use crate::{
    BondType, Element, Map, MolMapError, MolMapResult, Point, Pseudoelement, Spatial, Vector, View,
    ViewMut,
    entities::*,
    geometry,
    traits::{Stores, StoresPosition},
    units::{Angstrom, Relative, Unit},
};

use super::AtomMap;

/// A map holding only fundamental entities and their positions in `D`-dimensional space.
#[derive(Clone, Debug, Default)]
pub struct SpatialAtomMap<const D: usize, U> {
    graph: AtomMap,
    atom_positions: SecondaryMap<AtomKey, Point<D>>,
    pseudoatom_positions: SecondaryMap<PseudoatomKey, Point<D>>,
    unit: PhantomData<U>,
}

impl<const D: usize, U> SpatialAtomMap<D, U> {
    pub fn new() -> Self {
        Self {
            graph: AtomMap::new(),
            atom_positions: SecondaryMap::new(),
            pseudoatom_positions: SecondaryMap::new(),
            unit: PhantomData,
        }
    }
}

impl<const D: usize, U> Map for SpatialAtomMap<D, U> {
    type Graph = AtomMap;

    fn into_graph(self) -> Self::Graph {
        self.graph
    }

    fn as_graph(&self) -> &Self::Graph {
        &self.graph
    }

    fn into_atomic(self) -> AtomMap {
        self.graph
    }

    fn as_atomic(&self) -> &AtomMap {
        &self.graph
    }

    fn contains<E: Entity>(&self, entity: E) -> bool {
        self.as_graph().contains(entity)
    }
}

impl<const D: usize, U: Unit> Spatial for SpatialAtomMap<D, U> {
    type Unit = U;

    type DimName = na::Const<D>;

    type Point = Point<D>;

    type Vector = Vector<D>;
}

/// Methods for entity addition.
impl<const D: usize, U: Unit> SpatialAtomMap<D, U> {
    /// Adds an atom to the map at the given position.
    pub fn add_atom(&mut self, element: Element, position: Point<D>) -> Atom {
        let atom = self.graph.add_atom(element);
        self.set_position(atom, position);
        atom
    }

    /// Adds a pseudoatom to the map at the given position.
    pub fn add_pseudoatom(
        &mut self,
        pseudoelement: Pseudoelement,
        position: Point<D>,
    ) -> Pseudoatom {
        let pseudoatom = self.graph.add_pseudoatom(pseudoelement);
        self.set_position(pseudoatom, position);
        pseudoatom
    }

    /// Creates a new (single covalent) bond between two bondable entities.
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
        Ok(self.graph.add_bond_unchecked(bond_type, start, end))
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

// Methods for accessing and calculating positions.
impl<const D: usize, U: Unit> SpatialAtomMap<D, U> {
    /// Returns the position of the given atom or pseudoatom.
    ///
    /// A stale (i.e. invalid) ID may return a position, as spatial data is not
    /// automatically cleaned up. The caller is responsible for not making such a
    /// request.
    ///
    /// # Panics
    ///
    /// Panics if the atomlike has no stored position (usually meaning that it is not
    /// in the map).
    pub(crate) fn atomlike_position(&self, atomlike: impl Atomlike) -> &Point<D> {
        match Atomlike::to_resolved(atomlike) {
            ResolvedAtomlike::Atom(atom) => self.position(atom).unwrap(),
            ResolvedAtomlike::Pseudoatom(pseudoatom) => self.position(pseudoatom).unwrap(),
        }
    }

    /// Calculates the vector of the line between two atomlikes, from `a` to `b`.
    ///
    /// Stale (i.e. invalid) IDs may return positions, as spatial data is not
    /// automatically cleaned up. The caller is responsible for not making such a
    /// request.
    ///
    /// # Panics
    ///
    /// Panics if either atomlike has no stored position (usually meaning that it is not
    /// in the map).
    #[allow(unused)]
    pub(crate) fn interatomlike_line(&self, a: impl Atomlike, b: impl Atomlike) -> Vector<D> {
        self.atomlike_position(b) - self.atomlike_position(a)
    }

    /// Returns the position of the [`Bondable`] from which the bond starts.
    ///
    /// # Panics
    ///
    /// Panics if the bond is not in the map.
    pub(crate) fn bond_origin(&self, bond: Bond) -> &Point<D> {
        match self.as_graph().data(bond).unwrap().start.resolve() {
            ResolvedBondable::Atom(atom) => self.position(atom).unwrap(),
            ResolvedBondable::Pseudoatom(pseudoatom) => self.position(pseudoatom).unwrap(),
        }
    }

    /// Returns the position of the [`Bondable`] at which the bond ends.
    ///
    /// # Panics
    ///
    /// Panics if the bond is not in the map.
    pub(crate) fn bond_terminus(&self, bond: Bond) -> &Point<D> {
        match self.as_graph().data(bond).unwrap().end.resolve() {
            ResolvedBondable::Atom(atom) => self.position(atom).unwrap(),
            ResolvedBondable::Pseudoatom(pseudoatom) => self.position(pseudoatom).unwrap(),
        }
    }

    /// Calculates the midpoint of the bond.
    ///
    /// # Panics
    ///
    /// Panics if the bond is not in the map.
    pub(crate) fn bond_midpoint(&self, bond: Bond) -> Point<D> {
        na::center(self.bond_origin(bond), self.bond_terminus(bond))
    }

    /// Calculates the vector that the bond follows from its origin to its terminus.
    ///
    /// # Panics
    ///
    /// Panics if the bond is not in the map.
    pub(crate) fn bond_vector(&self, bond: Bond) -> Vector<D> {
        self.bond_terminus(bond) - self.bond_origin(bond)
    }

    /// Calculates the unweighted geometric centre of the specified atoms and
    /// pseudoatoms.
    ///
    /// # Errors
    ///
    /// Fails (returning [`MolMapError::InvalidId`]) if any of the atomlikes are not in
    /// the map.
    ///
    /// Fails (returning [`MolMapError::EmptyIterator`]) if the provided iterator is
    /// empty.
    pub fn centroid(
        &self,
        atomlikes: impl IntoIterator<Item = AnyAtomlike>,
    ) -> MolMapResult<Point<D>> {
        let positions = atomlikes
            .into_iter()
            .map(|x| {
                if self.contains(x) {
                    Ok(self.atomlike_position(x))
                } else {
                    Err(MolMapError::InvalidId(x.as_entity()))
                }
            })
            .collect::<MolMapResult<Vec<&Point<D>>>>()?;
        geometry::mean_point(positions).ok_or(MolMapError::EmptyIterator)
    }
}

impl<const D: usize, U: Unit> StoresPosition<Atom> for SpatialAtomMap<D, U> {
    fn position(&self, entity: Atom) -> Option<&Point<D>> {
        // When an entity is removed from the map, any spatial data is not deleted.
        // Thus it is possible to obtain a stale position from the SecondaryMap that does
        // not correspond to any existing entity.
        //
        // If the entity ID is stale, it's not possible for the user to construct a
        // view of it. When working internally, it would be possible to get a spurious
        // position from a stale ID; to help avoid that, this method confirms the validity
        // of the ID first.
        if self.contains(entity) {
            self.atom_positions.get(entity.into())
        } else {
            None
        }
    }

    fn set_position(&mut self, entity: Atom, new: Self::Point) -> Option<Point<D>> {
        // Here, slotmap's insert method takes care of the checking for us
        // The promised behaviour of set_position matches that of slotmap::insert
        self.atom_positions.insert(entity.into(), new)
    }
}

impl<const D: usize, U: Unit> StoresPosition<Pseudoatom> for SpatialAtomMap<D, U> {
    fn position(&self, entity: Pseudoatom) -> Option<&Point<D>> {
        if self.contains(entity) {
            self.pseudoatom_positions.get(entity.into())
        } else {
            None
        }
    }

    fn set_position(&mut self, entity: Pseudoatom, new: Self::Point) -> Option<Point<D>> {
        self.pseudoatom_positions.insert(entity.into(), new)
    }
}

/// View methods specific to `SpatialAtomMap`.
mod views {
    use super::*;

    impl<'m, const D: usize, U: Unit> View<'m, SpatialAtomMap<D, U>, Atom> {
        /// Returns the position of the atom.
        pub fn position(&self) -> &Point<D> {
            self.map.position(self.id).unwrap()
        }
    }

    // For now the deletion methods are identical to the equivalents for AtomMap.
    // They may well diverge in future e.g. if we cache emergent information like
    // the bond vectors or the centroids or bounding boxes of the collections.
    //
    // It is not necessary to delete the positions from the SecondaryMap; they will
    // be overwritten.

    impl<'m, const D: usize, U: Unit> ViewMut<'m, SpatialAtomMap<D, U>, Atom> {
        /// Removes the atom from the map, as well as any bonds to it.
        pub fn delete(self) {
            self.map.graph.delete_atom(self.id);
        }

        /// Sets the position of the atom to the provided value.
        pub fn set_position(self, position: Point<D>) {
            self.map.set_position(self.id, position);
        }
    }

    impl<'m, const D: usize, U: Unit> View<'m, SpatialAtomMap<D, U>, Pseudoatom> {
        /// Returns the position of the pseudoatom.
        pub fn position(&self) -> &Point<D> {
            self.map.position(self.id).unwrap()
        }
    }

    impl<'m, const D: usize, U: Unit> ViewMut<'m, SpatialAtomMap<D, U>, Pseudoatom> {
        /// Removes the pseudoatom from the map, as well as any bonds to it.
        pub fn delete(self) {
            self.map.graph.delete_pseudoatom(self.id);
        }

        /// Sets the position of the pseudoatom to the provided value.
        pub fn set_position(self, position: Point<D>) {
            self.map.set_position(self.id, position);
        }
    }

    impl<'m, const D: usize, U: Unit> View<'m, SpatialAtomMap<D, U>, Bond> {
        /// Returns the position of the [`Bondable`] from which the bond starts.
        pub fn origin(&self) -> &Point<D> {
            self.map.bond_origin(self.id)
        }

        /// Returns the position of the [`Bondable`] at which the bond ends.
        pub fn terminus(&self) -> &Point<D> {
            self.map.bond_terminus(self.id)
        }

        /// Calculates the midpoint of the bond.
        pub fn midpoint(&self) -> Point<D> {
            self.map.bond_midpoint(self.id)
        }

        /// Calculates the vector that the bond follows from its origin to its terminus.
        pub fn vector(&self) -> Vector<D> {
            self.map.bond_vector(self.id)
        }

        /// Calculates the distance from the bond's origin to its terminus.
        pub fn length(&self) -> f64 {
            self.vector().magnitude()
        }
    }

    impl<'m, const D: usize, U: Unit> ViewMut<'m, SpatialAtomMap<D, U>, Bond> {
        /// Removes the bond from the map (but not its bonding partners).
        pub fn delete(self) {
            self.map.graph.delete_bond(self.id);
        }
    }
}

/// A map holding only fundamental entities and their two-dimensional positions.
pub type AtomMap2 = SpatialAtomMap<2, Relative>;

/// A map holding only fundamental entities and their three-dimensional positions.
pub type AtomMap3 = SpatialAtomMap<3, Angstrom>;

#[cfg(test)]
#[allow(unused)]
mod tests {
    use crate::{
        Point2, Point3,
        units::{Angstrom, Relative},
    };

    use super::*;

    #[test]
    fn add_atom() {
        let mut am: SpatialAtomMap<2, Angstrom> = SpatialAtomMap::new();
        assert_eq!(am.entities::<Atom>().count(), 0);
        let h1 = am.add_atom(Element::H, Point2::new(1.0, 2.0));
        assert_eq!(am.entities::<Atom>().count(), 1);
        // Confirm that the atom positions also have data for a single atom
        assert_eq!(am.atom_positions.len(), 1);
        // Confirm that the atom position can be accessed
        assert_eq!(
            am.atom_positions.get(h1.to_key()).unwrap().clone(),
            Point2::new(1.0, 2.0)
        );
    }

    #[test]
    fn delete_atom() {
        let mut am: SpatialAtomMap<3, Relative> = SpatialAtomMap::new();
        let h1 = am.add_atom(Element::H, Point3::new(1.0, 2.0, 3.0));
        assert_eq!(am.entities::<Atom>().count(), 1);
        assert_eq!(am.atom_positions.len(), 1);
        am.view_mut(h1).unwrap().delete();
        assert_eq!(am.entities::<Atom>().count(), 0);
        // Note that stale position is still present, but can't be accessed
        assert!(am.atom_positions.contains_key(h1.to_key()));
        assert!(am.view(h1).is_none());
    }
}
