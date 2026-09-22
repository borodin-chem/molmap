// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Definition of the generic [`SpatialMolMap`] type and methods for membership management.

use nalgebra as na;
use nalgebra::Point;
use slotmap::SecondaryMap;

use crate::{
    BondType, Element, Pseudoelement,
    entities::*,
    error::*,
    graph::{MolGraph, keys::*},
    molmap::{MolMap, MolMapCore},
    view::*,
};

/// A matrix with one column and D rows.
///
/// Note that `Vector<T, 2>` using this alias is just shorthand for
/// `nalgebra::Vector2<T>` (and similarly `Vector<T, 3>` is `nalgebra::Vector3<T>`).
pub type Vector<T, const D: usize> = na::Vector<T, na::Const<D>, na::ArrayStorage<T, D, 1>>;
// This is a more convenient alias than the nalgebra vector type, which leaves the storage type generic

/// A [`MolMap`] that also holds the spatial positions (with dimensionality `D`)
/// of its entities, but no further application-specific information.
///
/// Note that the units of the space are arbitrary and no assumptions should be made
/// as to what a length of 1 represents. Other types that wrap or extend a
/// `SpatialMolMap` will want to fix a value.
#[derive(Clone, Debug)]
pub struct SpatialMolMap<const D: usize> {
    pub(super) core: MolGraph,
    pub(super) atom_positions: SecondaryMap<AtomKey, Point<f64, D>>,
    pub(super) pseudoatom_positions: SecondaryMap<PseudoatomKey, Point<f64, D>>,
    ///// Bond positions are just the positions of their start and end bondable,
    ///// but the vectors of the bonds are cached due to their usefulness.
    //bonds: SecondaryMap<BondKey, Vector<f64, D>>,
}

// When an entity is removed from the map, any spatial data is not deleted.
// Thus, in theory, when working internally, it is possible to obtain a stale
// position that does not correspond to any existing entity.
//
// As the entity will have been removed from the underlying slotmap in the core
// MolGraph, the position of the entity will be inaccessible to the user, as
// obtaining a view is impossible once the entity is not in the slotmap.
//
// Stale data should not in reality be a problem internally either, as there
// should never be a situation where a request for it is made.
// For example, when calculating the centroid of a molecule, the set of member
// IDs will never contain any stale IDs anyway.
// Essentially, retaining the stale position shouldn't be an issue because no
// "references" to it should persist.
// It should however not be forgotten that the whole set of spatial data may
// contain stale items.

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

/// Methods for entity addition.
impl<const D: usize> SpatialMolMap<D> {
    /// Adds an atom to the map at the given position.
    pub fn add_atom(&mut self, element: Element, position: Point<f64, D>) -> Atom {
        let atom = self.core.add_atom(element);
        self.set_atom_position(atom, position);
        atom
    }

    /// Adds a pseudoatom to the map at the given position.
    pub fn add_pseudoatom(
        &mut self,
        pseudoelement: Pseudoelement,
        position: Point<f64, D>,
    ) -> Pseudoatom {
        let pseudoatom = self.core.add_pseudoatom(pseudoelement);
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
        Ok(self.core.add_bond(bond_type, start, end))
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

    /// Adds an empty substituent to the map.
    pub fn add_substituent(&mut self) -> Substituent {
        self.core.add_substituent()
    }

    /// Adds a substituent to the map with a single, newly-created central atom.
    ///
    /// Returns the IDs of the added substituent and central atom.
    pub fn add_substituent_with_atom(
        &mut self,
        element: Element,
        position: Point<f64, D>,
    ) -> (Substituent, Atom) {
        let centre = self.add_atom(element, position);
        let sub = self.core.add_substituent_with_centre(centre);
        (sub, centre)
    }

    /// Adds an empty molecule to the map.
    pub fn add_molecule(&mut self) -> Molecule {
        self.core.add_molecule()
    }
}

// Public API for deleting entities/changing their collection membership via mutable views
// For now these are identical to the equivalents for MolMap0, but may well
// diverge in future e.g. if we cache emergent information like the bond vectors
// or the centroids or bounding boxes of the collections

impl<'m, const D: usize> ViewMut<'m, SpatialMolMap<D>, Atom> {
    /// Removes the atom from the map, as well as any bonds to it.
    pub fn delete(self) {
        self.map.core_mut().delete_atom(self.id);
    }
}

impl<'m, const D: usize> ViewMut<'m, SpatialMolMap<D>, Pseudoatom> {
    /// Removes the pseudoatom from the map, as well as any bonds to it.
    pub fn delete(self) {
        self.map.core_mut().delete_pseudoatom(self.id);
    }
}

impl<'m, const D: usize> ViewMut<'m, SpatialMolMap<D>, Bond> {
    /// Removes the bond from the map (but not its bonding partners).
    pub fn delete(self) {
        self.map.core_mut().delete_bond(self.id);
    }
}

impl<'m, const D: usize> ViewMut<'m, SpatialMolMap<D>, Substituent> {
    /// Removes the substituent from the map, as well as all of its members.
    pub fn delete(self) {
        self.map.core_mut().delete_substituent(self.id);
    }

