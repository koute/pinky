#![macro_use]

/*
    CPU memory map:
        |                 |    MOST     |      |
        |      RANGE      | SIGNIFICANT | SIZE |              CONTENTS
        |                 |   NIBBLE    |      |
        ----------------------------------------------------------------------------
        | 0x0000...0x07FF | 0000...0000 |  2kb | RAM
        | 0x0800...0x1FFF | 0000...0001 |  6kb | mirrors of RAM
        | 0x2000...0x2007 | 0010...0010 |   8b | I/O registers (PPU, 8 registers)
        | 0x2008...0x3FFF | 0010...0011 |      | mirrors of I/O registers (PPU)
        | 0x4000...0x401F | 0100...0100 |  32b | I/O registers (APU, DMA, Joypads)
        | 0x4020...0x5FFF | 0100...0101 |< 8kb | expansion ROM
        | 0x6000...0x7FFF | 0110...0111 |  8kb | save RAM
        | 0x8000...0xBFFF | 1000...1011 | 16kb | PRG-ROM lower bank
        | 0xC000...0xFFFF | 1100...1111 | 16kb | PRG-ROM upper bank

    Whole 0x4020...0xFFFF is mapped to the cartridge.
*/

pub const SRAM_ADDRESS: u16 = 0x6000;
pub const LOWER_ROM_ADDRESS: u16 = 0x8000;
pub const UPPER_ROM_ADDRESS: u16 = 0xC000;

pub const LOWER_VROM_ADDRESS: u16 = 0x0000;
pub const UPPER_VROM_ADDRESS: u16 = 0x1000;

#[inline]
pub fn translate_address_ram( address: u16 ) -> u16 {
    address & (2048 - 1)
}

#[inline]
pub fn translate_address_ioreg_ppu( address: u16 ) -> u16 {
    address & (8 - 1)
}

#[inline]
pub fn translate_address_ioreg_other( address: u16 ) -> u16 {
    address & (32 - 1)
}

#[inline]
pub fn translate_address_expansion_rom( address: u16 ) -> u16 {
    (address - 0x20) & (8192 - 1)
}

#[inline]
pub fn translate_address_save_ram( address: u16 ) -> u16 {
    address & (8192 - 1)
}

#[inline]
pub fn translate_address_rom( address: u16 ) -> u16 {
    address & (0xffff & !(1 << 15))
}

#[inline]
pub fn translate_address_background_tilemap( address: u16 ) -> u16 {
    address - 0x2000
}

macro_rules! match_cpu_address {(
                $address: ident,
                 $on_ram: expr,
           $on_ioreg_ppu: expr,
         $on_ioreg_other: expr,
       $on_expansion_rom: expr,
            $on_save_ram: expr,
             $on_prg_rom: expr
    ) => (
        /* TODO: Verify that LLVM compiles this into a jump table; if not make it into an explicit jump table. */
        match $address >> (16 - 3) {
            0b000 => {
                $on_ram
            },
            0b001 => {
                $on_ioreg_ppu
            },
            0b010 => {
                if $address <= 0x401F {
                    $on_ioreg_other
                } else {
                    $on_expansion_rom
                }
            },
            0b011 => {
                $on_save_ram
            },
            0b100 | 0b101 | 0b110 | 0b111 => {
                $on_prg_rom
            },
            _ => unsafe { fast_unreachable!() }
        }
    )
}

#[test]
fn test_cpu_memory_map() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Type {
        RAM,
        IOREG_PPU,
        IOREG_OTHER,
        EXPANSION_ROM,
        SAVE_RAM,
        ROM
    }

    fn get( address: u16 ) -> (Type, u16) {
        match_cpu_address!( address,
            (Type::RAM, translate_address_ram( address )),
            (Type::IOREG_PPU, translate_address_ioreg_ppu( address )),
            (Type::IOREG_OTHER, translate_address_ioreg_other( address )),
            (Type::EXPANSION_ROM, translate_address_expansion_rom( address )),
            (Type::SAVE_RAM, translate_address_save_ram( address )),
            (Type::ROM, translate_address_rom( address ))
        )
    }

    assert_eq!( get( 0x0000 ), (Type::RAM, 0x0000) );
    assert_eq!( get( 0x07FF ), (Type::RAM, 0x07FF) );
    assert_eq!( get( 0x0800 ), (Type::RAM, 0x0000) );
    assert_eq!( get( 0x08FF ), (Type::RAM, 0x00FF) );
    assert_eq!( get( 0x0800 ), (Type::RAM, 0x0000) );
    assert_eq!( get( 0x1000 ), (Type::RAM, 0x0000) );
    assert_eq!( get( 0x1FFF ), (Type::RAM, 0x07FF) );

    assert_eq!( get( 0x2000 ), (Type::IOREG_PPU, 0) );
    assert_eq!( get( 0x2001 ), (Type::IOREG_PPU, 1) );
    assert_eq!( get( 0x2007 ), (Type::IOREG_PPU, 7) );
    assert_eq!( get( 0x2008 ), (Type::IOREG_PPU, 0) );
    assert_eq!( get( 0x2009 ), (Type::IOREG_PPU, 1) );
    assert_eq!( get( 0x200F ), (Type::IOREG_PPU, 7) );
    assert_eq!( get( 0x3FFF ), (Type::IOREG_PPU, 7) );

    assert_eq!( get( 0x4000 ), (Type::IOREG_OTHER, 0) );
    assert_eq!( get( 0x401F ), (Type::IOREG_OTHER, 31) );

    assert_eq!( get( 0x4020 ), (Type::EXPANSION_ROM, 0) );
    assert_eq!( get( 0x4021 ), (Type::EXPANSION_ROM, 1) );
    assert_eq!( get( 0x5FFF ), (Type::EXPANSION_ROM, 0x2000 - 0x0020 - 1) );

    assert_eq!( get( 0x6000 ), (Type::SAVE_RAM, 0) );
    assert_eq!( get( 0x6001 ), (Type::SAVE_RAM, 1) );
    assert_eq!( get( 0x7FFF ), (Type::SAVE_RAM, 8 * 1024 - 1) );

    assert_eq!( get( 0x8000 ), (Type::ROM, 0) );
    assert_eq!( get( 0xFFFF ), (Type::ROM, 32 * 1024 - 1) );
}
