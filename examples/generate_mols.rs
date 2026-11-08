// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Manual assembly of maps containing basic molecules, which are then serialized
//! for use as test fixtures.

#![allow(unused)]

use molmap::{atomic::*, entities::AnyAtomlike, *};

fn water() -> AtomMap {
    let mut m = AtomMap::new();
    let (o, _bonds) = m.add_atom_with_hydrogen(Element::O, 2);
    m
}

fn methanol() -> AtomMap {
    let mut m = AtomMap::new();
    let (c1, _bonds) = m.add_atom_with_hydrogen(Element::C, 3);
    let (o1, _bonds) = m.add_atom_with_hydrogen(Element::O, 1);
    let c1o1 = m.add_bond_unchecked(BondType::Covalent { order: 1.0 }, c1, o1);
    m
}

fn ethane() -> AtomMap {
    let mut m = AtomMap::new();
    let (c1, _bonds) = m.add_atom_with_hydrogen(Element::C, 3);
    let (c2, _bonds) = m.add_atom_with_hydrogen(Element::C, 3);
    let c1c2 = m.add_bond_unchecked(BondType::Covalent { order: 1.0 }, c1, c2);
    m
}

fn propane() -> AtomMap {
    let mut m = AtomMap::new();
    let (c1, _bonds) = m.add_atom_with_hydrogen(Element::C, 3);
    let (c2, _bonds) = m.add_atom_with_hydrogen(Element::C, 2);
    let (c3, _bonds) = m.add_atom_with_hydrogen(Element::C, 3);
    m.add_bond_unchecked(BondType::Covalent { order: 1.0 }, c1, c2);
    m.add_bond_unchecked(BondType::Covalent { order: 1.0 }, c2, c3);
    m
}

fn cyclohexane() -> AtomMap {
    let mut m = AtomMap::new();
    let mut carbons: Vec<AnyAtomlike> = Vec::new();
    for n in 0..6 {
        let (c, _bonds) = m.add_atom_with_hydrogen(Element::C, 2);
        if n != 0 {
            m.add_bond_unchecked(BondType::Covalent { order: 1.0 }, c, carbons[n - 1]);
        }
        carbons.push(c.into());
    }
    m.add_bond_unchecked(BondType::Covalent { order: 1.0 }, carbons[0], carbons[5]);
    m
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = ron::ser::PrettyConfig::new().depth_limit(2);
    std::fs::write(
        "tests/fixtures/atomic/water.ron",
        ron::ser::to_string_pretty(&water(), cfg.clone())?,
    )?;
    std::fs::write(
        "tests/fixtures/atomic/methanol.ron",
        ron::ser::to_string_pretty(&methanol(), cfg.clone())?,
    )?;
    std::fs::write(
        "tests/fixtures/atomic/ethane.ron",
        ron::ser::to_string_pretty(&ethane(), cfg.clone())?,
    )?;
    std::fs::write(
        "tests/fixtures/atomic/propane.ron",
        ron::ser::to_string_pretty(&propane(), cfg.clone())?,
    )?;
    std::fs::write(
        "tests/fixtures/atomic/cyclohexane.ron",
        ron::ser::to_string_pretty(&cyclohexane(), cfg.clone())?,
    )?;
    Ok(())
}
