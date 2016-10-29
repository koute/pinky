use std::ops::Sub;
use emumisc::{PeekPoke, BitExtra, is_b0_set, is_b7_set};
use rom::{NesRom, LoadError, Mirroring};
use mappers::Mapper;
use memory_map::{
    self,
    SRAM_ADDRESS,
    LOWER_ROM_ADDRESS,
    UPPER_ROM_ADDRESS,
    translate_address_background_tilemap
};

// FIXME: This implementation of MMC1 is not yet accurate.
// At very least the serial writes to the serial port
// should be ignored, and they are not currently.
// This does break a few games.

#[inline]
fn wraparound< T: Sub< Output = T > + PartialOrd + Copy >( limit: T, mut value: T ) -> T {
    // This isn't going to be super fast,
    // and it's probably unnecessary if we have
    // a bank count that is a power of two
    // (since then we could have used a simple
    // bitwise 'and' then), but for now it'll do.

    while value >= limit {
        value = value - limit;
    }

    value
}

const SHIFT_REGISTER_DEFAULT_VALUE: u8 = 0b10000;

#[derive(Debug)]
enum SwitchingModeForROM {
    Fused, // Lower and upper ROM banks are treated as a single, big, switchable bank.
    OnlyLower, // Lower ROM bank is switchable; upper ROM bank is fixed to the last physical bank.
    OnlyUpper // Upper ROM bank is switchable; lower ROM bank is fixed to the first physical bank.
}

#[derive(Debug)]
enum SwitchingModeForVROM {
    Fused, // Upper and lower VROM banks are treated as a single, big, switchable bank.
    Independent // Upper and lower VROM banks are independently switched.
}

pub struct MapperMMC1 {
    // At 0x6000 - 0x7FFF, fixed on most boards.
    current_sram_offset: i32,

    // At 0x8000 - 0xBFFF, either switchable or fixed to the first bank.
    current_lower_rom_offset: i32,

    // At 0xC000 - 0xFFFF, either switchable or fixed to the last bank.
    current_upper_rom_offset: i32,

    // At 0x0000 - 0x0FFF in the PPU address space.
    current_lower_vrom_offset: i32,

    // At 0x1000 - 0x1FFF in the PPU address space.
    current_upper_vrom_offset: i32,

    sram: Vec< u8 >,
    rom: Vec< u8 >,
    vrom: Vec< u8 >,
    background_tilemaps: [u8; 2048],
    is_vrom_writable: bool,

    shift_register: u8,

    mirroring: fn( u16 ) -> u16,
    rom_switching_mode: SwitchingModeForROM,
    vrom_switching_mode: SwitchingModeForVROM,
    selected_rom_bank: u8,
    selected_lower_vrom_bank: u8,
    selected_upper_vrom_bank: u8,
    is_sram_writable: bool
}

const LOWER_VROM_ADDRESS: u16 = 0x0000;
const UPPER_VROM_ADDRESS: u16 = 0x1000;

const SRAM_BANK_SIZE: i32 = 8 * 1024;
const ROM_BANK_SIZE: i32 = 16 * 1024;
const VROM_BANK_SIZE: i32 = 4 * 1024;

// These allow us to bake the memory address of a given memory
// region in our offsets, effectively saving us one extra operation
// per memory access.
const SRAM_OFFSET: i32 = -(SRAM_ADDRESS as i32);
const LOWER_ROM_OFFSET: i32 = -(LOWER_ROM_ADDRESS as i32);
const UPPER_ROM_OFFSET: i32 = -(UPPER_ROM_ADDRESS as i32);
const LOWER_VROM_OFFSET: i32 = -(LOWER_VROM_ADDRESS as i32);
const UPPER_VROM_OFFSET: i32 = -(UPPER_VROM_ADDRESS as i32);

