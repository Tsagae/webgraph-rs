//! # `webgraph-rs`
//! A pure Rust implementation of the [WebGraph framework](https://webgraph.di.unimi.it/)
//! for graph compression.

// No warnings
#![deny(warnings)]

// the code must be safe and shouldn't ever panic to be relayable
#![deny(clippy::todo)]
#![deny(unsafe_code)]
#![deny(clippy::panic)]
#![deny(clippy::panicking_unwrap)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

// for now we don't need any new feature but we might remove this in the future
#![deny(unstable_features)]

// no dead code
#![deny(dead_code)]
#![deny(trivial_casts)]
#![deny(unconditional_recursion)]
#![deny(clippy::empty_loop)]
#![deny(unreachable_code)]
#![deny(unreachable_pub)]
#![deny(unreachable_patterns)]
#![deny(unused_macro_rules)]
//#![deny(unused_results)]

// the code must be documented and everything should have a debug print implementation
#![deny(unused_doc_comments)]
#![deny(missing_docs)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
#![deny(clippy::missing_safety_doc)]
//#![deny(clippy::missing_doc_code_examples)]
//#![deny(clippy::missing_crate_level_docs)]
//#![deny(clippy::missing_docs_in_private_items)]
//#![deny(missing_debug_implementations)]
