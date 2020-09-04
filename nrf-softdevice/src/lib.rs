#![no_std]
#![feature(asm)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
#![feature(const_fn)]

pub(crate) mod util;

pub mod interrupt;

mod events;
pub use events::*;
mod flash;
pub use flash::*;