    /// Adds an atom, pseudoatom, or bond to the substituent.
    ///
    /// Returns whether the fundamental was newly inserted.
    ///
    /// If the fundamental is already a member of another substituent, it is removed
    /// from it before it is inserted into this one.
    ///
    /// # Errors
    ///
    /// Returns an error if the fundamental is invalid.
    pub fn insert(self, fundamental: impl Fundamental) -> MolMapResult<bool> {
        if !self.map.contains(fundamental) {
            return Err(MolMapError::InvalidId(fundamental.as_entity()));
        };
        Ok(self
            .map
            .core_mut()
            .insert_into_substituent(self.id, fundamental))
    }

    /// Adds atoms, pseudoatoms, or bonds from an iterator to the substituent.
    ///
    /// If any of the fundamentals are already members of other substituents, they are
    /// removed before they are inserted into this one.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the fundamentals are invalid, in which case the
    /// map will remain unchanged; the returned error will hold the ID of the first
    /// invalid fundamental encountered.
    pub fn extend<E, I>(self, fundamentals: I) -> MolMapResult<()>
    where
        E: Entity + Into<AnyFundamental>,
        I: IntoIterator<Item = E>,
        I::IntoIter: Clone,
    {
        let fundamentals = fundamentals.into_iter();
        for f in fundamentals.clone() {
            if !self.map.contains(f) {
                return Err(MolMapError::InvalidId(f.as_entity()));
            }
        }
        for f in fundamentals {
            self.map
                .core_mut()
                .insert_into_substituent(self.id, f.into());
        }
        Ok(())
    }

    /// Removes an atom, pseudoatom, or bond from the substituent.
    ///
    /// Returns whether the fundamental was a member of the substituent.
    ///
    /// If the fundamental is an atomlike and is the centre of the substituent,
    /// the centre is adjusted accordingly; if it is the lone centre, the
    /// substituent becomes centreless. If it is one of two centres,
    /// however, the centre remains `SubstituentCentre::Multiple` rather than
    /// becoming `Single`.
    ///
    /// The substituent continues to exist, even if empty, as does the removed
    /// fundamental.
    pub fn remove(self, fundamental: impl Fundamental) -> bool {
        self.map
            .core_mut()
            .remove_from_substituent(self.id, fundamental)
    }

    /// Empties the substituent by removing all its members, returning an iterator
    /// over the IDs of the former members.
    ///
    /// The substituent and all removed fundamentals continue to exist.
    ///
    /// After this operation, the substituent will be centreless.
    pub fn drain(self) -> impl Iterator<Item = AnyFundamental> {
        self.map.core_mut().drain_substituent(self.id)
    }

    /// Empties the substituent by deleting all its members.
    ///
    /// The substituent itself continues to exist, and will be centreless.
    pub fn clear(self) {
        self.map.core_mut().clear_substituent(self.id);
    }

    /// Empties the substituent and then removes it from the map, returning the IDs of
    /// the former members.
    ///
    /// All removed fundamentals continue to exist.
    pub fn dissolve(self) -> impl Iterator<Item = AnyFundamental> {
        self.map.core_mut().dissolve_substituent(self.id)
    }
}

impl<'m, const D: usize> ViewMut<'m, SpatialMolMap<D>, Molecule> {
    /// Removes the molecule from the map, as well as all of its members.
    pub fn delete(self) {
        self.map.core_mut().delete_molecule(self.id);
    }

