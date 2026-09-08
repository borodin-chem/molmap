// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{MolMap, error::MolMapError};

/// A parser between a chemical notation or file format and at least one kind of
/// `MolMap`.
///
/// Parsers may provide read or write functionality, or both, by implementing the
/// [`Reader`] and [`Writer`] traits.
///
/// Parsers may offer configuration options, which can be provided by the user at
/// construction in the form of a custom type. However, parsers must always make it
/// possible to obtain an instance with the default configuration by implementing
/// `Default`.
pub trait Parser: Default {
    /// The type used to (optionally) configure the parser.
    ///
    /// Implementors are not obliged to be configurable, but it is often useful to take
    /// user-specified options. If a parser cannot be configured, `Config` should just
    /// be `()`.
    type Config;

    /// The error type returned when parsing fails.
    ///
    /// Parsers may simply return [`MolMapError`] if they wish, or some custom error
    /// type if they wish to provide finer-grained diagnostics.
    type Error: Into<MolMapError>;

    /// Initializes a parser with the given configuration.
    fn with_config(config: Self::Config) -> Self;
}

/// A parser that reads a chemical notation or file format into a corresponding
/// `MolMap`.
///
/// Parsers are not obliged to parse all information contained in the input, but
/// should make their scope clear.
///
/// Parsers should avoid assuming, interpreting, or inferring information not
/// encoded by the format. Such processing can be subsequently applied by the
/// downstream user at their own discretion. This applies especially when multiple
/// interpretations or algorithms are possible.
///
/// For example, a parser that converts SMILES to a [`MolMap0`] makes sense, but one
/// that parses to a [`MolMap3`] does not, as the parser would have to apply some
/// algorithm for 3D structure generation from the molecular graph.
pub trait Read: Parser {
    type Output: MolMap;

    /// Parses the format from a `std::io` reader.
    ///
    /// For text formats the input should be valid UTF-8.
    fn read<R: std::io::Read>(&self, reader: R) -> Result<Self::Output, Self::Error>;
}

/// A parser that can produce multiple maps from a single input.
///
/// This is not meant for converting a stream of items to a stream of maps, but
/// rather for notations and formats from which several separate maps can reasonably
/// be extracted.
///
/// For example, a parser that just reads the optimized geometry from a quantum
/// chemical calculation output might implement [`Read`], but a parser that reads
/// the geometries from all steps of the calculation could implement [`MultiRead`].
///
/// See the main [`Read`] trait for implementation guidelines.
pub trait MultiRead: Parser {
    type Output: MolMap;
    type Iter: Iterator<Item = Self::Output>;

    /// Parses the format from a `std::io` reader.
    ///
    /// The maps are returned as an iterator over the output `MolMap` type.
    ///
    /// For text formats the input should be valid UTF-8.
    fn read<R: std::io::Read>(&self, reader: R) -> Result<Self::Iter, Self::Error>;
}

/// A parser that writes a chemical notation or file format based on a `MolMap`.
///
/// Parsers are not obliged to support all features of the `MolMap` type – and
/// indeed this may not even be possible for the output format in question – but
/// should make their scope clear.
///
/// Parsers should avoid assuming, interpreting, or inferring information that is
/// either required by or conventionally encoded by the format. Such processing
/// should be applied in advance by the downstream user at their own discretion.
/// This applies especially when multiple interpretations or algorithms are
/// possible.
///
/// For example, a parser that generates a SMILES string from any `MolMap` makes
/// sense, as all `MolMap` types hold the details of the molecular graph that are
/// necessary for SMILES generation, but one that writes an XYZ file from a
/// [`MolMap0`] does not, as the parser would have to apply some algorithm for 3D
/// structure generation from the molecular graph.
pub trait Write<M>: Parser
where
    M: MolMap,
{
    /// Parses the format to a `std::io` writer.
    fn write<W: std::io::Write>(&self, writer: W, map: M) -> Result<(), Self::Error>;
}

/// A parser that can write multiple maps to the same output.
///
/// This is not meant for producing a stream of items from a stream of maps, but
/// rather for notations and formats into which several separate maps can reasonably
/// be combined.
///
/// See the main [`Write`] trait for implementation guidelines.
pub trait MultiWrite<M, I>: Parser
where
    M: MolMap,
    I: Iterator<Item = M>,
{
    /// Parses the format to a `std::io` writer.
    ///
    /// The maps are supplied as an iterator over the output `MolMap` type.
    fn write<W: std::io::Write>(&self, writer: W, maps: I) -> Result<(), Self::Error>;
}