impl MapperMMC1 {
    fn empty() -> Self {
        MapperMMC1 {
            current_sram_offset: SRAM_OFFSET,
            current_lower_rom_offset: LOWER_ROM_OFFSET,
            current_upper_rom_offset: UPPER_ROM_OFFSET,
            current_lower_vrom_offset: LOWER_VROM_OFFSET,
            current_upper_vrom_offset: UPPER_VROM_OFFSET,

            sram: Vec::new(),
            rom: Vec::new(),
            vrom: Vec::new(),
            background_tilemaps: [0; 2048],
            is_vrom_writable: false,

            shift_register: SHIFT_REGISTER_DEFAULT_VALUE,

            mirroring: memory_map::horizontal_mirroring,
            rom_switching_mode: SwitchingModeForROM::OnlyLower,
            vrom_switching_mode: SwitchingModeForVROM::Independent,
            selected_rom_bank: 0,
            selected_lower_vrom_bank: 0,
            selected_upper_vrom_bank: 1,
            is_sram_writable: true // TODO: Different carts have different default.
        }
    }

    pub fn from_rom( rom: NesRom ) -> Result< Self, LoadError > {
        let mut mapper = Self::empty();
        mapper.sram.resize( rom.ram_bank_count as usize * SRAM_BANK_SIZE as usize, 0 );
        for rom_bank in rom.rom_banks {
            mapper.rom.extend_from_slice( &rom_bank[..] );
        }

        if mapper.rom.len() < 2 * ROM_BANK_SIZE as usize {
            mapper.rom.resize( 2 * ROM_BANK_SIZE as usize, 0 );
        }

        if rom.vrom_banks.is_empty() {
            // This means we simply have 8k of VRAM here.
            mapper.vrom.resize( 2 * VROM_BANK_SIZE as usize, 0 );
            mapper.is_vrom_writable = true;
        } else {
            for vrom_bank in rom.vrom_banks {
                mapper.vrom.extend_from_slice( &vrom_bank[..] );
            }
        }

        mapper.mirroring = match rom.mirroring {
            Mirroring::Horizontal => memory_map::horizontal_mirroring,
            Mirroring::Vertical => memory_map::vertical_mirroring,
            _ => return Err( LoadError::new( format!( "Unsupported mirroring: {:?}", rom.mirroring ) ) )
        };

        mapper.update_offsets();
        Ok( mapper )
    }

    fn update_offsets( &mut self ) {
        self.current_sram_offset = 0;

        let rom_size = self.rom.len() as i32;
        match self.rom_switching_mode {
            SwitchingModeForROM::Fused => {
                let bank = (self.selected_rom_bank as i32 / 2) * 2;
                self.current_lower_rom_offset = wraparound( rom_size, bank * ROM_BANK_SIZE );
                self.current_upper_rom_offset = wraparound( rom_size, self.current_lower_rom_offset + ROM_BANK_SIZE );
            },
            SwitchingModeForROM::OnlyLower => {
                self.current_lower_rom_offset = wraparound( rom_size, self.selected_rom_bank as i32 * ROM_BANK_SIZE );
                self.current_upper_rom_offset = rom_size - ROM_BANK_SIZE;
            },
            SwitchingModeForROM::OnlyUpper => {
                self.current_lower_rom_offset = 0;
                self.current_upper_rom_offset = wraparound( rom_size, self.selected_rom_bank as i32 * ROM_BANK_SIZE );
            }
        }

        let vrom_size = self.vrom.len() as i32;
        match self.vrom_switching_mode {
            SwitchingModeForVROM::Fused => {
                let bank = (self.selected_lower_vrom_bank as i32 / 2) * 2;
                self.current_lower_vrom_offset = wraparound( vrom_size, bank * VROM_BANK_SIZE );
                self.current_upper_vrom_offset = wraparound( vrom_size, self.current_lower_vrom_offset + VROM_BANK_SIZE );
            },
            SwitchingModeForVROM::Independent => {
                self.current_lower_vrom_offset = wraparound( vrom_size, self.selected_lower_vrom_bank as i32 * VROM_BANK_SIZE );
                self.current_upper_vrom_offset = wraparound( vrom_size, self.selected_upper_vrom_bank as i32 * VROM_BANK_SIZE );
            }
        }

        // Now we prebake the offsets.
        self.current_sram_offset += SRAM_OFFSET;
        self.current_lower_rom_offset += LOWER_ROM_OFFSET;
        self.current_upper_rom_offset += UPPER_ROM_OFFSET;
        self.current_lower_vrom_offset += LOWER_VROM_OFFSET;
        self.current_upper_vrom_offset += UPPER_VROM_OFFSET;
    }
}

impl Mapper for MapperMMC1 {
    fn peek_sram( &self, address: u16 ) -> u8 {
        self.sram.peek( self.current_sram_offset.wrapping_add( address as i32 ) )
    }

