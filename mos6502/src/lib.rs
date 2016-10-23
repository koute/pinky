#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

#[macro_use]
extern crate emumisc;

#[macro_use]
extern crate bitflags;

mod virtual_mos6502_decoder;
mod virtual_mos6502;

pub use virtual_mos6502::{Interface, State, Context, Address, EmulationStatus, decode_instruction};
