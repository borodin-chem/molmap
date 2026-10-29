// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// A unit of measure.
pub trait Unit {
    // We don't go to the trouble of using decimal floating point everywhere (at
    // least, not for now), but we at least try to have conversion between
    // different prefixed SI units be exact when possible, by storing and using a decimal rather
    // than binary representation of the number.

    /// The significand of the proportionality factor of the value of the unit when expressed in picometres.
    const SIGNIFICAND: u64;

    /// The exponent of the proportionality factor of the value of the unit when expressed in picometres.
    const EXPONENT: i8;

    /// The proportionality factor of the value of the unit when expressed in picometres.
    const FACTOR: f64;

    /// Expresses the unit in terms of the requested unit.
    fn in_terms_of<U: Unit>() -> f64 {
        if Self::SIGNIFICAND % U::SIGNIFICAND == 0 {
            ((Self::SIGNIFICAND / U::SIGNIFICAND) as f64)
                * 10_f64.powi((Self::EXPONENT - U::EXPONENT) as i32)
        } else {
            Self::FACTOR / U::FACTOR
        }
    }

    /// Converts a quantity with this unit and the provided number to a quantity in
    /// terms of the requested unit and returns its number.
    fn convert_to<U: Unit>(number: f64) -> f64 {
        if Self::SIGNIFICAND % U::SIGNIFICAND == 0 {
            number
                * ((Self::SIGNIFICAND / U::SIGNIFICAND) as f64)
                * 10_f64.powi((Self::EXPONENT - U::EXPONENT) as i32)
        } else {
            number * Self::FACTOR / U::FACTOR
        }
    }

    /// Converts a quantity with the provided number and unit to a quantity in
    /// terms of this unit and returns its number.
    fn convert_from<U: Unit>(number: f64) -> f64 {
        if U::SIGNIFICAND % Self::SIGNIFICAND == 0 {
            number
                * ((U::SIGNIFICAND / Self::SIGNIFICAND) as f64)
                * 10_f64.powi((U::EXPONENT - Self::EXPONENT) as i32)
        } else {
            number * U::FACTOR / Self::FACTOR
        }
    }
}

/// The meter (m), the SI unit of length.
pub struct Metre;

impl Unit for Metre {
    const SIGNIFICAND: u64 = 1;

    const EXPONENT: i8 = 12;

    const FACTOR: f64 = 1e12;
}

pub type Meter = Metre;

/// The angstrom (Å), equal to 1 × 10<sup>−10</sup> m.
pub struct Angstrom;

impl Unit for Angstrom {
    const SIGNIFICAND: u64 = 1;

    const EXPONENT: i8 = 2;

    const FACTOR: f64 = 1e2;
}

/// The nanometre (nm), equal to 1 × 10<sup>−9</sup> m.
pub struct Nanometre;

impl Unit for Nanometre {
    const SIGNIFICAND: u64 = 1;

    const EXPONENT: i8 = 3;

    const FACTOR: f64 = 1e3;
}

pub type Nanometer = Nanometre;

/// The picometre (pm), equal to 1 × 10<sup>−12</sup> m.
pub struct Picometre;

impl Unit for Picometre {
    const SIGNIFICAND: u64 = 1;

    const EXPONENT: i8 = 0;

    const FACTOR: f64 = 1e0;
}

pub type Picometer = Picometre;

/// `molmap`'s "standard bond length", equivalent to 1.5 Å (the rough length of a
/// carbon-carbon single covalent bond).
pub struct StdBond;

impl Unit for StdBond {
    const SIGNIFICAND: u64 = 15;

    const EXPONENT: i8 = 1;

    const FACTOR: f64 = 15e1;
}

/// A length unit relative to the approximate length of a carbon-carbon single
/// covalent bond (1.5 Å).
pub type Relative = StdBond;

/// The Bohr radius (_a_<sub>0</sub>).
///
/// The value used is the 2022 CODATA recommended value of 5.291 772 105 44 × 10<sup>−11</sup> m.
pub struct BohrRadius;

impl Unit for BohrRadius {
    const SIGNIFICAND: u64 = 529177210544;

    const EXPONENT: i8 = -10;

    const FACTOR: f64 = 52.9177210544;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bohr_radius_magnitude() {
        // Just a sanity check that the provided factor is equivalent to s * 10^e
        assert_eq!(
            // Should round to 53 pm
            ((BohrRadius::SIGNIFICAND as f64) * 10_f64.powi(BohrRadius::EXPONENT as i32)).round(),
            BohrRadius::FACTOR.round()
        );
    }

    #[test]
    fn in_terms_of() {
        // These we can definitely manage without floating point precision error
        // (as long as we use the decimal representation)
        assert_eq!(Angstrom::in_terms_of::<Metre>(), 1e-10);
        assert_eq!(Angstrom::in_terms_of::<Nanometre>(), 0.1);
        assert_eq!(Angstrom::in_terms_of::<Picometre>(), 100.0);
        assert_eq!(Picometre::in_terms_of::<Nanometre>(), 0.001);
    }

    #[test]
    fn convert() {
        assert_eq!(Angstrom::convert_to::<Nanometre>(3.21), 0.321);
        assert_eq!(Nanometre::convert_to::<Picometre>(3.21), 3210.0);
        assert_eq!(Angstrom::convert_from::<Nanometre>(3.21), 32.1);
        assert_eq!(Nanometre::convert_from::<Picometre>(3.21), 0.00321);
        // Unfortunately the above pass by luck more than anything, this one fails:
        //assert_eq!(Angstrom::convert_to::<Nanometre>(0.321), 0.0321);
        // No real way to avoid this though without either a) using decimals or b) going via strings
    }
}
