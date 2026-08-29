// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use nalgebra::{ArrayStorage, Const, Point};
use slotmap::SecondaryMap;

use crate::{MolMap, categories::AnyAtomlike, entities::*, graph::keys::*};

// The nalgebra vector type leaves the storage type generic
// This is a more convenient alias
/// A matrix with one column and D rows.
pub type Vector<T, const D: usize> = nalgebra::Vector<T, Const<D>, ArrayStorage<T, D, 1>>;

/// A container for the positions of the fundamental entities in a [`MolMap`].
#[derive(Clone, Debug)]
pub struct Positions<const D: usize> {
    atoms: SecondaryMap<AtomKey, Point<f64, D>>,
    pseudoatoms: SecondaryMap<PseudoatomKey, Point<f64, D>>,
    /// Bond positions are stored as a tuple of an origin and a vector.
    bonds: SecondaryMap<BondKey, (Point<f64, D>, Vector<f64, D>)>,
}

impl<const D: usize> Positions<D> {
    pub(crate) fn new() -> Self {
        Self {
            atoms: SecondaryMap::new(),
            pseudoatoms: SecondaryMap::new(),
            bonds: SecondaryMap::new(),
        }
    }

    pub(crate) fn with_capacities(atoms: usize, pseudoatoms: usize, bonds: usize) -> Self {
        Self {
            atoms: SecondaryMap::with_capacity(atoms),
            pseudoatoms: SecondaryMap::with_capacity(pseudoatoms),
            bonds: SecondaryMap::with_capacity(bonds),
        }
    }
}

/// A trait implemented by all `SpatialMolMap` types to provide access to the
/// position data in a generic fashion without exposing a public interface to it.
///
/// The trait has visibility `pub` to match `SpatialMolMap`, but it should not be
/// exposed publicly through re-export.
pub trait SpatialMolMapCore<const D: usize> {
    fn positions(&self) -> &Positions<D>;

    fn positions_mut(&mut self) -> &mut Positions<D>;
}

/// A [`MolMap`] that also holds the spatial positions (with dimensionality `D`)
/// of its entities.
pub trait SpatialMolMap<const D: usize>: MolMap + SpatialMolMapCore<D> {}
