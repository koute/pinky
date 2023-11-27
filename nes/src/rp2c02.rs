use core::mem;
use core::default::Default;
use core::slice;

use emumisc::{WrappingExtra, BitExtra, HiLoAccess, PeekPoke, At, is_b5_set, is_b6_set, is_b7_set, reverse_bits};

pub trait Context: Sized {
    fn state_mut( &mut self ) -> &mut State;
    fn state( &self ) -> &State;

    fn on_cycle( &mut self ) {}
    fn on_frame_was_generated( &mut self );
    fn set_vblank_nmi( &mut self, value: bool );
    fn peek_video_memory( &self, offset: u16 ) -> u8;
    fn poke_video_memory( &mut self, offset: u16, value: u8 );
}

pub trait Interface: Sized + Context {
    #[inline]
    fn framebuffer( &mut self ) -> &Framebuffer {
        Private::framebuffer( self )
    }

    #[inline]
    fn swap_framebuffer( &mut self, other: Framebuffer ) -> Framebuffer {
        Private::swap_framebuffer( self, other )
    }

    #[inline]
    fn execute( &mut self ) {
        Private::execute( self )
    }

    #[inline]
    fn peek_ppustatus( &mut self ) -> u8 {
        Private::peek_ppustatus( self )
    }

    #[inline]
    fn poke_ppuctrl( &mut self, value: u8 ) {
        Private::poke_ppuctrl( self, value )
    }

    #[inline]
    fn poke_ppumask( &mut self, value: u8 ) {
        Private::poke_ppumask( self, value )
    }

    #[inline]
    fn poke_ppuaddr( &mut self, value: u8 ) {
        Private::poke_ppuaddr( self, value )
    }

    #[inline]
    fn poke_ppuscroll( &mut self, value: u8 ) {
        Private::poke_ppuscroll( self, value )
    }

    #[inline]
    fn poke_ppudata( &mut self, value: u8 ) {
        Private::poke_ppudata( self, value )
    }

    #[inline]
    fn peek_ppudata( &mut self ) -> u8 {
        Private::peek_ppudata( self )
    }

    #[inline]
    fn poke_oamaddr( &mut self, value: u8 ) {
        Private::poke_oamaddr( self, value )
    }

    #[inline]
    fn poke_oamdata( &mut self, value: u8 ) {
        Private::poke_oamdata( self, value )
    }

    #[inline]
    fn peek_oamdata( &mut self ) -> u8 {
        Private::peek_oamdata( self )
    }

    #[inline]
    fn poke_sprite_list_ram( &mut self, index: u8, value: u8 ) {
        Private::poke_sprite_list_ram( self, index, value )
    }
}

impl< T: Context > Interface for T {}
impl< T: Context > Private for T {}

pub struct State {
    palette_ram: [u8; 32],
    sprite_list_ram: [u8; 256],
    secondary_sprite_list_ram: [u8; 32],
    framebuffer: Option< Framebuffer >,
    odd_frame_flag: bool,
    vblank_flag_was_cleared: bool,

    // Internal registers.
    current_address: u16,	// 15-bit
    temporary_address: u16,	// 15-bit
    fine_horizontal_scroll: u8,	//  3-bit
    write_toggle: bool,	//  1-bit

    n_pixel: u16,
    n_scanline: u16,
    n_dot: u16,
    bg_pattern_lo_shift_register: u16,
    bg_pattern_hi_shift_register: u16,
    bg_palette_index_lo_shift_register: u16,
    bg_palette_index_hi_shift_register: u16,

    sprites: [Sprite; 8],

    sprite_list_address: u8,
    ppudata_read_buffer: u8,

    // I/O registers.
    ppuctrl: PpuCtrl,
    ppumask: PpuMask,
    ppustatus: PpuStatus,

    residual_data: u8,

    address: u16,
    skip_cycle_flag: bool,
    skip_next_cycle: bool,
    odd_cycle_flag: bool,
    background_pattern_index_latch: u8,
    background_palette_index_latch: u8,
    tile_lo_latch: u8,
    tile_hi_latch: u8,
    scanline_counter: u16,
    scanline_index: u8,
    chunk_counter: u16,
    chunk_index: u8,
    action_index: u16,
    auxiliary_sprite_list_address: u8,
    secondary_sprite_list_address: u8,
    sprite_list_data_latch: u8,
    sprite_evaluation_mode: SpriteEvaluationMode,
    sprite_pattern_index_latch: u8,
    sprite_vertical_position_latch: u8,
    sprite_attributes_latch: u8,
    sprite_index: u8,
    first_sprite_is_sprite_zero_on_current_scanline: bool,
    first_sprite_is_sprite_zero_on_next_scanline: bool
}

impl State {
    pub const fn new() -> State {
        let last_scanline = match SCANLINES_CONST.last() {
            Some(value) => value,
            None => unreachable!()
        };

        State {
            palette_ram: [0; 32],
            sprite_list_ram: [0; 64 * 4],
            secondary_sprite_list_ram: [0;  8 * 4],
            framebuffer: None,
            odd_frame_flag: false,
            vblank_flag_was_cleared: false,

            current_address: 0,
            temporary_address: 0,
            fine_horizontal_scroll: 0,
            write_toggle: false,

            n_pixel: 0,
            n_scanline: 261, // We start on the prerender scanline.
            n_dot: 0,
            bg_pattern_lo_shift_register: 0,
            bg_pattern_hi_shift_register: 0,
            bg_palette_index_lo_shift_register: 0,
            bg_palette_index_hi_shift_register: 0,

            sprites: [Sprite::new(); 8],

            sprite_list_address: 0,
            ppudata_read_buffer: 0,

            ppuctrl: PpuCtrl::new(),
            ppumask: PpuMask::new(),
            ppustatus: PpuStatus::new(),

            residual_data: 0,

            address: 0,
            skip_cycle_flag: false,
            skip_next_cycle: false,
            odd_cycle_flag: false,
            background_pattern_index_latch: 0,
            background_palette_index_latch: 0,
            tile_lo_latch: 0,
            tile_hi_latch: 0,
            scanline_index: SCANLINES_CONST.len() as u8 - 1, // Point at the prerender scanline.
            scanline_counter: 0,
            chunk_index: last_scanline.first_chunk_index,
            chunk_counter: 0,
            action_index: CHUNKS_CONST[ last_scanline.first_chunk_index as usize ].first_action_index,
            auxiliary_sprite_list_address: 0,
            secondary_sprite_list_address: 0,
            sprite_list_data_latch: 0,
            sprite_evaluation_mode: SpriteEvaluationMode::Search,
            sprite_pattern_index_latch: 0,
            sprite_vertical_position_latch: 0,
            sprite_attributes_latch: 0,
            sprite_index: 0,
            first_sprite_is_sprite_zero_on_next_scanline: false,
            first_sprite_is_sprite_zero_on_current_scanline: false
        }
    }

    fn initialize_framebuffer_if_needed( &mut self ) {
    }

