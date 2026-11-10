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
    bonding::BondType,
    elements::{Element, Pseudoelement},
    entities::*,
    error::{MolMapError, MolMapResult},
    geometry::{self, Point, Vector},
    traits::{Map, Spatial, Stores},
    units::{Angstrom, Relative, Unit},
};

use super::{AtomMap, Atomic};

/// Methods common to all spatial maps.
///
/// The purpose of this trait is to allow internal methods common to several spatial
/// map types to be written only once. The trait and its methods are not intended to
/// be exposed publicly. Functionality that should be available as part of the
/// public API is generally implemented via view methods, or in some cases via
/// inherent wrapper methods on the map type itself.
///
/// The current way to express that a map is both "spatial" and "atomic" via bounds
/// is to use both the respective traits as bounds. However, that does not allow
/// generic use of this trait's methods. Should a need for that arise in future,
/// `pub trait SpatialAtomic<const D: usize>: Spatial<D> + Atomic + SpatialAtomicCommon<D>`
/// should be introduced and blanket implemented for all maps that fulfil the
/// bounds -- this trait should *not* simply be made public.
pub(crate) trait SpatialAtomicCommon<const D: usize>: Spatial<D> + Atomic {
    /// Returns the position of the atom, or `None` if it is invalid.
    fn atom_position(&self, atom: Atom) -> Option<&Point<D>>;

    /// Sets the position of the atom.
    ///
    /// Can silently fail and return `None` if the atom was removed.
    ///
    /// Returns `None` if the atom did not previously have a position, or the
    /// old value otherwise.
    ///
    /// As such, this method does not panic – if the atom is not in the map,
    /// nothing happens, and `None` is returned.
    fn set_atom_position(&mut self, atom: Atom, new: Point<D>) -> Option<Point<D>>;

    /// Returns the position of the pseudoatom, or `None` if it is invalid.
    fn pseudoatom_position(&self, pseudoatom: Pseudoatom) -> Option<&Point<D>>;

    /// Sets the position of the pseudoatom.
    ///
    /// Can silently fail and return `None` if the pseudoatom was removed.
    ///
    /// Returns `None` if the pseudoatom did not previously have a position, or the
    /// old value otherwise.
    ///
    /// As such, this method does not panic – if the pseudoatom is not in the map,
    /// nothing happens, and `None` is returned.
    fn set_pseudoatom_position(
        &mut self,
        pseudoatom: Pseudoatom,
        new: Point<D>,
    ) -> Option<Point<D>>;

    /// Returns the position of the atom or pseudoatom, or `None` if it is invalid.
    fn atomlike_position(&self, atomlike: impl Atomlike) -> Option<&Point<D>> {
        match Atomlike::to_resolved(atomlike) {
            ResolvedAtomlike::Atom(atom) => self.atom_position(atom),
            ResolvedAtomlike::Pseudoatom(pseudoatom) => self.pseudoatom_position(pseudoatom),
        }
    }

    /// Sets the position of the atom or pseudoatom.
    ///
    /// Can silently fail and return `None` if the atomlike was removed.
    ///
    /// Returns `None` if the atomlike did not previously have a position, or the
    /// old value otherwise.
    ///
    /// As such, this method does not panic – if the atomlike is not in the map,
    /// nothing happens, and `None` is returned.
    #[expect(unused)]
    fn set_atomlike_position(
        &mut self,
        atomlike: impl Atomlike,
        new: Point<D>,
    ) -> Option<Point<D>> {
        match Atomlike::to_resolved(atomlike) {
            ResolvedAtomlike::Atom(atom) => self.set_atom_position(atom, new),
            ResolvedAtomlike::Pseudoatom(pseudoatom) => {
                self.set_pseudoatom_position(pseudoatom, new)
            }
        }
    }

