// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use nalgebra as na;

/// A point in `D`-dimensional space.
pub type Point<const D: usize> = na::Point<f64, D>;

/// A point in two-dimensional space.
pub type Point2 = Point<2>;

/// A point in three-dimensional space.
pub type Point3 = Point<3>;

/// A vector in `D`-dimensional space.
///
/// This is an alias for the `nalgebra::Vector` type that specifies the scalar and
/// storage types, and takes a `const` dimension rather a type, making it more
/// convenient.
///
/// `Vector<2>` using this alias is just shorthand for `nalgebra::Vector2<f64>`,
/// and `Vector<3>` is `nalgebra::Vector3<f64>`.
pub type Vector<const D: usize> = na::Vector<f64, na::Const<D>, na::ArrayStorage<f64, D, 1>>;

/// A vector in two-dimensional space.
pub type Vector2 = Vector<2>;

/// A vector in three-dimensional space.
pub type Vector3 = Vector<3>;

/// Calculates the mean position from a set of positions, or `None` if the iterator is empty.
pub(crate) fn mean_point<'a, I, const D: usize>(positions: I) -> Option<Point<D>>
where
    I: IntoIterator<Item = &'a Point<D>>,
{
    let mut count: u32 = 0;
    let mut sum: na::SVector<f64, D> = na::SVector::zeros();
    for pos in positions {
        count += 1;
        sum += pos.coords
    }
    if count == 0 {
        None
    } else {
        let avg = sum / f64::from(count);
        Some(Point::from(avg))
    }
}
