// feature has been removed science 1.100.0
// #![feature(box_patterns)]

#![feature(deref_patterns)]


#[cfg(feature = "binary-ir")]
pub mod binary;

pub mod ssa;
pub mod tree;

pub mod compiler;
pub mod linker;
pub mod target;
pub mod types;

pub mod style;

use xxhash_rust::xxh3::Xxh3;
pub type DefaultHasher = Xxh3;
