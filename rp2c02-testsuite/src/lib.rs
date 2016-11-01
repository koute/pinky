use std::fmt;

// TODO: Remove this once rustc will be able to compile
// functions with many assertions in a reasonable time.
//
// See: https://github.com/rust-lang/rust/issues/37468
fn assert_eq< T: PartialEq + fmt::Debug >( a: T, b: T ) {
    assert_eq!( a, b );
}

pub trait TestPPU {
    fn expect_vram_read( &mut self, address: u16, value: u8 );
    fn expect_no_vram_read( &mut self );

    fn get_current_address( &self ) -> u16;
    fn get_temporary_address( &self ) -> u16;

    fn read_ioreg( &mut self, index: u8 ) -> u8;
    fn read_secondary_sprite_ram( &self, index: u8 ) -> u8;

    fn write_ioreg( &mut self, index: u8, value: u8 );
    fn write_palette_ram( &mut self, index: u8, value: u8 );
    fn write_sprite_ram( &mut self, index: u8, value: u8 );
    fn write_secondary_sprite_ram( &mut self, index: u8, value: u8 );
    fn write_vram( &mut self, address: u16, value: u8 );

    fn scanline( &self ) -> u16;
    fn dot( &self ) -> u16;

    fn step_pixel( &mut self );
    fn step_scanline( &mut self ) {
        let scanline = self.scanline();
        while self.scanline() == scanline {
            self.step_pixel();
        }
    }
}

#[macro_use]
#[doc(hidden)]
pub mod tests;
