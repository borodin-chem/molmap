// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The data held for the entities in the core molecular graph and the view
//! methods to access it.
//!
//! # A note on crate organization
//!
//! By the very nature of the MolMap types, the data and functionality pertaining
//! to a particular kind of entity is implemented across many different types.
//! Some data is contained in the MolGraph, in the structs defined below, while
//! some (e.g. positional information) is held by higher map types.
//! Some of the internal functions that handle the entities are defined on
//! MolGraph, some in the MolMap trait, some on the higher maps such as MolMap0
//! or SpatialMolMap, some on the entities, and a very small number of them on
//! the data structs themselves.
//! Because Rust is compiled by crate, we are presented with two good options for
//! organizing the code.
//!
//! Firstly, it would actually be possible to collect *everything* related to a
//! single entity type together, e.g. put all code for atoms in a single `atom`
//! module, and do likewise for each of the entity types. This would put the
//! definition of the corresponding Entity type, the core data struct, any
//! additional data, the methods on the different maps, the methods on the
//! different views, etc. all in a single file. General things, such as the
//! Entity trait or the macro to define a new Kind, would remain at the top level.
//!
//! The other good option, and the one currently taken, is to organize the code
//! into modules by the thing being implemented e.g. the core data, spatial data,
//! methods on MolGraph, on MolMap, on SpatialMolMap, on the MolGraph views, on
//! the general MolMap views, on SpatialMolMap views, etc.
//! If desired this approach can be further split up e.g. different modules for
//! the different kinds of functionality of a SpatialMolMap.
//!
//! The first approach would make it easier to see and change the behaviour of a
//! single kind of entity all at once, and to have a better overview of what is
//! implemented for it at each level.
//! However, it makes parallel implementation of the same or similar things for
//! multiple kinds of entity much harder.
//! It also creates a lot of circular dependencies, which while not technically a
//! problem due to the crate-level compilation do still feel kind of bad.
//! The second option also fits better the way the crate is actually used - you
//! are more likely to want to work with (and thus import) the Entity types for
//! all kinds of entity at once, or all spatial stuff for all kinds at once.
//!
//! As such, this module contains the **core data structs** for *all* entity
//! kinds in a single file, as well as any **methods on completely generic views**
//! for access and manipulation **of the core data**.
//! As the maps are generally expected to want side-effects to occur when the
//! core data is changed, the methods implemented completely generically are
//! on the whole (currently, exclusively) confined to *access* via the immutable
//! views.
//!
//! It also currently contains many types necessary for the definitions of the
//! core structs e.g. [`BondType`] and [`SubstituentCentre`], but these may well
//! be moved in the future.

use std::collections::HashSet;

use crate::{Element, MolMap, Pseudoelement, categories::*, entities::*, view::*};

/// The core data of an atom entity.
#[derive(Clone, Debug)]
pub struct AtomData {
    pub(crate) element: Element,
    pub(crate) bonds: Vec<Bond>,
}

impl AtomData {
    pub(crate) fn new(element: Element) -> Self {
        Self {
            element,
            bonds: Vec::new(),
        }
    }
}

impl<'m, M: MolMap> View<'m, M, Atom> {
    pub fn element(&self) -> Element {
        self.data().element
    }

    pub fn symbol(&self) -> &str {
        self.data().element.symbol()
    }

    pub fn bonds(&self) -> &[Bond] {
        &self.data().bonds
    }
}

/// The core data of a pseudoatom entity.
#[derive(Clone, Debug)]
pub struct PseudoatomData {
    pub(crate) pseudoelement: Pseudoelement,
    pub(crate) bonds: Vec<Bond>,
}

impl PseudoatomData {
    pub(crate) fn new(pseudoelement: Pseudoelement) -> Self {
        Self {
            pseudoelement,
            bonds: Vec::new(),
        }
    }
}

impl<'m, M: MolMap> View<'m, M, Pseudoatom> {
    pub fn bonds(&self) -> &[Bond] {
        &self.data().bonds
    }
}

/// The type of a bond e.g. covalent, ionic.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BondType {
    Covalent,
    Intermolecular,
    Coordination,
    Ionic,
}

/// The core data of a bond entity.
#[derive(Clone, Debug)]
pub struct BondData {
    pub(crate) bond_type: BondType,
    pub(crate) order: f32,
    pub(crate) start: AnyBondable,
    pub(crate) end: AnyBondable,
}

impl BondData {
    pub(crate) fn new(
        bond_type: BondType,
        order: f32,
        start: AnyBondable,
        end: AnyBondable,
    ) -> Self {
        Self {
            bond_type,
            order,
            start,
            end,
        }
    }
}

