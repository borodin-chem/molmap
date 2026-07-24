// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use slotmap::new_key_type;

use crate::{
    MolMapError, MolMapResult,
    ids::{AtomlikeId, BondId, FundamentalId, Id, SubstituentId, TaggedAtomlike, TaggedEntity},
    traits::MolMap,
};

#[derive(Clone, Debug)]
pub enum SubstituentCentre {
    None,
    Single(AtomlikeId),
    Multiple(Box<Vec<AtomlikeId>>),
}

/// The core data of a substituent entity.
///
/// Substituents are the smallest collection in a `MolMap` and represent the units
/// that chemists tend to actually think, rather than individual atoms. For example,
/// a substituent is conceptually equivalent to:
/// - a non-hydrogen atom and "its" implicit hydrogen atoms in SMILES or in packages
/// that work that way (all hydrogen atoms are explicit in a MolMap)
/// - the carbon atom and hydrogen atoms at a vertex in a skeletal formula
/// - atoms drawn together as a group without explicit bonds in a skeletal formula
///   e.g. –OH, –COOH, –CH₃
///
/// Substituents generally indicate one or more centres, so that bonds can be made
/// "to" the centre. This allows molecules to be built up conveniently by adding and
/// connecting substituents rather than individual atoms.
#[derive(Clone, Debug)]
pub(crate) struct Substituent {
    pub(crate) centre: SubstituentCentre,
    pub(crate) members: Vec<FundamentalId>,
}

impl Substituent {
    pub(crate) fn new(centre: AtomlikeId, members: &[FundamentalId]) -> Self {
        Self {
            centre: SubstituentCentre::Single(centre),
            members: members.to_vec(),
        }
    }
}

/// An immutable view over a specific substituent entity in a specific `MolMap`.
#[derive(Copy, Clone, Debug)]
pub struct SubstituentView<'a, M: MolMap> {
    pub molmap: &'a M,
    pub id: SubstituentId,
}

impl<'a, M: MolMap> From<SubstituentView<'a, M>> for SubstituentId {
    fn from(view: SubstituentView<'a, M>) -> Self {
        view.id
    }
}

impl<'a, M: MolMap> SubstituentView<'a, M> {
    /// Returns the corresponding data from the core `MolGraph`.
    fn core(&self) -> &'a Substituent {
        self.molmap.core().substituents.get(self.id).unwrap()
    }

    /// Returns details of the centre(s) of the substituent.
    pub fn centre(&self) -> &SubstituentCentre {
        &self.core().centre
    }

    /// Returns an iterator over the IDs of all constituent atoms, pseudoatoms, and bonds.
    pub fn members(&self) -> impl Iterator<Item = FundamentalId> {
        self.core().members.iter().copied()
    }

    /// Checks if the substituent contains the given atom, pseudoatom, or bond.
    pub fn contains(&self, fundamental: FundamentalId) -> bool {
        self.core().members.contains(&fundamental)
    }
}

/// A mutable view over a specific substituent entity in a specific `MolMap`.
#[derive(Debug)]
pub struct SubstituentViewMut<'a, M: MolMap> {
    pub molmap: &'a mut M,
    pub id: SubstituentId,
}

impl<'a, M: MolMap> From<SubstituentViewMut<'a, M>> for SubstituentId {
    fn from(view: SubstituentViewMut<'a, M>) -> Self {
        view.id
    }
}

impl<'a, M: MolMap> SubstituentViewMut<'a, M> {
    /// Returns the corresponding data from the core `MolGraph`.
    fn core(&mut self) -> &mut Substituent {
        self.molmap
            .core_mut()
            .substituents
            .get_mut(self.id)
            .unwrap()
    }

    /// Returns an immutable view over the same substituent.
    fn as_ref(&self) -> SubstituentView<'_, M> {
        SubstituentView {
            molmap: &*self.molmap,
            id: self.id,
        }
    }

    // Public methods, which should consume the view

    /// Attempts to change the centre of the substituent to the one requested.
    ///
    /// Fails if the requested centre is not already a member of the substituent,
    /// or if there are already bonds to the current centre(s).
    pub fn change_centre(mut self, new: AtomlikeId) -> MolMapResult<()> {
        // First confirm that `new` is actually a member of `self`
        self.core()
            .members
            .contains(&new.into())
            .then_some(())
            .ok_or(MolMapError::Membership(new.into()))?;
        // A closure that determines if an atom or pseudoatom has bonds already
        let atomlike_has_bonds = |id: AtomlikeId| -> bool {
            let bonds = match id.to_tagged() {
                TaggedAtomlike::Atom(id) => {
                    &self
                        .molmap
                        .core()
                        .atoms
                        .get(id.try_into().unwrap())
                        .expect("Wouldn't be listed as the centre if it had been removed")
                        .bonds
                }
                TaggedAtomlike::Pseudoatom(id) => {
                    &self
                        .molmap
                        .core()
                        .pseudoatoms
                        .get(id)
                        .expect("Wouldn't be listed as the centre if it had been removed")
                        .bonds
                }
            };
            !bonds.is_empty()
        };
        // Check that there aren't already bonds to the current centre
        let already_bonded = match self.as_ref().centre().clone() {
            SubstituentCentre::None => false,
            SubstituentCentre::Single(atomlike_id) => atomlike_has_bonds(atomlike_id),
            SubstituentCentre::Multiple(atomlike_ids) => {
                atomlike_ids.into_iter().any(atomlike_has_bonds)
            }
        };
        if already_bonded {
            Err(MolMapError::Disallowed(String::from(
                "Substituent already has at least one bond to its centre(s)",
            )))
        } else {
            self.core().centre = SubstituentCentre::Single(new.into());
            Ok(())
        }
    }

    /// Removes the substituent from the map, as well as all of its members.
    pub fn delete(mut self) {
        self.molmap.core_mut().delete_substituent(self.id);
    }
}
