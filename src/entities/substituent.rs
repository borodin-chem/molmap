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
    ids::{AtomlikeId, BondId, FundamentalId},
    traits::MolMap,
};

new_key_type! {
    /// An ID corresponding to a specific substituent entity in a `MolMap`.
    pub struct SubstituentId;
}

#[derive(Debug, Clone)]
pub enum SubstituentCentre {
    None,
    Single(AtomlikeId),
    Multiple(Box<Vec<AtomlikeId>>),
}

/// The core data of a substituent entity.
///
/// Substituents are the smallest grouping in a MolMap
/// Substituents are conceptually equivalent to a non-hydrogen atom and "its" implicit
/// hydrogen atoms in SMILES or in packages that work that way,
/// or to the groups drawn together without explicit bonds in a skeletal formula
/// e.g. –OH, –COOH, –CH3
/// Substituents have an internal structure of Atoms, Pseudoatoms, and Bonds
/// Substituents generally indicate one or more centres to which bonds can be made,
/// but occasionally bonds are made to a substituent as a whole.
#[derive(Debug)]
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
#[derive(Clone, Copy)]
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

    pub fn centre(&self) -> &SubstituentCentre {
        &self.core().centre
    }

    pub fn members(&self) -> &[FundamentalId] {
        &self.core().members
    }

    /// Checks if the substituent contains the given atom, pseudoatom, or bond.
    pub fn contains(&self, fundamental: FundamentalId) -> bool {
        self.members().contains(&fundamental)
    }
}

/// A mutable view over a specific substituent entity in a specific `MolMap`.
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
            let bonds = match id {
                AtomlikeId::Atom(atom_id) => {
                    &self
                        .molmap
                        .core()
                        .atoms
                        .get(atom_id)
                        .expect("Wouldn't be listed as the centre if it had been removed")
                        .bonds
                }
                AtomlikeId::Pseudoatom(pseudoatom_id) => {
                    &self
                        .molmap
                        .core()
                        .pseudoatoms
                        .get(pseudoatom_id)
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
