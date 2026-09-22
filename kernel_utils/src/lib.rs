#![no_std]
#![no_builtins]

mod memcmp;
mod memcpy;
mod memmove;
mod memset;

pub use memmove::memmove;
