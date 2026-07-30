# molmap

Arena-based molecular structure representation in Rust.

## What `molmap` is

`molmap` aims to be a foundational library for chemistry and cheminformatics programming in Rust.
It is a small, focused crate that is deliberately limited in scope.

`molmap` looks to provide:

- A small set of data structures ("molecular maps") for representing a chemical space in memory
- Robust management of the complexities of molecular relationships
- A performant `slotmap`-based data model under the hood that suits Rust's memory management
- An API that abstracts that away, in a way that feels idiomatic of a [Rust collection type](https://doc.rust-lang.org/std/collections/) while also reflecting how chemists think
- Rich chemical semantics and support for concepts like pseudoelements, repeating units, hapticity
- Graph functionality
- Geometric manipulation
- A graph-only map: `MolMap0`
- Maps with 2D or 3D spatial information: `MolMap2`, `MolMap3`
- Traits for common behaviour: `MolMap`, `SpatialMolMap`
- Parser traits for implementation by file format parsers
- A core building block – _but not all the building blocks_ – for other chemistry packages to build upon

## What `molmap` is _not_

- A full-blown cheminformatics toolkit
- An accurate representation of quantum-chemical reality
- A format for calculation results
- A format for experimental data
- A tool for molecular visualization
- A program for file format conversion
- [Everything but the kitchen sink](https://en.wiktionary.org/wiki/everything_but_the_kitchen_sink)

These are jobs for other crates built on top of `molmap`, not `molmap` itself.
The intention is to foster an ecosystem similar to those around `serde`, `num`, or `nalgebra`.

The list below will in future hopefully hold a whole swathe of crates that empower you to do these things with a `MolMap`, and more.

## The `molmap` kitchen sink

### Extension maps

The basic `MolMap` types only hold the molecular graph and, in the case of `MolMap2` and `MolMap3`, spatial positions.
Other crates will extend these to store more information; initially planned are:

- `molmap-drawing` – Extends `molmap` to describe 2D chemical graphics

### Parsers

The first few crates built around `molmap` are likely to be parsers for common file formats; initially planned are:

- `molmap-smiles` – Parsers for OpenSMILES and SMILES+ strings
- `molmap-xyz` – Parsers for XYZ and Extended XYZ files

## Status

The design of `molmap` is mostly settled but the crate is very much in an early state of development.
APIs are unstable and breaking changes should be expected.

## Contributing

Contributions via PR are welcome.

## License

The source code of `molmap` is subject to the terms of the Mozilla Public License, v. 2.0.
