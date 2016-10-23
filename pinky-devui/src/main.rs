extern crate sdl2;
extern crate clock_ticks;
extern crate serde_json;
extern crate md5;

#[macro_use]
extern crate emumisc;

extern crate nes;

mod user_interface;
mod renderer;
mod frame_limiter;

fn main() {
    let mut ui = user_interface::create();
    ui.run();
}
