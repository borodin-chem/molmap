// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Functionality common to all spatial maps.

use nalgebra as na;

use crate::{
    MolMapError, MolMapResult, Point, Spatial, Vector, entities::*, geometry, traits::Stores,
};

/// Methods common to all spatial maps.
pub(crate) trait SpatialCommon<const D: usize>: Spatial {
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