    /// Returns the position of the bondable, or `None` if it is invalid.
    fn bondable_position(&self, bondable: impl Bondable) -> Option<&Point<D>> {
        match Bondable::to_resolved(bondable) {
            ResolvedBondable::Atom(atom) => self.atom_position(atom),
            ResolvedBondable::Pseudoatom(pseudoatom) => self.pseudoatom_position(pseudoatom),
        }
    }

    /// Calculates the vector of the line between two atomlikes, from `a` to `b`.
    ///
    /// # Errors
    ///
    /// Fails if either atomlike has no stored position (usually meaning that it is not
    /// in the map).
    #[expect(unused)]
    fn interatomlike_line(&self, a: impl Atomlike, b: impl Atomlike) -> MolMapResult<Vector<D>> {
        let pos_a = self
            .atomlike_position(b)
            .ok_or(MolMapError::InvalidId(b.as_entity()))?;
        let pos_b = self
            .atomlike_position(a)
            .ok_or(MolMapError::InvalidId(a.as_entity()))?;
        Ok(pos_b - pos_a)
    }

    /// Returns the position of the [`Bondable`] from which the bond starts.
    ///
    /// # Errors
    ///
    /// Fails if the bond is not in the map.
    fn bond_origin(&self, bond: Bond) -> MolMapResult<&Point<D>> {
        let bond_data = self
            .as_atomic()
            .data(bond)
            .ok_or(MolMapError::InvalidId(bond.into()))?;
        let origin = self.bondable_position(bond_data.start).expect("If the bond exists, the bonding partners must exist, and any existing atom must have a position");
        Ok(origin)
    }

    /// Returns the position of the [`Bondable`] at which the bond ends.
    ///
    /// # Errors
    ///
    /// Fails if the bond is not in the map.
    fn bond_terminus(&self, bond: Bond) -> MolMapResult<&Point<D>> {
        let bond_data = self
            .as_atomic()
            .data(bond)
            .ok_or(MolMapError::InvalidId(bond.into()))?;
        let terminus = self.bondable_position(bond_data.end).expect("If the bond exists, the bonding partners must exist, and any existing atom must have a position");
        Ok(terminus)
    }

    /// Calculates the midpoint of the bond.
    ///
    /// # Errors
    ///
    /// Fails if the bond is not in the map.
    fn bond_midpoint(&self, bond: Bond) -> MolMapResult<Point<D>> {
        let origin = self.bond_origin(bond)?;
        let terminus = self.bond_terminus(bond)?;
        let midpoint = na::center(origin, terminus);
        Ok(midpoint)
    }

    /// Calculates the vector that the bond follows from its origin to its terminus.
    ///
    /// # Errors
    ///
    /// Fails if the bond is not in the map.
    fn bond_vector(&self, bond: Bond) -> MolMapResult<Vector<D>> {
        let origin = self.bond_origin(bond)?;
        let terminus = self.bond_terminus(bond)?;
        Ok(terminus - origin)
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
    fn centroid(&self, atomlikes: impl IntoIterator<Item = AnyAtomlike>) -> MolMapResult<Point<D>> {
        let positions = atomlikes
            .into_iter()
            .map(|x| {
                self.atomlike_position(x)
                    .ok_or(MolMapError::InvalidId(x.as_entity()))
            })
            .collect::<MolMapResult<Vec<&Point<D>>>>()?;
        geometry::mean_point(positions).ok_or(MolMapError::EmptyIterator)
    }
}

/// An [`AtomMap`] extended to also hold the positions of its entities in `D`-dimensional space.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    fn contains<E: Entity>(&self, entity: E) -> bool {
        self.as_graph().contains(entity)
    }
}

impl<const D: usize, U> Atomic for SpatialAtomMap<D, U> {
    fn into_atomic(self) -> AtomMap {
        self.graph
    }

    fn as_atomic(&self) -> &AtomMap {
        &self.graph
    }
}

impl<const D: usize, U: Unit> Spatial<D> for SpatialAtomMap<D, U> {
    type Unit = U;
}

