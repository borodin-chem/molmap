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

use molmap::{atomic::*, *};

fn meoh() -> AtomMap {
    let mut m = AtomMap::new();
    let h1 = m.add_atom(Element::H);
    let h2 = m.add_atom(Element::H);
    let h3 = m.add_atom(Element::H);
    let c1 = m.add_atom(Element::C);
    let c1h1 = m.add_bond_unchecked(BondType::Covalent { order: 1.0 }, c1, h1);
    let c1h2 = m.add_bond_unchecked(BondType::Covalent { order: 1.0 }, c1, h2);
    let c1h3 = m.add_bond_unchecked(BondType::Covalent { order: 1.0 }, c1, h3);
    let o1 = m.add_atom(Element::O);
    let h4 = m.add_atom(Element::H);
    let o1h4 = m.add_bond_unchecked(BondType::Covalent { order: 1.0 }, o1, h4);
    let c1o1 = m.add_bond_unchecked(BondType::Covalent { order: 1.0 }, c1, o1);
    m
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = ron::ser::PrettyConfig::new();
    std::fs::write(
        "tests/fixtures/meoh.ron",
        ron::ser::to_string_pretty(&meoh(), cfg)?,
    )?;
    Ok(())
}
