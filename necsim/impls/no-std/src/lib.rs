#![deny(clippy::pedantic)]
#![no_std]
#![feature(iter_advance_by)]
#![feature(drain_filter)]
#![feature(type_alias_impl_trait)]
#![feature(const_trait_impl)]
#![feature(const_type_name)]
#![feature(const_mut_refs)]
#![feature(const_refs_to_cell)]
#![feature(control_flow_enum)]
#![feature(negative_impls)]
#![feature(impl_trait_in_assoc_type)]
#![feature(associated_type_bounds)]
#![feature(const_float_bits_conv)]
#![feature(core_intrinsics)]
#![allow(incomplete_features)]
#![feature(specialization)]

extern crate alloc;

#[macro_use]
extern crate const_type_layout;

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate log;

pub mod alias;
pub mod array2d;
pub mod cache;
pub mod cogs;
pub mod decomposition;
pub mod parallelisation;
