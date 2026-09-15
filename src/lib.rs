// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Private modules
// ---------------
mod element;
mod molmap;
mod pseudoelement;

// ----------
// Public API
// ----------

// Publicly accessible modules
// ---------------------------
pub mod entities;
pub mod error;
pub mod graph;
pub mod parse;
pub mod spatial;
pub mod view;

// Top-level items
// ---------------
pub use element::Element;
pub use graph::MolMap0;
pub use graph::data::BondType;
pub use molmap::MolMap;
pub use pseudoelement::Pseudoelement;
pub use spatial::{MolMap2, MolMap3};

// Foreign re-exports
// ------------------
// Foreign crates or things from them
// Re-exporting nalgebra makes it easier for others to use
pub use nalgebra;
pub use nalgebra::{Point2, Point3, Vector2, Vector3};
