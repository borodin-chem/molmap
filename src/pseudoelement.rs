// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::entities::definition::DefinitionId;

/// The "element" of a pseudoatom, represented in a chemical formula by a symbol
/// like a real element, but representing a group of atoms.
///
/// While there is a significant number of such [symbols in common use(https://en.wikipedia.org/wiki/Skeletal_formula#Pseudoelement_symbols),
/// and many are widely accepted and thus essentially unambiguous in what they
/// refer to, only the few listed explicitly in the
/// [2008 IUPAC Recommendations for chemical structure diagrams](https://doi.org/10.1351/pac200880020277)
/// are defined as named variants; others should be defined with custom definitions.
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
#[non_exhaustive]
#[allow(non_camel_case_types)]
pub enum Pseudoelement {
    Me,  // methyl
    Et,  // ethyl
    Pr,  // propyl
    iPr, // isopropyl
    Bu,  // butyl
    iBu, // isobutyl
    sBu, // sec-butyl
    tBu, // tert-butyl
    Ac,  // acetyl
    Ph,  // phenyl
    Ms,  // mesyl
    Ts,  // tosyl
    Cp,  // cyclopentadienyl
    Defined(DefinitionId),
}

impl Pseudoelement {
    /// Returns the pseudoelement's symbol as UTF-8, without formatting.
    pub fn symbol(&self) -> &str {
        match self {
            Pseudoelement::Me => "Me",
            Pseudoelement::Et => "Et",
            Pseudoelement::Pr => "Pr",
            Pseudoelement::iPr => "iPr",
            Pseudoelement::Bu => "Bu",
            Pseudoelement::iBu => "iBu",
            Pseudoelement::sBu => "s-Bu",
            Pseudoelement::tBu => "t-Bu",
            Pseudoelement::Ac => "Ac",
            Pseudoelement::Ph => "Ph",
            Pseudoelement::Ms => "Ms",
            Pseudoelement::Ts => "Ts",
            Pseudoelement::Cp => "Cp",
            Pseudoelement::Defined(definition_id) => todo!(),
        }
    }
}