    /// Adds an atom, pseudoatom, or bond to the molecule.
    ///
    /// Returns whether the fundamental was newly inserted.
    ///
    /// If the fundamental is already a member of another molecule, it is removed
    /// from it before it is inserted into this one.
    ///
    /// # Errors
    ///
    /// Returns an error if the fundamental is invalid.
    pub fn insert(self, fundamental: impl Fundamental) -> MolMapResult<bool> {
        if !self.map.contains(fundamental) {
            return Err(MolMapError::InvalidId(fundamental.as_entity()));
        };
        Ok(self
            .map
            .core_mut()
            .insert_into_molecule(self.id, fundamental))
    }

    /// Adds atoms, pseudoatoms, or bonds from an iterator to the molecule.
    ///
    /// If any of the fundamentals are already members of other molecules, they are
    /// removed before they are inserted into this one.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the fundamentals are invalid, in which case the
    /// map will remain unchanged; the returned error will hold the ID of the first
    /// invalid fundamental encountered.
    pub fn extend<E, I>(self, fundamentals: I) -> MolMapResult<()>
    where
        E: Entity + Into<AnyFundamental>,
        I: IntoIterator<Item = E>,
        I::IntoIter: Clone,
    {
        let fundamentals = fundamentals.into_iter();
        for f in fundamentals.clone() {
            if !self.map.contains(f) {
                return Err(MolMapError::InvalidId(f.as_entity()));
            }
        }
        for f in fundamentals {
            self.map.core_mut().insert_into_molecule(self.id, f.into());
        }
        Ok(())
    }

    /// Removes an atom, pseudoatom, or bond from the molecule.
    ///
    /// Returns whether the fundamental was a member of the molecule.
    ///
    /// The molecule continues to exist even if empty, as does the removed
    /// fundamental.
    pub fn remove(self, fundamental: impl Fundamental) -> bool {
        self.map
            .core_mut()
            .remove_from_molecule(self.id, fundamental)
    }

    /// Empties the molecule by removing all its members, returning an iterator
    /// over the IDs of the former members.
    ///
    /// The molecule and all removed fundamentals continue to exist.
    pub fn drain(self) -> impl Iterator<Item = AnyFundamental> {
        self.map.core_mut().drain_molecule(self.id)
    }

    /// Empties the molecule by deleting all its members.
    ///
    /// The molecule itself continues to exist.
    pub fn clear(self) {
        self.map.core_mut().clear_molecule(self.id);
    }

    /// Empties the molecule and then removes it from the map, returning the IDs of the
    /// former members.
    ///
    /// All removed fundamentals continue to exist.
    pub fn dissolve(self) -> impl Iterator<Item = AnyFundamental> {
        self.map.core_mut().dissolve_molecule(self.id)
    }
}

#[cfg(test)]
#[allow(unused)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn add_atom() {
        let mut mm = SpatialMolMap::<2>::new();
        assert_eq!(mm.all::<Atom>().count(), 0);
        let h1 = mm.add_atom(Element::H, na::Point2::new(1.0, 2.0));
        assert_eq!(mm.all::<Atom>().count(), 1);
        // Confirm that the atom positions also have data for a single atom
        assert_eq!(mm.atom_positions.len(), 1);
        // Confirm that the atom position can be accessed
        assert_eq!(
            mm.atom_positions.get(h1.to_key()).unwrap().clone(),
            na::Point2::new(1.0, 2.0)
        );
    }

    #[test]
    fn delete_atom() {
        let mut mm = SpatialMolMap::<2>::new();
        let h1 = mm.add_atom(Element::H, na::Point2::new(1.0, 2.0));
        assert_eq!(mm.all::<Atom>().count(), 1);
        assert_eq!(mm.atom_positions.len(), 1);
        mm.view_mut(h1).unwrap().delete();
        assert_eq!(mm.all::<Atom>().count(), 0);
        // Note that stale position is still present, but can't be accessed
        assert!(mm.atom_positions.contains_key(h1.to_key()));
        assert!(mm.view(h1).is_none());
    }
}
