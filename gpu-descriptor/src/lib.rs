//!
//! gpu-descriptor crate.
//!

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications
)]

mod allocator;

pub use crate::allocator::*;