    fn poke_sram( &mut self, address: u16, value: u8 ) {
        if self.is_sram_writable {
            self.sram.poke( self.current_sram_offset.wrapping_add( address as i32 ), value )
        }
    }

    fn peek_rom( &self, address: u16 ) -> u8 {
        debug_assert!( address >= LOWER_ROM_ADDRESS );

        if address < UPPER_ROM_ADDRESS {
            self.rom.peek( self.current_lower_rom_offset.wrapping_add( address as i32 ) )
        } else {
            debug_assert!( address >= UPPER_ROM_ADDRESS );
            self.rom.peek( self.current_upper_rom_offset.wrapping_add( address as i32 ) )
        }
    }

    fn poke_rom( &mut self, address: u16, value: u8 ) {
        debug_assert!( address >= LOWER_ROM_ADDRESS );

        // MMC1 is configured through a serial port, connected
        // to a shift register.

        if is_b7_set( value ) {
            // The shift register is reset when a value with 7th bit
            // set is written.
            self.shift_register = SHIFT_REGISTER_DEFAULT_VALUE;
            return;
        }

        let bit = value & 1;

        // We shift the bit in from the left.
        let full_value = (self.shift_register >> 1) | (bit << 4);

        if !is_b0_set( self.shift_register ) {
            self.shift_register = full_value;
            return;
        }

        // We've already shifted five times since the '1'
        // that was originally in the 4th place was now
        // in the 0th one.

        // Bits 13 and 14 of the address contain the register to which
        // the write was done.
        let register = address.get_bits( 0b01100000_00000000 );
        self.shift_register = SHIFT_REGISTER_DEFAULT_VALUE;

        match register {
            0b00 => { // Control register.
                self.mirroring = match full_value.get_bits( 0b00011 ) {
                    0b00 => memory_map::only_lower_bank_mirroring,
                    0b01 => memory_map::only_upper_bank_mirroring,
                    0b10 => memory_map::vertical_mirroring,
                    0b11 => memory_map::horizontal_mirroring,
                    _ => unsafe { fast_unreachable!() }
                };

                self.rom_switching_mode = match full_value.get_bits( 0b01100 ) {
                    0b00 | 0b01 => SwitchingModeForROM::Fused,
                    0b10 => SwitchingModeForROM::OnlyUpper,
                    0b11 => SwitchingModeForROM::OnlyLower,
                    _ => unsafe { fast_unreachable!() }
                };

                self.vrom_switching_mode = match full_value.get_bits( 0b10000 ) {
                    0b0 => SwitchingModeForVROM::Fused,
                    0b1 => SwitchingModeForVROM::Independent,
                    _ => unsafe { fast_unreachable!() }
                };

                debug!( "Mirroring = {:?}, ROM switching mode = {:?}, VROM switching mode = {:?}",
                    memory_map::mirroring_to_str( self.mirroring ),
                    self.rom_switching_mode,
                    self.vrom_switching_mode
                );
            },
            0b01 => { // Switch lower VROM bank
                self.selected_lower_vrom_bank = full_value;
            },
            0b10 => { // Switch upper VROM bank
                self.selected_upper_vrom_bank = full_value;
            },
            0b11 => { // Switch ROM bank
                self.selected_rom_bank = full_value.get_bits( 0b01111 );
                self.is_sram_writable = full_value.get_bits( 0b10000 ) == 0;
            },
            _ => unsafe { fast_unreachable!() }
        }

        self.update_offsets();
    }

    fn peek_video_memory( &self, address: u16 ) -> u8 {
        if address <= 0x0FFF {
            self.vrom.peek( self.current_lower_vrom_offset.wrapping_add( address as i32 ) )
        } else if address >= 0x1000 && address <= 0x1FFF {
            self.vrom.peek( self.current_upper_vrom_offset.wrapping_add( address as i32 ) )
        } else {
            let translated_address = (self.mirroring)( translate_address_background_tilemap( address ) );
            self.background_tilemaps.peek( translated_address )
        }
    }

