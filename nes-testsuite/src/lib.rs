extern crate md5;

mod harness;

#[macro_use]
#[doc(hidden)]
pub mod tests;
pub use harness::{
    TestcaseState,
    EmulatorInterface,
    on_cycle,
    on_frame
};