    #[inline]
    pub fn framebuffer( &mut self ) -> &mut Framebuffer {
        self.framebuffer.get_or_insert_with( Framebuffer::default )
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct PpuCtrl(u8);

impl PpuCtrl {
    const fn new() -> PpuCtrl {
        PpuCtrl(0)
    }

    /* Bits 0 and 1 determine the base background tilemap address. */

    bit!( 2, increment_vram_address_by_32 );			/* Whenever the internal address register will get incremented by
                                                           1 or by 32 after a PPUDATA read/write. */
    bit!( 3, use_second_pattern_table_for_sprites );
    bit!( 4, use_second_pattern_table_for_background );
    bit!( 5, big_sprite_mode );							/* Whenever we use 8x8 sprites (== 0) or 8x16 sprites (== 16). */
    bit!( 6, slave_mode );								/* This bit is useless; setting it to 1 can even damage an unmodified NES. */
    bit!( 7, should_generate_vblank_nmi );

    fn base_background_tilemap_address( self ) -> u16 {
        match self.0 & 0b11 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => unsafe { fast_unreachable!() }
        }
    }

    fn vram_address_increment( self ) -> u16 {
        if self.increment_vram_address_by_32() == false {
            1
        } else {
            32
        }
    }

    fn sprite_pattern_table_address( self ) -> u16 {
        if self.use_second_pattern_table_for_sprites() == false {
            0
        } else {
            0x1000
        }
    }

    fn background_pattern_table_address( self ) -> u16 {
        if self.use_second_pattern_table_for_background() == false {
            0
        } else {
            0x1000
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct PpuMask(u8);

impl PpuMask {
    const fn new() -> PpuMask {
        PpuMask(0)
    }

    bit!( 0, grayscale_mode );
    bit!( 1, show_background_in_leftmost_8_pixels );
    bit!( 2, show_sprites_in_leftmost_8_pixels );
    bit!( 3, show_background );
    bit!( 4, show_sprites );
    bit!( 5, emphasize_red );
    bit!( 6, emphasize_green );
    bit!( 7, emphasize_blue );

    #[inline]
    fn color_emphasize_bits( &self ) -> u8 {
        self.0.get_bits( 0b11100000 )
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct PpuStatus(u8);

impl PpuStatus {
    const fn new() -> PpuStatus {
        PpuStatus(0)
    }

    bit!( 5, sprite_overflow ); /* Set when more than 8 sprites appear on a scanline. */
    bit!( 6, sprite_0_hit ); 	/* Set when a nonzero pixel of sprite 0 overlaps a nonzero background pixel. */
    bit_setter!( 6, modify_sprite_0_hit );
    bit!( 7, vblank_has_occured );
    bit_setter!( 7, modify_vblank_has_occured );
}

/*
    See Wikipedia[1] for an introduction.

    [1] -- https://en.wikipedia.org/wiki/Picture_Processing_Unit

    The NES has a system palette of 64 colors; the PPU stores 2 sets of palettes, one for the background,
    and another for the sprites. One palette set is composed of 4 palettes, where each palette is
    composed of four colors, each an index into the system palette. The first color in the first palette
    in the first palette set is a shared background color (used for transparency); the first color in
    other three palettes is not used by the PPU when normally rendering. The first color in the palettes
    in the second palette set are aliased to those from the first palette set.

    The PPU has access to 256 tiles of raw pixel art (commonly called "pattern table", or CHR), where each
    tile has 2 bit depth. The tiles are stored as two 1-bit images - the first 8 bytes store the low
    bits of the 8x8 tile, and the next 8 bytes store the high bits. Each 2-bit pixel is an index into
    the palette.

    The background is composed of 32x30 tiles, each 8x8 pixels big; the PPU has access to a 960 byte long
    tile map (commonly called "nametable") where each byte is an index into the pattern table. After each 960 byte
    long background tilemap follows a 64 byte long table (commonly called "attribute table") of packed palette
    indexes; this table controls which of the 4 palettes a given chunk of the background uses. Each 16x16 pixel area
    has one palette assigned to it and one byte controls four 16x16 blocks.

    Consider this 32x32 pixel big chunk (or four tiles, each 8x8 pixel big, in a 2x2 grid) of the background:

        -------------------------------------------------------------------------
        | A A A A A A A A | A A A A A A A A | B B B B B B B B | B B B B B B B B |
        | A A A A A A A A | A A A A A A A A | B B B B B B B B | B B B B B B B B |
        | A A A A A A A A | A A A A A A A A | B B B B B B B B | B B B B B B B B |
        | A A A A A A A A | A A A A A A A A | B B B B B B B B | B B B B B B B B |
        | A A A A A A A A | A A A A A A A A | B B B B B B B B | B B B B B B B B |
        | A A A A A A A A | A A A A A A A A | B B B B B B B B | B B B B B B B B |
        | A A A A A A A A | A A A A A A A A | B B B B B B B B | B B B B B B B B |
        | A A A A A A A A | A A A A A A A A | B B B B B B B B | B B B B B B B B |
        -------------------------------------------------------------------------
        | A A A A A A A A | A A A A A A A A | B B B B B B B B | B B B B B B B B |
        | A A A A A A A A | A A A A A A A A | B B B B B B B B | B B B B B B B B |
        | A A A A A A A A | A A A A A A A A | B B B B B B B B | B B B B B B B B |
        | A A A A A A A A | A A A A A A A A | B B B B B B B B | B B B B B B B B |
        | A A A A A A A A | A A A A A A A A | B B B B B B B B | B B B B B B B B |
        | A A A A A A A A | A A A A A A A A | B B B B B B B B | B B B B B B B B |
        | A A A A A A A A | A A A A A A A A | B B B B B B B B | B B B B B B B B |
        | A A A A A A A A | A A A A A A A A | B B B B B B B B | B B B B B B B B |
        -------------------------------------------------------------------------
        | C C C C C C C C | C C C C C C C C | D D D D D D D D | D D D D D D D D |
        | C C C C C C C C | C C C C C C C C | D D D D D D D D | D D D D D D D D |
        | C C C C C C C C | C C C C C C C C | D D D D D D D D | D D D D D D D D |
        | C C C C C C C C | C C C C C C C C | D D D D D D D D | D D D D D D D D |
        | C C C C C C C C | C C C C C C C C | D D D D D D D D | D D D D D D D D |
        | C C C C C C C C | C C C C C C C C | D D D D D D D D | D D D D D D D D |
        | C C C C C C C C | C C C C C C C C | D D D D D D D D | D D D D D D D D |
        | C C C C C C C C | C C C C C C C C | D D D D D D D D | D D D D D D D D |
        -------------------------------------------------------------------------
        | C C C C C C C C | C C C C C C C C | D D D D D D D D | D D D D D D D D |
        | C C C C C C C C | C C C C C C C C | D D D D D D D D | D D D D D D D D |
        | C C C C C C C C | C C C C C C C C | D D D D D D D D | D D D D D D D D |
        | C C C C C C C C | C C C C C C C C | D D D D D D D D | D D D D D D D D |
        | C C C C C C C C | C C C C C C C C | D D D D D D D D | D D D D D D D D |
        | C C C C C C C C | C C C C C C C C | D D D D D D D D | D D D D D D D D |
        | C C C C C C C C | C C C C C C C C | D D D D D D D D | D D D D D D D D |
        | C C C C C C C C | C C C C C C C C | D D D D D D D D | D D D D D D D D |
        -------------------------------------------------------------------------

    Every two bits of a byte from the packed palette indexes table control the palette of one 16x16 pixel region:

        |  Bit | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
        | Area | D | D | C | C | B | B | A | A |

    Assuming we know the X and Y coordinates of the current tile we can calculate:
        packed_palette_indexes_address = tilemap_address + 960 + tile_y / 4 * (32 / 4) + tile_x / 4
    where:
        tilemap_address -> see the memory map for details
                    960 -> length of a tilemap
                 tile_x -> X coordinate of an 8x8 pixel tile, <0, 32)
                 tile_y -> Y coordinate of an 8x8 pixel tile, <0, 30)
         tile_[x|y] / 4 -> since one byte of the packed palette index table encodes information
                           for a 4x4 block of tiles
               (32 / 4) -> there are 32 tiles in a row and for every 4 of those tiles we have one byte of
                           data in the packed palette index table

    We can also figure out which bits correspond to the current tile:
        bits_to_shift = (tile_x & 2) + (tile_y & 2) * 2
        palette_index = (packed_palette_indexes >> bits_to_shift) & 0b11
    where:
        tile_[x|y] & 2 -> For <0, inf) this produces: 0, 0, 2, 2, 0, 0, 2, 2, ...

    The 2-bit pixel values from the pattern table are used as an index into the palette specified
    by the packed palette index table to get the system palette index of the pixel.

                          8-bit index                      2-bit pixel                   4-bit index
        Background Tile Index -> Pattern Table ----------------------------------------+-------------> System Palette -> Pixel Color
                              |                                                        |
                              |                       2-bit palette index              |
                              -> Packed Palette Index Table -> Background Palette Set -^

    PPU memory map:
        |                 |      |
        |      RANGE      | SIZE |              CONTENTS
        |                 |      |
        ----------------------------------------------------------------------------
        | 0x0000...0x0FFF |  4kb | Pattern table #0 (256 tiles, each 8x8 in size, each 16 bytes long)
        | 0x1000...0x1FFF |  4kb | Pattern table #1
        | 0x2000...0x23FF |  1kb | Background tilemap #0 (960 byte long tile map and 64 byte long packed palette index table)
        | 0x2400...0x27FF |  1kb | Background tilemap #1
        | 0x2800...0x2BFF |  1kb | Background tilemap #2
        | 0x2C00...0x2FFF |  1kb | Background tilemap #3
        | 0x3000...0x3EFF | 3840 | Mirror of 0x2000...0x2EFF
        | 0x3F00...0x3F1F |  32b | Palette RAM (16 byte long background palettes and 16 byte long sprite palettes)
        | 0x3F20...0x3FFF | 224b | Mirrors of palette RAM (7 in total)
        | 0x4000...0xFFFF |      | Mirrors of 0x0000...0x3FFF

    0x0000...0x1FFF is mapped by the cartridge to either a VRAM or a VROM area on the cartridge itself.
    A cartridge can have either of those, or even both, and can have multiple banks that it can switch
    between. It can also map the 0x2000...0x2FFF area.

    The PPU has only 2kb of RAM for the background tilemaps, so by default it can only hold two
    background tilemaps simultaneously. The way the physical background tilemaps are mapped to logical
    ones is determined by the cartridge; some of the possible configurations are:
        - Horizontal
            #0 and #1 point to the same background tilemap
            #2 and #3 point to the same background tilemap
        - Vertical
            #0 and #2 point to the same background tilemap
            #1 and #4 point to the same background tilemap
        - Single screen
            #0, #1, #2 and #3 all point to the same background tilemap
        - No mirroring
            All of the background tilemaps are distinct (requires extra RAM on the cartridge)
        - Other
            Diagonal, L-shaped, 3-screen vertical, 3-screen horizontal, etc.

    The whole 0x0000...0x3EFF range can be mapped by the circuitry on the cartridge.
    For more details see http://wiki.nesdev.com/w/index.php/Mirroring

    (The following is strictly true for NTSC only.)
    PPU drawing procedure:
        1. PPU renders dummy scanline; on every second frame this scanline is 1 pixel shorter at the end of the scanline;
           the VINT flag is cleared on the second tick of this scanline.
        2. PPU renders 240 visible scanlines.
        3. PPU waits for 1 scanline.
        4. PPU waits for 20 scanlines; the VINT flag is set on the second tick of the first scanline.

    The VINT flag is also cleared after 0x2002 (PPUSTATUS) is read.

    The PPU renders 262 scanlines per frame in total.
    Each scanline takes 341 PPU cycles, with each cycle producing one pixel.
*/

#[derive(PartialEq, Eq)]
pub struct Framebuffer {
    buffer: alloc::boxed::Box<[u16; 256 * 240]>
}

impl Framebuffer {
    fn new() -> Framebuffer {
        Framebuffer::default()
    }

    pub fn iter< 'a >( &'a self ) -> FramebufferIterator< 'a > {
        FramebufferIterator {
            iter: self.buffer.iter()
        }
    }

    pub fn convert_to_abgr< T: AsMut< [u32] >>( &self, palette: &Palette, mut output: T ) {
        let array = output.as_mut();
        assert_eq!( array.len(), 256 * 240 );

        for (pixel, out) in self.iter().zip( array.iter_mut() ) {
            *out = palette.get_packed_abgr( pixel.full_color_index() );
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct FramebufferPixel( u16 );

impl FramebufferPixel {
    #[inline]
    pub const fn base_color_index( &self ) -> u8 {
        (self.0 & 0b111111) as u8
    }

    #[inline]
    pub fn full_color_index( &self ) -> u16 {
        self.0
    }

    #[inline]
    pub const fn is_red_emphasized( &self ) -> bool {
        (self.0 & 0b00000000_01000000) != 0
    }

    #[inline]
    pub const fn is_green_emphasized( &self ) -> bool {
        (self.0 & 0b00000000_10000000) != 0
    }

    #[inline]
    pub const fn is_blue_emphasized( &self ) -> bool {
        (self.0 & 0b00000001_00000000) != 0
    }
}

pub struct FramebufferIterator< 'a > {
    iter: slice::Iter< 'a, u16 >
}

impl< 'a > Iterator for FramebufferIterator< 'a > {
    type Item = FramebufferPixel;

    #[inline]
    fn next( &mut self ) -> Option< Self::Item > {
        self.iter.next().map( |&raw_pixel| FramebufferPixel( raw_pixel ) )
    }
}

impl Default for Framebuffer {
    fn default() -> Self {
        Framebuffer {
            buffer: alloc::boxed::Box::new([0; 256 * 240])
        }
    }
}

/*
    The NES doesn't output an RGB signal; it directly outputs analog video signal, hence
    there is a multitude of ways of interpreting the colors it generates.
*/
#[derive(Clone)]
#[repr(transparent)]
pub struct Palette( [u32; 512] );

const fn generate_emphasis_colors( mut palette: Palette ) -> Palette {
    use crate::float_softfloat::*;

    let mut index = 64;
    while index < 512 {
        let pixel = FramebufferPixel( index );
        // TODO: This isn't really accurate.
        let (r, g, b) = palette.get_rgb( pixel.base_color_index() as u16 );
        let mut r = u8_to_f32(r).div( softfloat::f32!(255.0) );
        let mut g = u8_to_f32(g).div( softfloat::f32!(255.0) );
        let mut b = u8_to_f32(b).div( softfloat::f32!(255.0) );
        if pixel.is_red_emphasized() {
            r = r.mul( softfloat::f32!(1.1) );
            g = g.mul( softfloat::f32!(0.85) );
            b = b.mul( softfloat::f32!(0.85) );
        }
        if pixel.is_green_emphasized() {
            r = r.mul( softfloat::f32!(0.85) );
            g = g.mul( softfloat::f32!(1.1) );
            b = b.mul( softfloat::f32!(0.85) );
        }
        if pixel.is_blue_emphasized() {
            r = r.mul( softfloat::f32!(0.85) );
            g = g.mul( softfloat::f32!(0.85) );
            b = b.mul( softfloat::f32!(1.1) );
        }

        const fn clamp(value: F32) -> F32 {
            if matches!( value.cmp( softfloat::f32!(1.0) ), None | Some( core::cmp::Ordering::Greater ) ) {
                softfloat::f32!(1.0)
            } else {
                value
            }
        }

        let r = clamp(r).mul( softfloat::f32!(255.0) );
        let g = clamp(g).mul( softfloat::f32!(255.0) );
        let b = clamp(b).mul( softfloat::f32!(255.0) );
        palette.0[ index as usize ] =
            f32_to_u32(r) |
            f32_to_u32(g) << 8 |
            f32_to_u32(b) << 16;

        index += 1;
    }

    palette
}

impl Palette {
    #[inline(always)]
    pub const fn new( data: &[u8] ) -> Palette {
        assert!( data.len() == 192 || data.len() == 1536 );

        let mut palette = Palette( [0; 512] );
        let mut index_in = 0;
        let mut index_out = 0;
        while index_in < data.len() {
            let r = data[ index_in ] as u32;
            let g = data[ index_in + 1 ] as u32;
            let b = data[ index_in + 2 ] as u32;
            palette.0[ index_out ] = r | g << 8 | b << 16 | 0xFF << 24;
            index_in += 3;
            index_out += 1;
        }

        if data.len() == 192 {
            palette = generate_emphasis_colors( palette );
        }

        palette
    }

    pub const fn get_packed_abgr( &self, index: u16 ) -> u32 {
        debug_assert!( index < 512 );
        self.0[ index as usize ]
    }

    pub const fn get_rgb( &self, index: u16 ) -> (u8, u8, u8) {
        let value = self.get_packed_abgr( index );
        let r = (value & 0x000000FF) as u8;
        let g = ((value & 0x0000FF00) >> 8) as u8;
        let b = ((value & 0x00FF0000) >> 16) as u8;

        (r, g, b)
    }
}

impl Palette {
    pub fn get_default() -> &'static Palette {
        // Source: http://forums.nesdev.com/viewtopic.php?p=150239#p150239
        static DEFAULT: Palette = Palette::new( include_bytes!( "../data/FBX-Final.pal" ) );
        &DEFAULT
    }
}

impl Default for Palette {
    #[inline(always)]
    fn default() -> Palette {
        Self::get_default().clone()
    }
}

enum SpriteEvaluationMode {
    Search,
    Copy,
    Idle
}

#[derive(Copy, Clone)]
struct Sprite {
    pattern_lo_shift_register: u8,
    pattern_hi_shift_register: u8,
    attributes_latch: u8,
    dots_until_is_displayed: u8
}

impl Sprite {
    #[inline]
    const fn new() -> Self {
        Sprite {
            pattern_lo_shift_register: 0,
            pattern_hi_shift_register: 0,
            attributes_latch: 0,
            dots_until_is_displayed: 0
        }
    }

    #[inline]
    fn display_sprite_behind_background( &self ) -> bool {
        is_b5_set( self.attributes_latch )
    }

    #[inline]
    fn palette_index( &self ) -> u8 {
        self.attributes_latch & 0b11
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self::new()
    }
}

ppu_scheduling_logic!();

trait Private: Sized + Context {
    fn fetch( &mut self ) -> u8 {
        self.peek( self.state().address )
    }

    fn sprite_evaluation( &mut self ) {
        if self.state().odd_cycle_flag {
            // On odd cycles data is read from the primary sprite list.
            self.state_mut().sprite_list_data_latch = self.state().sprite_list_ram.peek( self.state().auxiliary_sprite_list_address );
            self.state_mut().auxiliary_sprite_list_address.wrapping_inc();
        } else {
            // On even cycles data is written to the secondary sprite list.
            match self.state().sprite_evaluation_mode {
                SpriteEvaluationMode::Search => {
                    {
                        let address = self.state().secondary_sprite_list_address;
                        let value = self.state().sprite_list_data_latch;
                        self.poke_secondary_sprite_list_ram( address, value );
                    }

                    let y = self.state().sprite_list_data_latch as u16;
                    let is_in_range = if !self.state().ppuctrl.big_sprite_mode() {
                        (self.state().n_scanline >= y) && (self.state().n_scanline < y + 8)
                    } else {
                        (self.state().n_scanline >= y) && (self.state().n_scanline < y + 16)
                    };

                    if is_in_range {
                        if self.state_mut().secondary_sprite_list_address == 0 {
                            self.state_mut().first_sprite_is_sprite_zero_on_next_scanline = self.state().auxiliary_sprite_list_address == 1;
                        }

                        self.state_mut().sprite_evaluation_mode = SpriteEvaluationMode::Copy;
                        self.state_mut().secondary_sprite_list_address += 1;
                    } else {
                        self.state_mut().auxiliary_sprite_list_address = self.state().auxiliary_sprite_list_address.wrapping_add( 3 );
                        //if self.state().auxiliary_sprite_list_address >= 255 {
                        //    self.state_mut().sprite_evaluation_mode = SpriteEvaluationMode::Idle;
                        //}
                    }
                },
                SpriteEvaluationMode::Copy => {
                    {
                        let address = self.state().secondary_sprite_list_address;
                        let value = self.state().sprite_list_data_latch;
                        self.poke_secondary_sprite_list_ram( address, value );
                    }

                    self.state_mut().secondary_sprite_list_address += 1;
                    if self.state().secondary_sprite_list_address % 4 == 0 {
                        if self.state().secondary_sprite_list_address / 4 == 8 /*|| self.state().auxiliary_sprite_list_address >= 255*/ {
                            // TODO: This isn't accurate.
                            self.state_mut().sprite_evaluation_mode = SpriteEvaluationMode::Idle;
                        } else {
                            self.state_mut().sprite_evaluation_mode = SpriteEvaluationMode::Search;
                        }
                    }
                },
                SpriteEvaluationMode::Idle => {}
            }
        }
        self.state_mut().odd_cycle_flag = !self.state().odd_cycle_flag;
    }

    fn current_sprite_mut( &mut self ) -> &mut Sprite {
        let index = self.state().sprite_index;
        self.state_mut().sprites.at_mut( index )
    }

    fn clear_secondary_sprite_ram_cell( &mut self ) {
        if self.state().odd_cycle_flag {

        } else {
            {
                let address = self.state().secondary_sprite_list_address;
                self.state_mut().secondary_sprite_list_ram.poke( address, 0xFF );
            }
            self.state_mut().secondary_sprite_list_address += 1;
            if self.state().secondary_sprite_list_address == 32 {
                for i in 0..32 {
                    let value = self.state().secondary_sprite_list_ram[i];
                    debug_assert_eq!( value, 0xFF );
                }
            }
            self.state_mut().secondary_sprite_list_address = self.state().secondary_sprite_list_address & (32 - 1);
        }

        self.state_mut().odd_cycle_flag = !self.state().odd_cycle_flag;
    }

    fn shift_sprite_registers( &mut self ) {
        for sprite in self.state_mut().sprites.iter_mut() {
            if sprite.dots_until_is_displayed == 0 {
                sprite.pattern_lo_shift_register <<= 1;
                sprite.pattern_hi_shift_register <<= 1;
            }
        }
    }

    fn decrement_sprite_horizontal_counters( &mut self ) {
        for sprite in self.state_mut().sprites.iter_mut() {
            if sprite.dots_until_is_displayed != 0 {
                sprite.dots_until_is_displayed -= 1;
            }
        }
    }

    fn update_sprite_registers( &mut self ) {
        for sprite in self.state_mut().sprites.iter_mut() {
            if sprite.dots_until_is_displayed == 0 {
                sprite.pattern_lo_shift_register <<= 1;
                sprite.pattern_hi_shift_register <<= 1;
            } else {
                sprite.dots_until_is_displayed -= 1;
            }
        }
    }

    fn reload_background_shift_registers( &mut self ) {
        self.state_mut().bg_pattern_lo_shift_register = (self.state().bg_pattern_lo_shift_register & 0xFF00) | (self.state().tile_lo_latch as u16);
        self.state_mut().bg_pattern_hi_shift_register = (self.state().bg_pattern_hi_shift_register & 0xFF00) | (self.state().tile_hi_latch as u16);
        self.state_mut().bg_palette_index_lo_shift_register = (self.state().bg_palette_index_lo_shift_register & 0xFF00) | (((self.state().background_palette_index_latch as u16 >> 0) & 1) * 0x00FF);
        self.state_mut().bg_palette_index_hi_shift_register = (self.state().bg_palette_index_hi_shift_register & 0xFF00) | (((self.state().background_palette_index_latch as u16 >> 1) & 1) * 0x00FF);
    }

    #[inline]
    fn sprite_tile_lo_address( &self ) -> u16 {
        let flip_vertically = is_b7_set( self.state().sprite_attributes_latch );

        if self.state().ppuctrl.big_sprite_mode() == false {
            let index = self.state().sprite_pattern_index_latch;
            let mut tile_y = (self.state().n_scanline as i16 - self.state().sprite_vertical_position_latch as i16) & 7;
            debug_assert!( tile_y < 8 );

            if flip_vertically {
                tile_y = (8 - 1) - tile_y;
            }

            self.state().ppuctrl.sprite_pattern_table_address() + index as u16 * 16 + tile_y as u16
        } else {
            // In double height sprite mode the pattern table is not selected
            // by the ppuctrl; it's taken from the least significant bit of
            // the sprite's index attribute.
            let pattern_table_address = if self.state().sprite_pattern_index_latch & 1 == 0 {
                0x0000
            } else {
                0x1000
            };

            // The top and the bottom 8x8 sprite patterns that compose a single
            // 8x16 sprite are laid out in memory interleaved, e.g.:
            //     sprite0_top, sprite0_bottom, sprite1_top, sprite1_bottom, ...
            // so we mask out the least significant bit of the index to get
            // the index of the top sprite.
            let top_sprite_index = self.state().sprite_pattern_index_latch & (!1);
            let bottom_sprite_index = top_sprite_index + 1;

            let sprite_y = (self.state().n_scanline as i16 - self.state().sprite_vertical_position_latch as i16) & 15;
            let mut is_upper_sprite = sprite_y < 8;

            if flip_vertically {
                // The top and the bottom tiles of the sprite are swapped.
                is_upper_sprite = !is_upper_sprite;
            }

            let mut tile_y = if sprite_y < 8 {
                sprite_y
            } else {
                sprite_y - 8
            };

            let index = if is_upper_sprite {
                top_sprite_index
            } else {
                bottom_sprite_index
            };

            if flip_vertically {
                tile_y = (8 - 1) - tile_y;
            }

            pattern_table_address + index as u16 * 16 + tile_y as u16
        }
    }

    #[inline]
    fn sprite_tile_hi_address( &self ) -> u16 {
        self.sprite_tile_lo_address() + 8
    }

    #[inline]
    fn sprite_tile_lo_address_lo( &self ) -> u8 {
        self.sprite_tile_lo_address() as u8
    }

    #[inline]
    fn sprite_tile_lo_address_hi( &self ) -> u8 {
        (self.sprite_tile_lo_address() >> 8) as u8
    }

    #[inline]
    fn sprite_tile_hi_address_lo( &self ) -> u8 {
        self.sprite_tile_hi_address() as u8
    }

    #[inline]
    fn sprite_tile_hi_address_hi( &self ) -> u8 {
        (self.sprite_tile_hi_address() >> 8) as u8
    }

    fn framebuffer( &mut self ) -> &Framebuffer {
        self.state_mut().framebuffer()
    }

    fn swap_framebuffer( &mut self, mut other: Framebuffer ) -> Framebuffer {
        mem::swap( self.state_mut().framebuffer(), &mut other );
        other
    }

    fn is_rendering_enabled( &self ) -> bool {
        self.state().ppumask.show_background() || self.state().ppumask.show_sprites()
    }

    fn local_pixel_coordinate_x( &self ) -> u8 {
        self.state().fine_horizontal_scroll
    }

    fn local_pixel_coordinate_y( &self ) -> u8 {
        debug_assert!( (self.state().current_address & (1 << 15)) == 0 ); // The register is only 15-bit.
        self.state().current_address.get_bits( 0b0111000000000000 ) as u8
    }

    fn set_local_pixel_coordinate_y( &mut self, value: u8 ) {
        self.state_mut().current_address.replace_bits( 0b0111000000000000, value as u16 );
    }

    fn increment_horizontal_counters( &mut self ) {
        if !self.rendering_is_enabled() {
            return;
        }
        if self.tile_x() == 0b11111 {
            self.state_mut().current_address ^= 1 << 10; // Select next horizontal tilemap.
            self.set_tile_x( 0 );
        } else {
            let value = self.tile_x() + 1;
            self.set_tile_x( value );
        }
    }

    fn increment_vertical_counters( &mut self ) {
        if !self.rendering_is_enabled() {
            return;
        }
        let local_y = self.local_pixel_coordinate_y();
        if local_y != 0b111 {
            self.set_local_pixel_coordinate_y( local_y + 1 );
        } else {
            self.set_local_pixel_coordinate_y( 0 );
            let tile_y = self.tile_y();
            if tile_y == 29 {
                self.set_tile_y( 0 );
                self.state_mut().current_address ^= 1 << 11; // Select next vertical tilemap.
            } else if tile_y == 31 {
                // The tile Y coordinate can be manually set to be out-of-bounds.
                self.set_tile_y( 0 );
            } else {
                self.set_tile_y( tile_y + 1 );
            }
        }
    }

    fn reset_horizontal_counters( &mut self ) {
        if self.rendering_is_enabled() {
            let mask = 0b0000010000011111;
            let value = self.state().temporary_address;
            self.state_mut().current_address.copy_bits_from( mask, value );
        }
    }

    fn reset_vertical_counters( &mut self ) {
        if self.rendering_is_enabled() {
            let mask = 0b0111101111100000;
            let value = self.state().temporary_address;
            self.state_mut().current_address.copy_bits_from( mask, value );
        }
    }

    fn tile_x( &self ) -> u8 {
        (self.state().current_address >> 0) as u8 & 0b11111
    }

    fn set_tile_x( &mut self, value: u8 ) {
        self.state_mut().current_address.replace_bits( 0b11111, value as u16 );
    }

    fn tile_y( &self ) -> u8 {
        (self.state().current_address >> 5) as u8 & 0b11111
    }

    fn set_tile_y( &mut self, value: u8 ) {
        self.state_mut().current_address.replace_bits( 0b1111100000, value as u16 );
    }

    fn tilemap_address( &self ) -> u16 {
        0x2000 + (self.state().current_address & 0b0000110000000000)
    }

    fn pattern_index_address( &self ) -> u16 {
        (self.tilemap_address() + (self.tile_x() as u16 + self.tile_y() as u16 * 32)) & 0b0011111111111111
    }

    fn packed_palette_indexes_address( &self ) -> u16 {
        if self.state().n_dot > 256 && self.state().n_dot < 321 {
            // This is for the garbage fetches during the sprite fetching.
            // Not sure if this is how it's supposed to be done, but it seems
            // to match up with the real PPU behavior so far.
            self.tilemap_address() + (self.tile_y() as u16 / 4 * (32 / 4)) + (self.state().current_address & 0xFF)
        } else {
            (self.tilemap_address() + 960) + (self.tile_y() as u16 / 4 * (32 / 4)) + (self.tile_x() as u16 / 4)
        }
    }

    fn bg_tile_lo_address( &self, index: u8 ) -> u16 {
        self.state().ppuctrl.background_pattern_table_address() + index as u16 * 16 + self.local_pixel_coordinate_y() as u16
    }

    fn bg_tile_hi_address( &self, index: u8 ) -> u16 {
        self.state().ppuctrl.background_pattern_table_address() + index as u16 * 16 + 8 + self.local_pixel_coordinate_y() as u16
    }

    fn background_pixel( &self ) -> (u8, u8) {
        if self.state().ppumask.show_background() && (self.state().ppumask.show_background_in_leftmost_8_pixels() || self.state().n_dot >= 8) {
            let shift = 15 - self.local_pixel_coordinate_x();
            let pattern_lo = (self.state().bg_pattern_lo_shift_register >> shift) & 1; // << (7 - self.local_pixel_coordinate_x())) & 1;
            let pattern_hi = (self.state().bg_pattern_hi_shift_register >> shift) & 1; // (7 - self.local_pixel_coordinate_x())) & 1;
            let palette_index_lo = (self.state().bg_palette_index_lo_shift_register >> shift) & 1;
            let palette_index_hi = (self.state().bg_palette_index_hi_shift_register >> shift) & 1;

            let color_in_palette_index = pattern_lo | (pattern_hi << 1);
            let palette_index = palette_index_lo | (palette_index_hi << 1);

            (palette_index as u8, color_in_palette_index as u8)
        } else {
            (0, 0)
        }
    }

    fn sprite_pixel( &self ) -> (u8, u8, bool, bool) {
        if self.state().ppumask.show_sprites() && (self.state().ppumask.show_sprites_in_leftmost_8_pixels() || self.state().n_dot >= 8) {
            for (nth, sprite) in self.state().sprites.iter().enumerate() {
                if sprite.dots_until_is_displayed != 0 {
                    continue;
                }

                let pattern_lo = (sprite.pattern_lo_shift_register >> 7) & 1;
                let pattern_hi = (sprite.pattern_hi_shift_register >> 7) & 1;
                if pattern_lo == 0 && pattern_hi == 0 {
                    continue;
                }

                let color_in_palette_index = pattern_lo | (pattern_hi << 1);
                let palette_index = 4 + sprite.palette_index();

                return (palette_index, color_in_palette_index, sprite.display_sprite_behind_background(), nth == 0);
            }
        }

        (0, 0, true, false)
    }

    fn pixel( &mut self ) -> (u8, u8) {
        let (background_palette_index, background_color_index) = self.background_pixel();
        let (sprite_palette_index, sprite_color_index, display_sprite_behind_background, is_sprite_zero) = self.sprite_pixel();

        // Since sprite 0 hit doesn't happen on dot 255 I'm also assuming
        // that we're not supposed to draw any sprites that have their
        // position set to 255. If we do draw sprites on dot 255 then we
        // usually get this ugly vertical line there since the sprites'
        // position is cleared to be 0xFF by default, so any unused sprites
        // end up on dot 255.
        if self.state().n_dot == 255 || self.state().n_scanline == 0 {
            return (background_palette_index, background_color_index);
        }

        if self.state().first_sprite_is_sprite_zero_on_current_scanline && sprite_color_index != 0 && background_color_index != 0 && is_sprite_zero {
            self.state_mut().ppustatus.modify_sprite_0_hit( true );
        }

        match (background_color_index, sprite_color_index, display_sprite_behind_background) {
            (0, 0,     _) => (0, 0),
            (0, _,     _) => (sprite_palette_index, sprite_color_index),
            (_, 0,     _) => (background_palette_index, background_color_index),
            (_, _, false) => (sprite_palette_index, sprite_color_index),
            (_, _,  true) => (background_palette_index, background_color_index)
        }
    }

    fn output_pixel( &mut self ) {
        let (palette_index, color_in_palette_index) = self.pixel();

        let color_in_system_palette_index = self.peek( 0x3F00 + (palette_index as u16 * 4) + color_in_palette_index as u16 );
        // let color_in_system_palette_index = self.state_mut().palette_ram.peek( (palette_index as usize * 4) + color_in_palette_index as usize );
        let nth = self.state().n_pixel;
        let pixel = ((self.state().ppumask.color_emphasize_bits() as u16) << 6) | (color_in_system_palette_index as u16);
        {
            let framebuffer = match self.state_mut().framebuffer {
                Some( ref mut framebuffer ) => framebuffer,
                None => unsafe { core::hint::unreachable_unchecked() }
            };
            framebuffer.buffer.poke( nth, pixel );
        }
        self.state_mut().n_pixel += 1;
    }

    fn shift_background_registers( &mut self ) {
        self.state_mut().bg_pattern_lo_shift_register <<= 1;
        self.state_mut().bg_pattern_hi_shift_register <<= 1;
        self.state_mut().bg_palette_index_lo_shift_register <<= 1;
        self.state_mut().bg_palette_index_hi_shift_register <<= 1;
    }

    fn rendering_is_enabled( &mut self ) -> bool {
        self.state().ppumask.show_background() || self.state().ppumask.show_sprites()
    }

    fn execute( &mut self ) {
        self.state_mut().framebuffer(); // Make sure the framebuffer is initialized.

        self.execute_next_action();
        self.on_cycle();

        if self.state().skip_next_cycle {
            // Technically we're supposed to skip the next cycle,
            // but we might just as well execute it.
            self.state_mut().skip_next_cycle = false;
            self.execute_next_action();
        }
    }

    fn execute_next_action( &mut self ) {
        let action = unsafe { get_action( self.state().action_index as usize ) };
        (action)( self );
        self.state_mut().action_index += 1;
        self.on_next_action();
    }

    fn on_next_action( &mut self ) {
        let chunk = CHUNKS.at( self.state().chunk_index );

        if self.state().action_index > chunk.last_action_index {
            self.state_mut().chunk_counter += 1;
            if self.state().chunk_counter != chunk.times {
                self.state_mut().action_index = chunk.first_action_index;
            } else {
                self.state_mut().chunk_counter = 0;
                self.state_mut().chunk_index += 1;

                let scanline = SCANLINES.at( self.state().scanline_index );
                if self.state().chunk_index > scanline.last_chunk_index {
                    self.state_mut().scanline_counter += 1;
                    if self.state().scanline_counter != scanline.times {
                        self.state_mut().chunk_index = scanline.first_chunk_index;
                        let chunk = CHUNKS.at( self.state().chunk_index );
                        self.state_mut().action_index = chunk.first_action_index;
                    } else {
                        self.state_mut().scanline_index += 1;
                        self.state_mut().scanline_counter = 0;
                        if self.state().scanline_index >= (SCANLINES.len() as u8) {
                            self.state_mut().scanline_index = 0;
                        }
                        let scanline = SCANLINES.at( self.state().scanline_index );
                        self.state_mut().chunk_index = scanline.first_chunk_index;
                        let chunk = CHUNKS.at( self.state().chunk_index );
                        self.state_mut().action_index = chunk.first_action_index;
                    }
                } else {
                    let chunk = CHUNKS.at( self.state().chunk_index );
                    self.state_mut().action_index = chunk.first_action_index;
                }
            }
        }
    }

    fn try_trigger_vblank_nmi( &mut self ) {
        let value = self.state().ppuctrl.should_generate_vblank_nmi() && self.state().ppustatus.vblank_has_occured();
        self.set_vblank_nmi( value );
    }

    fn peek_ppustatus( &mut self ) -> u8 {
        let value = self.state().ppustatus.0;
        self.state_mut().ppustatus.modify_vblank_has_occured( false );
        self.state_mut().write_toggle = false;
        self.state_mut().vblank_flag_was_cleared = true;

        let result = (value & 0b11100000) | (self.state().residual_data & 0b00011111);
        self.state_mut().residual_data = result;
        result
    }

    fn poke_ppuctrl( &mut self, value: u8 ) {
        self.state_mut().ppuctrl.0 = value;
        self.state_mut().temporary_address = (self.state().temporary_address & 0b0111001111111111) | ((value as u16 & 0b11) << 10);
    }

    fn poke_ppumask( &mut self, value: u8 ) {
        self.state_mut().ppumask.0 = value;
    }

    fn poke_ppuaddr( &mut self, value: u8 ) {
        if self.state().write_toggle == false {
            // Write the upper byte. The register is only 15-bit, and the 14-bit always get zero'd
            // so we AND the value with 0x3F. The first write always goes to the temporary address register.
            self.state_mut().temporary_address = (((value & 0x3F) as u16) << 8) | (self.state().temporary_address & 0x00FF);
            self.state_mut().write_toggle = true;
        } else {
            // Write the lower byte.
            // On the second write the current address register gets updated.
            self.state_mut().temporary_address = (value as u16) | (self.state().temporary_address & 0xFF00);
            self.state_mut().current_address = self.state().temporary_address;
            self.state_mut().write_toggle = false;
        }
    }

    fn poke_ppuscroll( &mut self, value: u8 ) {
        // The scrolling IO register shares the same physical register as the address IO register.
        if self.state().write_toggle == false {
            self.state_mut().fine_horizontal_scroll = value & 0b111;
            self.state_mut().temporary_address = (self.state().temporary_address & 0b0111111111100000) | ((value >> 3) as u16);
            self.state_mut().write_toggle = true;
        } else {
            //   VALUE  -> TEMPORARY ADDR. REG.
            // XXYYYZZZ -> 0ZZZ--XXYYY-----
            //
            //      '0': always zero, since the register is 15-bit
            //      '-': the bit is unmodified
            //      'X', 'Y', 'Z': that region of bits is copied from the input value

            let tmp = ((((value & 0b00000111) as u16) >> 0) << 12) |
                      ((((value & 0b00111000) as u16) >> 3) <<  5) |
                      ((((value & 0b11000000) as u16) >> 6) <<  8);

            self.state_mut().temporary_address = (self.state().temporary_address & 0b0000110000011111) | tmp;
            self.state_mut().write_toggle = false;
        }
    }

    fn poke_ppudata( &mut self, value: u8 ) {
        let address = self.state().current_address & 0x3FFF; // Addresses >= 0x3FFF are mirrors of 0...0x3FFF.

        self.poke( address, value );
        self.increment_current_address();
    }

    fn peek_ppudata( &mut self ) -> u8 {
        let address = self.state().current_address & 0x3FFF;
        let mut value = self.peek( address );

        self.increment_current_address();
        if address <= 0x3EFF {
            // Put the value into the read buffer; return the old value.
            let old_value = self.state().ppudata_read_buffer;
            self.state_mut().ppudata_read_buffer = value;
            self.state_mut().residual_data = old_value;
            old_value
        } else {
            // Data from the palette RAM is always immediately available, though
            // when reading the pallete RAM the read buffer gets updated with
            // a value from 0x3000...0x3EFF range just as if that mirror would be
            // extended to cover whole 0x3000...0x3FFF.
            self.state_mut().ppudata_read_buffer = self.peek_video_memory( address );

            debug_assert!( (value & 0b11000000) == 0 );
            if self.state().ppumask.grayscale_mode() {
                // When the greyscale mode is on only the following colors will be returned:
                //   0x00 - 0b000000
                //   0x10 - 0b010000
                //   0x20 - 0b100000
                //   0x30 - 0b110000
                value &= 0b110000;
            }
            self.state_mut().residual_data = (self.state().residual_data & 0b11000000) | value;
            value
        }
    }

    fn increment_current_address( &mut self ) {
        let addr_increment = self.state().ppuctrl.vram_address_increment();
        self.state_mut().current_address = self.state().current_address.wrapping_add( addr_increment ) & 0x7FFF;
    }

    fn poke_oamaddr( &mut self, value: u8 ) {
        self.state_mut().sprite_list_address = value;
    }

    fn poke_oamdata( &mut self, value: u8 ) {
        let index = self.state().sprite_list_address;
        self.state_mut().sprite_list_ram.poke( index, value );
        self.state_mut().sprite_list_address.wrapping_inc();
    }

    fn peek_oamdata( &mut self ) -> u8 {
        let index = self.state().sprite_list_address;
        self.state_mut().sprite_list_ram.peek( index )
    }

    fn poke_sprite_list_ram( &mut self, index: u8, value: u8 ) {
        self.state_mut().sprite_list_ram.poke( index, value );
    }

    fn poke_secondary_sprite_list_ram( &mut self, index: u8, value: u8 ) {
        self.state_mut().secondary_sprite_list_ram.poke( index, value );
    }

    fn peek( &self, mut address: u16 ) -> u8 {
        if address <= 0x3EFF {
            self.peek_video_memory( address )
        } else {
            address = address & (32 - 1);
            debug_assert!( address < 32 );

            self.state().palette_ram.peek( address )
        }
    }

    fn poke( &mut self, mut address: u16, mut value: u8 ) {
        if address <= 0x3EFF {
            self.poke_video_memory( address, value );
        } else {
            address = address & (32 - 1);
            debug_assert!( address < 32 );

            // The memory cells containing palette data are limited to only 64 values.
            value &= 64 - 1;

            self.state_mut().palette_ram.poke( address, value );

            if (address & 0b11) == 0 {
                // Duplicate the write to the mirrored location. See the palette_access test for details.
                if address >= 16 {
                    address -= 16;
                } else {
                    address += 16;
                }
                self.state_mut().palette_ram.poke( address, value );
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{State, Context, Private};

    struct DummyPPU {
        cycle: usize,
        state: State,
        memory: [u8; 0xFFFF + 1]
    }

    impl DummyPPU {
        fn new() -> Self {
            DummyPPU {
                cycle: 0,
                state: State::new(),
                memory: [0; 0xFFFF + 1]
            }
        }
    }

    impl Context for DummyPPU {
        fn state_mut( &mut self ) -> &mut State {
            &mut self.state
        }

        fn state( &self ) -> &State {
            &self.state
        }

        fn on_cycle( &mut self ) {
            self.cycle += 1;
        }

        fn on_frame_was_generated( &mut self ) {}
        fn set_vblank_nmi( &mut self, _: bool ) {}
        fn peek_video_memory( &self, offset: u16 ) -> u8 {
            self.memory[ offset as usize ]
        }

        fn poke_video_memory( &mut self, offset: u16, value: u8 ) {
            self.memory[ offset as usize ] = value
        }
    }

    #[test]
    fn vblank_flag_clear() {
        struct TestPPU {
            cycle: usize,
            nmi_set_cycle: Option< usize >,
            nmi_cleared_cycle: Option< usize >,
            state: State
        }

        impl TestPPU {
            fn new() -> TestPPU {
                TestPPU {
                    cycle: 0,
                    nmi_set_cycle: None,
                    nmi_cleared_cycle: None,
                    state: State::new()
                }
            }

            fn execute_frame( &mut self ) {
                for _ in 0..(341 * 262) {
                    self.execute();
                }
            }
        }

        impl Context for TestPPU {
            fn state_mut( &mut self ) -> &mut State {
                &mut self.state
            }

            fn state( &self ) -> &State {
                &self.state
            }

            fn on_cycle( &mut self ) {
                self.cycle += 1;
                let vblank = self.state_mut().ppustatus.vblank_has_occured();

                if self.nmi_set_cycle.is_none() && vblank {
                    self.nmi_set_cycle = Some( self.cycle );
                } else if self.nmi_set_cycle.is_some() && self.nmi_cleared_cycle.is_none() && !vblank {
                    self.nmi_cleared_cycle = Some( self.cycle );
                }
            }

            fn on_frame_was_generated( &mut self ) {}
            fn set_vblank_nmi( &mut self, _: bool ) {}
            fn peek_video_memory( &self, _: u16 ) -> u8 { 0 }
            fn poke_video_memory( &mut self, _: u16, _: u8 ) {}
        }

        let mut ppu = TestPPU::new();

        ppu.execute_frame();
        assert_eq!( ppu.cycle, 262 * 341 );
        ppu.execute_frame();

        // The vblank flag is cleared 6820 PPU clocks after it's set.
        assert_eq!( ppu.nmi_cleared_cycle.unwrap() - ppu.nmi_set_cycle.unwrap(), 6820 );
    }

    #[test]
    fn poke_ppuaddr_once() {
        let mut ppu = DummyPPU::new();

        ppu.state.current_address = 0xCCCC;
        ppu.state.temporary_address = 0xFFFF;
        ppu.poke_ppuaddr( 0x12 );
        assert_eq!( ppu.state.temporary_address, 0x12FF ); // Writes into the high byte of the temporary address register.
        assert_eq!( ppu.state.current_address, 0xCCCC ); // Current address register is unchanged.
    }

    #[test]
    fn poke_ppuaddr_twice() {
        let mut ppu = DummyPPU::new();

        ppu.poke_ppuaddr( 0x12 );
        ppu.poke_ppuaddr( 0x34 );
        assert_eq!( ppu.state.temporary_address, 0x1234 ); // Writes into the low byte of the temporary address register.
        assert_eq!( ppu.state.current_address, 0x1234 ); // The value is copied to the current address register.
    }

    #[test]
    fn poke_ppuaddr_and_peek_status() {
        let mut ppu = DummyPPU::new();

        ppu.state.current_address = 0xCCCC;
        ppu.state.temporary_address = 0xFFFF;
        ppu.poke_ppuaddr( 0x12 );
        ppu.peek_ppustatus(); // A read from the PPUSTATUS clears the high/low flag.
        assert_eq!( ppu.state.temporary_address, 0x12FF ); // Only the upper byte was written.
        assert_eq!( ppu.state.current_address, 0xCCCC ); // Current address register is unchanged.
    }

    #[test]
    fn poke_ppuaddr_0xFF() {
        let mut ppu = DummyPPU::new();

        ppu.poke_ppuaddr( 0xFF );
        assert_eq!( ppu.state.temporary_address, 0x3F00 ); // The upper two bits are masked out on the first write.
        ppu.poke_ppuaddr( 0xFF );
        assert_eq!( ppu.state.temporary_address, 0x3FFF );
        assert_eq!( ppu.state.current_address, 0x3FFF );
    }

    #[test]
    fn poke_ppuctrl_temporary_address_write() {
        let mut ppu = DummyPPU::new();

        ppu.poke_ppuctrl( 0b00000011 );
        assert_eq!( ppu.state.temporary_address, 0b0000110000000000 );
        ppu.poke_ppuctrl( 0b00000000 );
        assert_eq!( ppu.state.temporary_address, 0b0000000000000000 );
        ppu.poke_ppuctrl( 0b00000001 );
        assert_eq!( ppu.state.temporary_address, 0b0000010000000000 );
        ppu.poke_ppuctrl( 0b00000010 );
        assert_eq!( ppu.state.temporary_address, 0b0000100000000000 );
        ppu.poke_ppuctrl( 0b11111100 );
        assert_eq!( ppu.state.temporary_address, 0b0000000000000000 );
    }

    #[test]
    fn poke_ppuscroll_once() {
        let mut ppu = DummyPPU::new();

        ppu.poke_ppuscroll( 0b11111111 );
        assert_eq!( ppu.state.temporary_address, 0b0000000000011111 ); // Writes into the low five bits of the temporary address register...
        assert_eq!( ppu.state.fine_horizontal_scroll, 0b111 ); // ...and to the fine X scroll register.
        assert_eq!( ppu.state.current_address, 0 ); // Unmodified.
        ppu.poke_ppuscroll( 0 ); // To reset the high/low toggle.

        // Try writing various patterns.
        ppu.poke_ppuscroll( 0b00000111 );
        ppu.poke_ppuscroll( 0 ); // To reset the high/low toggle.
        assert_eq!( ppu.state.temporary_address, 0b0000000000000000 );
        assert_eq!( ppu.state.fine_horizontal_scroll, 0b111 );

        ppu.poke_ppuscroll( 0b11111000 );
        ppu.poke_ppuscroll( 0 );
        assert_eq!( ppu.state.temporary_address, 0b0000000000011111 );
        assert_eq!( ppu.state.fine_horizontal_scroll, 0b000 );

        ppu.poke_ppuscroll( 0b10101010 );
        ppu.poke_ppuscroll( 0 );
        assert_eq!( ppu.state.temporary_address, 0b0000000000010101 );
        assert_eq!( ppu.state.fine_horizontal_scroll, 0b010 );
    }

    #[test]
    fn poke_ppuscroll_twice() {
        let mut ppu = DummyPPU::new();

        ppu.poke_ppuscroll( 0b11111111 );
        ppu.poke_ppuscroll( 0b11111111 ); // Writes into the five high bits and three low bits of the temporary address register...
        assert_eq!( ppu.state.temporary_address, 0b0111001111111111 ); // ...the bits 11 and 10 are untouched.
        assert_eq!( ppu.state.current_address, 0 ); // Unmodified.

        ppu.state.temporary_address = 0b0111111111111111;
        ppu.poke_ppuscroll( 0b11111111 );
        ppu.poke_ppuscroll( 0b11111111 );
        assert_eq!( ppu.state.temporary_address, 0b0111111111111111 ); // Again, bits 11 and 10 are untouched.
        ppu.state.temporary_address = 0;

        // Write to the register with successively different patterns turning off bits one by one.
        ppu.poke_ppuscroll( 0b11111111 );
        ppu.poke_ppuscroll( 0b11111110 );
        assert_eq!( ppu.state.temporary_address, 0b0110001111111111 );

        ppu.poke_ppuscroll( 0b11111111 );
        ppu.poke_ppuscroll( 0b11111100 );
        assert_eq!( ppu.state.temporary_address, 0b0100001111111111 );

        ppu.poke_ppuscroll( 0b11111111 );
        ppu.poke_ppuscroll( 0b11111000 );
        assert_eq!( ppu.state.temporary_address, 0b0000001111111111 );

        ppu.poke_ppuscroll( 0b11111111 );
        ppu.poke_ppuscroll( 0b01111000 );
        assert_eq!( ppu.state.temporary_address, 0b0000000111111111 );

        ppu.poke_ppuscroll( 0b11111111 );
        ppu.poke_ppuscroll( 0b00111000 );
        assert_eq!( ppu.state.temporary_address, 0b0000000011111111 );

        ppu.poke_ppuscroll( 0b11111111 );
        ppu.poke_ppuscroll( 0b00011000 );
        assert_eq!( ppu.state.temporary_address, 0b0000000001111111 );

        ppu.poke_ppuscroll( 0b11111111 );
        ppu.poke_ppuscroll( 0b00001000 );
        assert_eq!( ppu.state.temporary_address, 0b0000000000111111 );

        ppu.poke_ppuscroll( 0b11111111 );
        ppu.poke_ppuscroll( 0b00000000 );
        assert_eq!( ppu.state.temporary_address, 0b0000000000011111 );
    }

    #[test]
    fn poke_ppuaddr_and_ppuscroll() {
        let mut ppu = DummyPPU::new();

        // Both IO registers share the same high/low toggle and the same address registers.
        ppu.poke_ppuaddr( 0b11111111 );
        ppu.poke_ppuscroll( 0b11111111 );
        assert_eq!( ppu.state.temporary_address, 0b0111111111100000 );
    }

    #[test]
    fn palette_access() {
        let mut ppu = DummyPPU::new();

        ppu.poke( 0x3F00, 255 );
        assert_eq!( ppu.peek( 0x3F00 ), 63 ); // The cells are limited to 6 bits.

        // Clear the memory.
        for i in 0..32 {
            ppu.poke( 0x3F00 + i, 63 );
        }

        // Write unique value to every cell in the first palette set.
        for i in 0..16 {
            ppu.poke( 0x3F00 + i, i as u8 );
        }

        // First palette set.
        assert_eq!( ppu.peek( 0x3F00 +  0 ),  0 );
        assert_eq!( ppu.peek( 0x3F00 +  1 ),  1 );
        assert_eq!( ppu.peek( 0x3F00 +  2 ),  2 );
        assert_eq!( ppu.peek( 0x3F00 +  3 ),  3 );
        assert_eq!( ppu.peek( 0x3F00 +  4 ),  4 );
        assert_eq!( ppu.peek( 0x3F00 +  5 ),  5 );
        assert_eq!( ppu.peek( 0x3F00 +  6 ),  6 );
        assert_eq!( ppu.peek( 0x3F00 +  7 ),  7 );
        assert_eq!( ppu.peek( 0x3F00 +  8 ),  8 );
        assert_eq!( ppu.peek( 0x3F00 +  9 ),  9 );
        assert_eq!( ppu.peek( 0x3F00 + 10 ), 10 );
        assert_eq!( ppu.peek( 0x3F00 + 11 ), 11 );
        assert_eq!( ppu.peek( 0x3F00 + 12 ), 12 );
        assert_eq!( ppu.peek( 0x3F00 + 13 ), 13 );
        assert_eq!( ppu.peek( 0x3F00 + 14 ), 14 );
        assert_eq!( ppu.peek( 0x3F00 + 15 ), 15 );

        // Second palette set.
        assert_eq!( ppu.peek( 0x3F00 + 16 ),  0 ); // Mirrored from 0x3F00.
        assert_eq!( ppu.peek( 0x3F00 + 17 ), 63 ); // Untouched.
        assert_eq!( ppu.peek( 0x3F00 + 18 ), 63 );
        assert_eq!( ppu.peek( 0x3F00 + 19 ), 63 );
        assert_eq!( ppu.peek( 0x3F00 + 20 ),  4 ); // Mirrored from 0x3F04.
        assert_eq!( ppu.peek( 0x3F00 + 21 ), 63 );
        assert_eq!( ppu.peek( 0x3F00 + 22 ), 63 );
        assert_eq!( ppu.peek( 0x3F00 + 23 ), 63 );
        assert_eq!( ppu.peek( 0x3F00 + 24 ),  8 ); // Mirrored from 0x3F08.
        assert_eq!( ppu.peek( 0x3F00 + 25 ), 63 );
        assert_eq!( ppu.peek( 0x3F00 + 26 ), 63 );
        assert_eq!( ppu.peek( 0x3F00 + 27 ), 63 );
        assert_eq!( ppu.peek( 0x3F00 + 28 ), 12 ); // Mirrored from 0x3F0C.
        assert_eq!( ppu.peek( 0x3F00 + 29 ), 63 );
        assert_eq!( ppu.peek( 0x3F00 + 30 ), 63 );
        assert_eq!( ppu.peek( 0x3F00 + 31 ), 63 );

        // Write unique value to every cell in the second palette set.
        for i in 16..32 {
            ppu.poke( 0x3F00 + i, i as u8 );
        }

        // First palette set.
        assert_eq!( ppu.peek( 0x3F00 +  0 ), 16 ); // Mirrored from 0x3F10.
        assert_eq!( ppu.peek( 0x3F00 +  1 ),  1 ); // Untouched.
        assert_eq!( ppu.peek( 0x3F00 +  2 ),  2 );
        assert_eq!( ppu.peek( 0x3F00 +  3 ),  3 );
        assert_eq!( ppu.peek( 0x3F00 +  4 ), 20 ); // Mirrored from 0x3F14.
        assert_eq!( ppu.peek( 0x3F00 +  5 ),  5 );
        assert_eq!( ppu.peek( 0x3F00 +  6 ),  6 );
        assert_eq!( ppu.peek( 0x3F00 +  7 ),  7 );
        assert_eq!( ppu.peek( 0x3F00 +  8 ), 24 ); // Mirrored from 0x3F18.
        assert_eq!( ppu.peek( 0x3F00 +  9 ),  9 );
        assert_eq!( ppu.peek( 0x3F00 + 10 ), 10 );
        assert_eq!( ppu.peek( 0x3F00 + 11 ), 11 );
        assert_eq!( ppu.peek( 0x3F00 + 12 ), 28 ); // Mirrored from 0x3F1C.
        assert_eq!( ppu.peek( 0x3F00 + 13 ), 13 );
        assert_eq!( ppu.peek( 0x3F00 + 14 ), 14 );
        assert_eq!( ppu.peek( 0x3F00 + 15 ), 15 );

        // Second palette set.
        assert_eq!( ppu.peek( 0x3F00 + 16 ), 16 );
        assert_eq!( ppu.peek( 0x3F00 + 17 ), 17 );
        assert_eq!( ppu.peek( 0x3F00 + 18 ), 18 );
        assert_eq!( ppu.peek( 0x3F00 + 19 ), 19 );
        assert_eq!( ppu.peek( 0x3F00 + 20 ), 20 );
        assert_eq!( ppu.peek( 0x3F00 + 21 ), 21 );
        assert_eq!( ppu.peek( 0x3F00 + 22 ), 22 );
        assert_eq!( ppu.peek( 0x3F00 + 23 ), 23 );
        assert_eq!( ppu.peek( 0x3F00 + 24 ), 24 );
        assert_eq!( ppu.peek( 0x3F00 + 25 ), 25 );
        assert_eq!( ppu.peek( 0x3F00 + 26 ), 26 );
        assert_eq!( ppu.peek( 0x3F00 + 27 ), 27 );
        assert_eq!( ppu.peek( 0x3F00 + 28 ), 28 );
        assert_eq!( ppu.peek( 0x3F00 + 29 ), 29 );
        assert_eq!( ppu.peek( 0x3F00 + 30 ), 30 );
        assert_eq!( ppu.peek( 0x3F00 + 31 ), 31 );
    }
}

#[cfg(test)]
mod test_ppu {
    use super::{Context, State, Private};
    use rp2c02_testsuite::TestPPU;
    use std::cell::Cell;
    use std::fmt;

    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct MemoryOperation {
        pub address: u16,
        pub value: u8
    }

    impl fmt::Debug for MemoryOperation {
        fn fmt( &self, fmt: &mut fmt::Formatter ) -> fmt::Result {
            write!( fmt, "{{ address: 0x{:04X}, value: 0x{:02X} }}", self.address, self.value )
        }
    }

    struct Instance {
        state: State,
        memory: [u8; 0x2400],
        last_vram_read: Cell< Option< MemoryOperation > >,
        expected_vram_read: Option< Option< MemoryOperation > >
    }

    impl Context for Instance {
        fn state_mut( &mut self ) -> &mut State {
            &mut self.state
        }

        fn state( &self ) -> &State {
            &self.state
        }

        fn on_frame_was_generated( &mut self ) {}
        fn set_vblank_nmi( &mut self, _: bool ) {}

        fn peek_video_memory( &self, address: u16 ) -> u8 {
            // The simulator uses one screen mirroring of the background tilemaps.
            let value = self.memory[ ((address & 0x23FF) | (address & 0x2000)) as usize ];

            self.last_vram_read.set( Some( MemoryOperation {
                address: address,
                value: value
            }));

            value
        }

        fn poke_video_memory( &mut self, _: u16, _: u8 ) {
            unreachable!();
        }
    }

    impl Instance {
        fn new() -> Self {
            Instance {
                state: State::new(),
                memory: [0; 0x2400],
                last_vram_read: Cell::new( None ),
                expected_vram_read: None
            }
        }
    }

    impl TestPPU for Instance {
        fn expect_vram_read( &mut self, address: u16, value: u8 ) {
            self.expected_vram_read = Some( Some( MemoryOperation {
                address: address,
                value: value
            }));
        }

        fn expect_no_vram_read( &mut self ) {
            self.expected_vram_read = Some( None );
        }

        fn get_current_address( &self ) -> u16 {
            self.state.current_address
        }

        fn get_temporary_address( &self ) -> u16 {
            self.state.temporary_address
        }

        fn read_ioreg( &mut self, index: u8 ) -> u8 {
            match index {
                2 => self.peek_ppustatus(),
                4 => self.peek_oamdata(),
                7 => self.peek_ppudata(),
                _ => unreachable!()
            }
        }

        fn read_secondary_sprite_ram( &self, index: u8 ) -> u8 {
            self.state.secondary_sprite_list_ram[ index as usize ]
        }

        fn write_ioreg( &mut self, index: u8, value: u8 ) {
            match index {
                0 => self.poke_ppuctrl( value ),
                1 => self.poke_ppumask( value ),
                3 => self.poke_oamaddr( value ),
                4 => self.poke_oamdata( value ),
                5 => self.poke_ppuscroll( value ),
                6 => self.poke_ppuaddr( value ),
                7 => self.poke_ppudata( value ),
                _ => unreachable!()
            }
        }

        fn write_vram( &mut self, address: u16, value: u8 ) {
            self.memory[ address as usize ] = value;
        }

        fn write_palette_ram( &mut self, index: u8, value: u8 ) {
            self.state.palette_ram[ index as usize ] = value;
        }

        fn write_sprite_ram( &mut self, index: u8, value: u8 ) {
            self.state.sprite_list_ram[ index as usize ] = value;
        }

        fn write_secondary_sprite_ram( &mut self, index: u8, value: u8 ) {
            self.state.secondary_sprite_list_ram[ index  as usize] = value;
        }

        fn step_pixel( &mut self ) {
            self.last_vram_read = Cell::new( None );

            self.execute();

            if let Some( expected_vram_read ) = self.expected_vram_read.take() {
                assert_eq!( self.last_vram_read.get(), expected_vram_read );
            }
        }

        fn scanline( &self ) -> u16 {
            self.state.n_scanline
        }

        fn dot( &self ) -> u16 {
            self.state.n_dot
        }
    }

    rp2c02_testsuite!( Instance );
}