    fn poke_video_memory( &mut self, address: u16, value: u8 ) {
        if address > 0x1FFF {
            let translated_address = (self.mirroring)( translate_address_background_tilemap( address ) );
            self.background_tilemaps.poke( translated_address, value );
        } else {
            if self.is_vrom_writable {
                if address < 0x1000 {
                    self.vrom.poke( self.current_lower_vrom_offset.wrapping_add( address as i32 ), value );
                } else {
                    self.vrom.poke( self.current_upper_vrom_offset.wrapping_add( address as i32 ), value );
                }
            } else {
                warn!( "Unhandled write to the VROM at 0x{:04X} (value=0x{:02X})", address, value );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MapperMMC1;
    use super::{
        LOWER_VROM_ADDRESS,
        UPPER_VROM_ADDRESS
    };

    use memory_map::{
        SRAM_ADDRESS,
        LOWER_ROM_ADDRESS,
        UPPER_ROM_ADDRESS
    };

    use mappers::Mapper;

    const SRAM_BANK_SIZE: usize = super::SRAM_BANK_SIZE as usize;
    const ROM_BANK_SIZE: usize = super::ROM_BANK_SIZE as usize;
    const VROM_BANK_SIZE: usize = super::VROM_BANK_SIZE as usize;

    const RESET_SHIFT_REGISTER: u8 = 1 << 7;

    const CONTROL_REG: u8 = 0b00;
    const LOWER_VROM_BANK_REG: u8 = 0b01;
    const UPPER_VROM_BANK_REG: u8 = 0b10;
    const ROM_BANK_REG: u8 = 0b11;

    const ONLY_LOWER_BANK_MIRRORING: u8 = 0b00011;
    const ROM_SWITCHING_MODE_ONLY_LOWER: u8 = 0b01100;
    const ROM_SWITCHING_MODE_ONLY_UPPER: u8 = 0b01000;
    const ROM_SWITCHING_MODE_FUSED_A: u8 = 0b00000;
    const ROM_SWITCHING_MODE_FUSED_B: u8 = 0b00100;
    const VROM_SWITCHING_MODE_INDEPENDENT: u8 = 0b10000;
    const VROM_SWITCHING_MODE_FUSED: u8 = 0b00000;

    const WRITABLE_SRAM: u8 = 0b00000;
    const UNWRITABLE_SRAM: u8 = 0b10000;

    fn write_reg( mapper: &mut MapperMMC1, register: u8, mut value: u8 ) {
        for _ in 0..4 {
            mapper.poke_rom( 0x8000, value );
            value = value >> 1;
        }
        mapper.poke_rom( 0x8000 | ((register as u16) << 13), value );
    }

    fn setup() -> MapperMMC1 {
        let mut mapper = MapperMMC1::empty();
        mapper.sram.resize( SRAM_BANK_SIZE as usize, 0 );
        mapper.rom.resize( 4 * ROM_BANK_SIZE as usize, 0 );
        mapper.vrom.resize( 4 * VROM_BANK_SIZE as usize, 0 );

        mapper.sram[ 0 ] = 200;
        mapper.sram[ 1 ] = 201;

        mapper.rom[ 0 * ROM_BANK_SIZE + 0 ] = 10;
        mapper.rom[ 0 * ROM_BANK_SIZE + 1 ] = 11;
        mapper.rom[ 1 * ROM_BANK_SIZE + 0 ] = 20;
        mapper.rom[ 1 * ROM_BANK_SIZE + 1 ] = 21;
        mapper.rom[ 2 * ROM_BANK_SIZE + 0 ] = 30;
        mapper.rom[ 2 * ROM_BANK_SIZE + 1 ] = 31;
        mapper.rom[ 3 * ROM_BANK_SIZE + 0 ] = 40;
        mapper.rom[ 3 * ROM_BANK_SIZE + 1 ] = 41;

        mapper.vrom[ 0 * VROM_BANK_SIZE + 0 ] = 110;
        mapper.vrom[ 0 * VROM_BANK_SIZE + 1 ] = 111;
        mapper.vrom[ 1 * VROM_BANK_SIZE + 0 ] = 120;
        mapper.vrom[ 1 * VROM_BANK_SIZE + 1 ] = 121;
        mapper.vrom[ 2 * VROM_BANK_SIZE + 0 ] = 130;
        mapper.vrom[ 2 * VROM_BANK_SIZE + 1 ] = 131;
        mapper.vrom[ 3 * VROM_BANK_SIZE + 0 ] = 140;
        mapper.vrom[ 3 * VROM_BANK_SIZE + 1 ] = 141;

        mapper.poke_rom( 0x8000, RESET_SHIFT_REGISTER );
        mapper
    }

    #[test]
    fn sram() {
        let mut mapper = setup();
        assert_eq!( mapper.peek_sram( SRAM_ADDRESS + 0 ), 200 );
        assert_eq!( mapper.peek_sram( SRAM_ADDRESS + 1 ), 201 );

        write_reg( &mut mapper, ROM_BANK_REG, WRITABLE_SRAM );
        mapper.poke_sram( SRAM_ADDRESS + 0, 10 ); // Should be ignored.
        mapper.poke_sram( SRAM_ADDRESS + 1, 11 );
        assert_eq!( mapper.peek_sram( SRAM_ADDRESS + 0 ), 10 );
        assert_eq!( mapper.peek_sram( SRAM_ADDRESS + 1 ), 11 );

        write_reg( &mut mapper, ROM_BANK_REG, UNWRITABLE_SRAM );
        mapper.poke_sram( SRAM_ADDRESS + 0, 255 ); // Should *not* be ignored.
        mapper.poke_sram( SRAM_ADDRESS + 1, 255 );
        assert_eq!( mapper.peek_sram( SRAM_ADDRESS + 0 ), 10 );
        assert_eq!( mapper.peek_sram( SRAM_ADDRESS + 1 ), 11 );
    }

    #[test]
    fn rom_switchable_lower_bank() {
        let mut mapper = setup();
        write_reg( &mut mapper, CONTROL_REG, ROM_SWITCHING_MODE_ONLY_LOWER );

        write_reg( &mut mapper, ROM_BANK_REG, 0 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 0 ), 10 ); // This switches.
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 1 ), 11 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 0 ), 40 ); // This is hardcoded.
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 1 ), 41 );

        write_reg( &mut mapper, ROM_BANK_REG, 1 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 0 ), 20 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 1 ), 21 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 0 ), 40 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 1 ), 41 );

        write_reg( &mut mapper, ROM_BANK_REG, 3 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 0 ), 40 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 1 ), 41 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 0 ), 40 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 1 ), 41 );
    }

    #[test]
    fn rom_switchable_upper_bank() {
        let mut mapper = setup();
        write_reg( &mut mapper, CONTROL_REG, ROM_SWITCHING_MODE_ONLY_UPPER );

        write_reg( &mut mapper, ROM_BANK_REG, 0 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 0 ), 10 ); // This is hardcoded.
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 1 ), 11 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 0 ), 10 ); // This switches.
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 1 ), 11 );

        write_reg( &mut mapper, ROM_BANK_REG, 1 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 0 ), 10 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 1 ), 11 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 0 ), 20 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 1 ), 21 );

        write_reg( &mut mapper, ROM_BANK_REG, 3 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 0 ), 10 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 1 ), 11 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 0 ), 40 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 1 ), 41 );
    }

    fn test_rom_fused( mapper: &mut MapperMMC1 ) {
        write_reg( mapper, ROM_BANK_REG, 0 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 0 ), 10 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 1 ), 11 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 0 ), 20 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 1 ), 21 );

        write_reg( mapper, ROM_BANK_REG, 1 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 0 ), 10 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 1 ), 11 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 0 ), 20 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 1 ), 21 );

        write_reg( mapper, ROM_BANK_REG, 2 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 0 ), 30 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 1 ), 31 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 0 ), 40 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 1 ), 41 );

        write_reg( mapper, ROM_BANK_REG, 3 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 0 ), 30 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 1 ), 31 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 0 ), 40 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 1 ), 41 );
    }

    #[test]
    fn rom_fused_a() {
        let mut mapper = setup();
        write_reg( &mut mapper, CONTROL_REG, ROM_SWITCHING_MODE_FUSED_A );
        test_rom_fused( &mut mapper );
    }

    #[test]
    fn rom_fused_b() {
        let mut mapper = setup();

        // This is the same as the other one.
        write_reg( &mut mapper, CONTROL_REG, ROM_SWITCHING_MODE_FUSED_B );
        test_rom_fused( &mut mapper );
    }

    #[test]
    fn vrom_peek_independent() {
        let mut mapper = setup();
        write_reg( &mut mapper, CONTROL_REG, VROM_SWITCHING_MODE_INDEPENDENT );

        write_reg( &mut mapper, LOWER_VROM_BANK_REG, 0 );
        write_reg( &mut mapper, UPPER_VROM_BANK_REG, 1 );
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 0 ), 110 );
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 1 ), 111 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 0 ), 120 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 1 ), 121 );

        write_reg( &mut mapper, LOWER_VROM_BANK_REG, 1 );
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 0 ), 120 );
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 1 ), 121 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 0 ), 120 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 1 ), 121 );

        write_reg( &mut mapper, UPPER_VROM_BANK_REG, 0 );
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 0 ), 120 );
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 1 ), 121 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 0 ), 110 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 1 ), 111 );
    }

    #[test]
    fn vrom_poke_independent() {
        let mut mapper = setup();
        mapper.is_vrom_writable = true;
        write_reg( &mut mapper, CONTROL_REG, VROM_SWITCHING_MODE_INDEPENDENT );

        write_reg( &mut mapper, LOWER_VROM_BANK_REG, 0 );
        write_reg( &mut mapper, UPPER_VROM_BANK_REG, 1 );
        mapper.poke_video_memory( LOWER_VROM_ADDRESS + 0, 1 );
        mapper.poke_video_memory( LOWER_VROM_ADDRESS + 1, 2 );
        mapper.poke_video_memory( UPPER_VROM_ADDRESS + 0, 3 );
        mapper.poke_video_memory( UPPER_VROM_ADDRESS + 1, 4 );

        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 0 ), 1 );
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 1 ), 2 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 0 ), 3 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 1 ), 4 );

        assert_eq!( mapper.vrom[ 0 ], 1 );
        assert_eq!( mapper.vrom[ 1 ], 2 );
        assert_eq!( mapper.vrom[ VROM_BANK_SIZE + 0 ], 3 );
        assert_eq!( mapper.vrom[ VROM_BANK_SIZE + 1 ], 4 );
    }

    #[test]
    fn vrom_peek_fused() {
        let mut mapper = setup();
        write_reg( &mut mapper, CONTROL_REG, VROM_SWITCHING_MODE_FUSED );

        write_reg( &mut mapper, LOWER_VROM_BANK_REG, 0 );
        write_reg( &mut mapper, UPPER_VROM_BANK_REG, 0 );
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 0 ), 110 );
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 1 ), 111 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 0 ), 120 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 1 ), 121 );

        write_reg( &mut mapper, UPPER_VROM_BANK_REG, 3 ); // Ignored.
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 0 ), 110 );
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 1 ), 111 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 0 ), 120 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 1 ), 121 );

        write_reg( &mut mapper, LOWER_VROM_BANK_REG, 1 );
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 0 ), 110 );
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 1 ), 111 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 0 ), 120 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 1 ), 121 );

        write_reg( &mut mapper, LOWER_VROM_BANK_REG, 2 );
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 0 ), 130 );
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 1 ), 131 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 0 ), 140 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 1 ), 141 );
    }

    #[test]
    fn vrom_poke_fused() {
        let mut mapper = setup();
        mapper.is_vrom_writable = true;
        write_reg( &mut mapper, CONTROL_REG, VROM_SWITCHING_MODE_FUSED );

        write_reg( &mut mapper, LOWER_VROM_BANK_REG, 0 );
        write_reg( &mut mapper, UPPER_VROM_BANK_REG, 3 );
        mapper.poke_video_memory( LOWER_VROM_ADDRESS + 0, 1 );
        mapper.poke_video_memory( LOWER_VROM_ADDRESS + 1, 2 );
        mapper.poke_video_memory( UPPER_VROM_ADDRESS + 0, 3 );
        mapper.poke_video_memory( UPPER_VROM_ADDRESS + 1, 4 );
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 0 ), 1 );
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 1 ), 2 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 0 ), 3 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 1 ), 4 );

        write_reg( &mut mapper, LOWER_VROM_BANK_REG, 2 );
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 0 ), 130 ); // These are unchanged.
        assert_eq!( mapper.peek_video_memory( LOWER_VROM_ADDRESS + 1 ), 131 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 0 ), 140 );
        assert_eq!( mapper.peek_video_memory( UPPER_VROM_ADDRESS + 1 ), 141 );
    }
}