impl<const D: usize, U: Unit> SpatialAtomicCommon<D> for SpatialAtomMap<D, U> {
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

/// Methods for entity addition.
impl<const D: usize, U: Unit> SpatialAtomMap<D, U> {
    /// Adds an atom to the map at the given position.
    pub fn add_atom(&mut self, element: Element, position: Point<D>) -> Atom {
        let atom = self.graph.add_atom(element);
        self.set_atom_position(atom, position);
        atom
    }

    /// Adds a pseudoatom to the map at the given position.
    pub fn add_pseudoatom(
        &mut self,
        pseudoelement: Pseudoelement,
        position: Point<D>,
    ) -> Pseudoatom {
        let pseudoatom = self.graph.add_pseudoatom(pseudoelement);
        self.set_pseudoatom_position(pseudoatom, position);
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

/// View methods specific to `SpatialAtomMap`.
mod views {
    use crate::{View, ViewMut};

    use super::*;

    impl<'m, const D: usize, U: Unit> View<'m, SpatialAtomMap<D, U>, Atom> {
        /// Returns the position of the atom.
        pub fn position(&self) -> &Point<D> {
            self.map.atom_position(self.id).unwrap()
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

        // /// Sets the position of the atom to the provided value.
        // pub fn set_position(self, position: Point<D>) {
        //     self.map.set_position(self.id, position);
        // }
    }

    impl<'m, const D: usize, U: Unit> View<'m, SpatialAtomMap<D, U>, Pseudoatom> {
        /// Returns the position of the pseudoatom.
        pub fn position(&self) -> &Point<D> {
            self.map.pseudoatom_position(self.id).unwrap()
        }
    }

    impl<'m, const D: usize, U: Unit> ViewMut<'m, SpatialAtomMap<D, U>, Pseudoatom> {
        /// Removes the pseudoatom from the map, as well as any bonds to it.
        pub fn delete(self) {
            self.map.graph.delete_pseudoatom(self.id);
        }

        // /// Sets the position of the pseudoatom to the provided value.
        // pub fn set_position(self, position: Point<D>) {
        //     self.map.set_position(self.id, position);
        // }
    }

    impl<'m, const D: usize, U: Unit> View<'m, SpatialAtomMap<D, U>, Bond> {
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

    impl<'m, const D: usize, U: Unit> ViewMut<'m, SpatialAtomMap<D, U>, Bond> {
        /// Removes the bond from the map (but not its bonding partners).
        pub fn delete(self) {
            self.map.graph.delete_bond(self.id);
        }
    }
}

/// An [`AtomMap`] extended to also hold the two-dimensional positions of its entities.
pub type AtomMap2 = SpatialAtomMap<2, Relative>;

/// An [`AtomMap`] extended to also hold the three-dimensional positions of its entities.
pub type AtomMap3 = SpatialAtomMap<3, Angstrom>;

#[cfg(test)]
#[allow(unused)]
mod tests {
    use crate::{
        geometry::{Point2, Point3},
        units::{Angstrom, Relative},
    };

    use super::*;

    #[test]
    fn add_atom() {
        let mut am: SpatialAtomMap<2, Angstrom> = SpatialAtomMap::new();
        assert_eq!(am.iter::<Atom>().count(), 0);
        let h1 = am.add_atom(Element::H, Point2::new(1.0, 2.0));
        assert_eq!(am.iter::<Atom>().count(), 1);
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
        assert_eq!(am.iter::<Atom>().count(), 1);
        assert_eq!(am.atom_positions.len(), 1);
        am.view_mut(h1).unwrap().delete();
        assert_eq!(am.iter::<Atom>().count(), 0);
        // Note that stale position is still present, but can't be accessed
        assert!(am.atom_positions.contains_key(h1.to_key()));
        assert!(am.view(h1).is_none());
    }
}