impl<'m, M: MolMap> View<'m, M, Bond> {
    pub fn bond_type(&self) -> BondType {
        self.data().bond_type
    }

    pub fn order(&self) -> f32 {
        self.data().order
    }

    pub fn partners(&self) -> [AnyBondable; 2] {
        let inner = self.data();
        [inner.start, inner.end]
    }
}

#[derive(Clone, Debug)]
pub enum SubstituentCentre {
    None,
    Single(AnyAtomlike),
    Multiple(Box<Vec<AnyAtomlike>>),
}

/// The core data of a substituent entity.
#[derive(Clone, Debug)]
pub struct SubstituentData {
    pub(crate) centre: SubstituentCentre,
    pub(crate) members: Vec<AnyFundamental>,
}

impl SubstituentData {
    pub(crate) fn new(centre: AnyAtomlike, members: &[AnyFundamental]) -> Self {
        Self {
            centre: SubstituentCentre::Single(centre),
            members: members.to_vec(),
        }
    }
}

impl<'m, M: MolMap> View<'m, M, Substituent> {
    /// Returns details of the centre(s) of the substituent.
    pub fn centre(&self) -> &SubstituentCentre {
        &self.data().centre
    }

    /// Returns an iterator over the IDs of all constituent atoms, pseudoatoms, and bonds.
    pub fn members(&self) -> impl Iterator<Item = AnyFundamental> {
        self.data().members.iter().copied()
    }

    /// Checks if the substituent contains the given atom, pseudoatom, or bond.
    pub fn contains(&self, fundamental: impl Fundamental) -> bool {
        self.data().members.contains(&fundamental.as_fundamental())
    }
}

//impl<'m, M: MolMap> ViewMut<'m, M, Substituent> {
///// Attempts to change the centre of the substituent to the one requested.
/////
///// # Errors
/////
///// Fails if the requested centre is not already a member of the substituent,
///// or if there are already bonds to the current centre(s).
//pub fn change_centre(mut self, new: Atomlike>) -> MolMapResult<() {
//    // First confirm that `new` is actually a member of `self`
//    self.core()
//        .members
//        .contains(&new.into())
//        .then_some(())
//        .ok_or(MolMapError::Membership(new.into()))?;
//    // A closure that determines if an atom or pseudoatom has bonds already
//    let atomlike_has_bonds = |id: Atomlike>| - bool {
//        let bonds = match id.to_tagged() {
//            ResolvedAtomlike::Atom(id) => {
//                &self
//                    .map
//                    .core()
//                    .atoms
//                    .get(id.try_into().unwrap())
//                    .expect("Wouldn't be listed as the centre if it had been removed")
//                    .bonds
//            }
//            ResolvedAtomlike::Pseudoatom(id) => {
//                &self
//                    .map
//                    .core()
//                    .pseudoatoms
//                    .get(id)
//                    .expect("Wouldn't be listed as the centre if it had been removed")
//                    .bonds
//            }
//        };
//        !bonds.is_empty()
//    };
//    // Check that there aren't already bonds to the current centre
//    let already_bonded = match self.as_view().centre().clone() {
//        SubstituentCentre::None => false,
//        SubstituentCentre::Single(atomlike_id) => atomlike_has_bonds(atomlike_id),
//        SubstituentCentre::Multiple(atomlike_ids) => {
//            atomlike_ids.into_iter().any(atomlike_has_bonds)
//        }
//    };
//    if already_bonded {
//        Err(MolMapError::Disallowed(String::from(
//            "Substituent already has at least one bond to its centre(s)",
//        )))
//    } else {
//        self.core().centre = SubstituentCentre::Single(new.into());
//        Ok(())
//    }
//}
//}

/// The core data of a molecule entity.
#[derive(Clone, Debug)]
pub struct MoleculeData {
    pub(crate) members: HashSet<AnyFundamental>,
}

impl MoleculeData {
    pub(crate) fn new() -> Self {
        Self {
            members: HashSet::new(),
        }
    }
}

impl<'m, M: MolMap> View<'m, M, Molecule> {
    /// Returns an iterator over the IDs of all constituent atoms, pseudoatoms, and bonds.
    pub fn members(&self) -> impl Iterator<Item = AnyFundamental> {
        self.data().members.iter().copied()
    }

    /// Checks if the molecule contains the given atom, pseudoatom, or bond.
    pub fn contains(&self, fundamental: impl Fundamental) -> bool {
        self.data().members.contains(&fundamental.as_fundamental())
    }
}
