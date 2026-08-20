// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashSet;

use crate::*;

/// A molecule: a discrete group of atoms held together by chemical bonds.
///
/// > An electrically neutral entity consisting of more than one atom (_n_ > 1).
/// > Rigorously, a molecule, in which n > 1 must correspond to a depression on the
/// > potential energy surface that is deep enough to confine at least one
/// > vibrational state.
/// >
/// > [_'molecule' in IUPAC Compendium of Chemical Terminology, 5th ed. International Union of Pure and Applied Chemistry; 2025._](https://doi.org/10.1351/goldbook.M04002)
///
/// This definition from the IUPAC Gold Book restricts the meaning of "molecule" to
/// electrically neutral species, but here, the typical practice is followed and no
/// distinction is made based on charge.
///
/// Note that the constituent atoms of a molecule are not actually required to be
/// joined by bonds, and it is also not required that all bonds are covalent. The
/// molecule need not have any bonds at all, or indeed any atoms (an empty molecule
/// is also permitted). Do not rely on any of these things being true.
#[derive(Copy, Clone, Debug)]
pub struct Molecule;

impl Entity for Molecule {}

impl KeyEntity for Molecule {
    fn kind() -> EntityKind {
        EntityKind::Molecule
    }
}

/// The core data of a molecule entity.
#[derive(Clone, Debug)]
pub(crate) struct MoleculeData {
    pub(crate) members: HashSet<Id<Fundamental>>,
}

impl MoleculeData {
    pub fn new() -> Self {
        Self {
            members: HashSet::new(),
        }
    }
}

impl<'a, M: MolMap> View<'a, M, Molecule> {
    fn core(&self) -> &MoleculeData {
        self.map.core().molecules.get(self.id).unwrap()
    }

    /// Returns an iterator over the IDs of all constituent atoms, pseudoatoms, and bonds.
    pub fn members(&self) -> impl Iterator<Item = Id<Fundamental>> {
        self.core().members.iter().copied()
    }

    /// Checks if the molecule contains the given atom, pseudoatom, or bond.
    pub fn contains(&self, fundamental: Id<Fundamental>) -> bool {
        self.core().members.contains(&fundamental)
    }
}
