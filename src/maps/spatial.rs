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

use crate::{
    categories::*,
    entities::*,
    error::{MolMapError, MolMapResult},
    graph::{MolGraph, entities::SubstituentCentre, keys::*},
    maps::MolMapCore,
    view::*,
    *,
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
    pub fn add_bond<A, B>(&mut self, start: A, end: B) -> MolMapResult<Bond>
    where
        A: Bondable,
        B: Bondable,
    {
        if !self.contains(start) {
            return Err(MolMapError::Id(start.as_entity()));
        } else if !self.contains(end) {
            return Err(MolMapError::Id(end.as_entity()));
        };
        Ok(self.core.add_bond(start, end))
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
            return Err(MolMapError::Id(fundamental.as_entity()));
        };
        if let Some(parent) = self.map.core().parent_substituent(fundamental) {
            if parent == self.id {
                return Ok(false);
            } else {
                self.map
                    .core_mut()
                    .remove_from_substituent(parent, fundamental);
            }
        }
        // All clear to add it to this substituent now
        // Note this will always return Ok(true)
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
                return Err(MolMapError::Id(f.as_entity()));
            }
        }
        for f in fundamentals {
            let f: AnyFundamental = f.into();
            if let Some(parent) = self.map.core().parent_substituent(f) {
                if parent == self.id {
                    // Already a member, so skip it
                    continue;
                } else {
                    self.map.core_mut().remove_from_substituent(parent, f);
                }
            }
            // All clear to add it to this substituent now
            self.map.core_mut().insert_into_substituent(self.id, f);
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
            return Err(MolMapError::Id(fundamental.as_entity()));
        };
        if let Some(parent) = self.map.core().parent_molecule(fundamental) {
            if parent == self.id {
                return Ok(false);
            } else {
                self.map
                    .core_mut()
                    .remove_from_molecule(parent, fundamental);
            }
        }
        // All clear to add it to this molecule now
        // Note this will always return Ok(true)
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
                return Err(MolMapError::Id(f.as_entity()));
            }
        }
        for f in fundamentals {
            let f: AnyFundamental = f.into();
            if let Some(parent) = self.map.core().parent_molecule(f) {
                if parent == self.id {
                    // Already a member, so skip it
                    continue;
                } else {
                    self.map.core_mut().remove_from_molecule(parent, f);
                }
            }
            // All clear to add it to this substituent now
            self.map.core_mut().insert_into_molecule(self.id, f);
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
        match self.core().data(bond).unwrap().start.resolve() {
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
        match self.core().data(bond).unwrap().end.resolve() {
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
            sum += pos.coords
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
    ///
    /// # Panics
    ///
    /// Panics if the substituent is not in the map.
    pub(crate) fn substituent_centroid(
        &self,
        substituent: Substituent,
        centres_only: bool,
    ) -> Option<Point<f64, D>> {
        let data = self
            .core
            .data(substituent)
            .expect("Caller is required to ensure that the ID is valid");
        if centres_only {
            match &data.centre {
                // Early return if substituent is empty/has no centre (should be the same thing)
                SubstituentCentre::None => None,
                // Early return if single centre (no need to take average)
                SubstituentCentre::Single(centre) => Some(self.atomlike_position(*centre)).copied(),
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
    ///
    /// # Panics
    ///
    /// Panics if the molecule is not in the map.
    pub(crate) fn molecule_member_positions(
        &self,
        molecule: Molecule,
    ) -> impl Iterator<Item = &Point<f64, D>> {
        let members = self
            .core
            .data(molecule)
            .expect("Caller is required to ensure that the ID is valid")
            .members
            .iter();
        let atomlikes = members.filter_map(|&x| AnyAtomlike::try_from(x).ok());
        atomlikes.map(|x| self.atomlike_position(x))
    }

    /// Calculates the unweighted geometric centre of the molecule,
    /// or `None` if the molecule is empty.
    ///
    /// Only the positions of the constituent atoms and pseudoatoms are taken into
    /// consideration, not the bonds, nor any other member fundamentals.
    ///
    /// # Panics
    ///
    /// Panics if the molecule is not in the map.
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

/// Methods for setting positions and applying transformations.
impl<const D: usize> SpatialMolMap<D> {
    /// Sets the position of the given atom to the provided value.
    ///
    /// This does not panic – if the atom is not in the map, nothing happens.
    pub(crate) fn set_atom_position(&mut self, atom: Atom, position: Point<f64, D>) {
        self.atom_positions.insert(atom.to_key(), position);
    }

    /// Sets the position of the given pseudoatom to the provided value.
    ///
    /// This does not panic – if the pseudoatom is not in the map, nothing happens.
    pub(crate) fn set_pseudoatom_position(
        &mut self,
        pseudoatom: Pseudoatom,
        position: Point<f64, D>,
    ) {
        self.pseudoatom_positions
            .insert(pseudoatom.to_key(), position);
    }
}

impl<'m, const D: usize> ViewMut<'m, SpatialMolMap<D>, Atom> {
    /// Sets the position of the atom to the provided value.
    pub fn set_position(self, position: Point<f64, D>) {
        self.map.set_atom_position(self.id, position)
    }
}

impl<'m, const D: usize> ViewMut<'m, SpatialMolMap<D>, Pseudoatom> {
    /// Sets the position of the pseudoatom to the provided value.
    pub fn set_position(self, position: Point<f64, D>) {
        self.map.set_pseudoatom_position(self.id, position)
    }
}
