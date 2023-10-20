#![cfg_attr(not(feature = "std"), no_std)]

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

#[cfg(feature = "std")]
extern crate std as core;

#[macro_use]
extern crate emumisc;

#[macro_use]
extern crate bitflags;

mod virtual_mos6502_decoder;
mod virtual_mos6502;

pub use virtual_mos6502::{Interface, State, Context, Address, EmulationError, EmulationStatus, decode_instruction};
