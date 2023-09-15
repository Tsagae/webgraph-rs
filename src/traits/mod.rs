/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/*!


This modules contains the traits that are used throughout the crate.

*/

mod bvgraph_codes;
pub use bvgraph_codes::*;

pub(crate) mod graph;
pub use graph::*;

mod serde;
pub use serde::*;
