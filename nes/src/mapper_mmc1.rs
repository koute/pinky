use emumisc::{BitExtra, is_b0_set, is_b7_set};
use rom::{NesRom, LoadError};
use mappers::Mapper;
use generic_mapper::{bank, BankedGenericMapper};
use memory_map::LOWER_ROM_ADDRESS;

// FIXME: This implementation of MMC1 is not yet accurate.
// At very least the serial writes to the serial port
// should be ignored, and they are not currently.
// This does break a few games.

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
    inner: BankedGenericMapper,

    shift_register: u8,
    rom_switching_mode: SwitchingModeForROM,
    vrom_switching_mode: SwitchingModeForVROM,
    selected_rom_bank: u8,
    selected_lower_vrom_bank: u8,
    selected_upper_vrom_bank: u8
}

impl MapperMMC1 {
    pub fn from_rom( rom: NesRom ) -> Result< Self, LoadError > {
        let mut mapper = MapperMMC1 {
            inner: BankedGenericMapper::from_rom( rom )?,

            shift_register: SHIFT_REGISTER_DEFAULT_VALUE,
            rom_switching_mode: SwitchingModeForROM::OnlyLower,
            vrom_switching_mode: SwitchingModeForVROM::Independent,
            selected_rom_bank: 0,
            selected_lower_vrom_bank: 0,
            selected_upper_vrom_bank: 1
        };

        mapper.update_mapping();
        Ok( mapper )
    }

    fn update_mapping( &mut self ) {
        match self.rom_switching_mode {
            SwitchingModeForROM::Fused => {
                let bank = (self.selected_rom_bank / 2) * 2;
                self.inner.set_cpu_lower_16k_bank_to_bank( bank );
                self.inner.set_cpu_upper_16k_bank_to_bank( bank.wrapping_add( 1 ) );
            },
            SwitchingModeForROM::OnlyLower => {
                let last_bank = self.inner.last_rom_16k_bank();
                self.inner.set_cpu_lower_16k_bank_to_bank( self.selected_rom_bank );
                self.inner.set_cpu_upper_16k_bank_to_bank( last_bank );
            },
            SwitchingModeForROM::OnlyUpper => {
                self.inner.set_cpu_lower_16k_bank_to_bank( 0 ); // First bank.
                self.inner.set_cpu_upper_16k_bank_to_bank( self.selected_rom_bank );
            }
        }

        match self.vrom_switching_mode {
            SwitchingModeForVROM::Fused => {
                let bank = (self.selected_lower_vrom_bank / 2) * 2;
                self.inner.set_ppu_lower_4k_bank_to_bank( bank );
                self.inner.set_ppu_upper_4k_bank_to_bank( bank.wrapping_add( 1 ) );
            },
            SwitchingModeForVROM::Independent => {
                self.inner.set_ppu_lower_4k_bank_to_bank( self.selected_lower_vrom_bank );
                self.inner.set_ppu_upper_4k_bank_to_bank( self.selected_upper_vrom_bank );
            }
        }
    }
}

impl Mapper for MapperMMC1 {
    fn peek_sram( &self, address: u16 ) -> u8 {
        self.inner.peek_sram( address )
    }

    fn poke_sram( &mut self, address: u16, value: u8 ) {
        self.inner.poke_sram( address, value )
    }

    fn peek_rom( &self, address: u16 ) -> u8 {
        self.inner.peek_rom( address )
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
                match full_value.get_bits( 0b00011 ) {
                    0b00 => self.inner.set_only_lower_bank_mirroring(),
                    0b01 => self.inner.set_only_upper_bank_mirroring(),
                    0b10 => self.inner.set_vertical_mirroring(),
                    0b11 => self.inner.set_horizontal_mirroring(),
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

                #[cfg(feature = "log")]
                debug!( "ROM switching mode = {:?}, VROM switching mode = {:?}",
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
                let is_sram_writable = full_value.get_bits( 0b10000 ) == 0;

                self.inner.set_cpu_8k_writable( bank::CPU_8K::Ox6000, is_sram_writable );
            },
            _ => unsafe { fast_unreachable!() }
        }

        self.update_mapping();
    }

    fn peek_video_memory( &self, address: u16 ) -> u8 {
        self.inner.peek_video_memory( address )
    }

    fn poke_video_memory( &mut self, address: u16, value: u8 ) {
        self.inner.poke_video_memory( address, value )
    }
}

#[cfg(test)]
mod tests {
    use super::MapperMMC1;

    use generic_mapper::bank;
    use rom::{Mirroring, NesRom};
    use memory_map::{
        SRAM_ADDRESS,
        LOWER_ROM_ADDRESS,
        UPPER_ROM_ADDRESS,
        LOWER_VROM_ADDRESS,
        UPPER_VROM_ADDRESS
    };

    use mappers::Mapper;

    const ROM_BANK_SIZE: usize = 16 * 1024;
    const VROM_BANK_SIZE: usize = 4 * 1024;

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
        let mut rom = Vec::new();
        let mut vrom = Vec::new();
        rom.resize( 4 * ROM_BANK_SIZE as usize, 0 );
        vrom.resize( 4 * VROM_BANK_SIZE as usize, 0 );

        rom[ 0 * ROM_BANK_SIZE + 0 ] = 10;
        rom[ 0 * ROM_BANK_SIZE + 1 ] = 11;
        rom[ 1 * ROM_BANK_SIZE + 0 ] = 20;
        rom[ 1 * ROM_BANK_SIZE + 1 ] = 21;
        rom[ 2 * ROM_BANK_SIZE + 0 ] = 30;
        rom[ 2 * ROM_BANK_SIZE + 1 ] = 31;
        rom[ 3 * ROM_BANK_SIZE + 0 ] = 40;
        rom[ 3 * ROM_BANK_SIZE + 1 ] = 41;

        vrom[ 0 * VROM_BANK_SIZE + 0 ] = 110;
        vrom[ 0 * VROM_BANK_SIZE + 1 ] = 111;
        vrom[ 1 * VROM_BANK_SIZE + 0 ] = 120;
        vrom[ 1 * VROM_BANK_SIZE + 1 ] = 121;
        vrom[ 2 * VROM_BANK_SIZE + 0 ] = 130;
        vrom[ 2 * VROM_BANK_SIZE + 1 ] = 131;
        vrom[ 3 * VROM_BANK_SIZE + 0 ] = 140;
        vrom[ 3 * VROM_BANK_SIZE + 1 ] = 141;

        let rom = NesRom {
            mapper: 1,
            rom: rom,
            video_rom: vrom,
            save_ram_length: 8 * 1024,
            mirroring: Mirroring::Horizontal
        };

        let mut mapper = MapperMMC1::from_rom( rom ).unwrap();
        mapper.inner.memory_mut()[ 0 ] = 200;
        mapper.inner.memory_mut()[ 1 ] = 201;

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

    #[test]
    fn rom_out_of_range_lower_bank() {
        let mut mapper = setup();
        write_reg( &mut mapper, CONTROL_REG, ROM_SWITCHING_MODE_ONLY_LOWER );

        write_reg( &mut mapper, ROM_BANK_REG, 4 );
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 0 ), 10 ); // This switches.
        assert_eq!( mapper.peek_rom( LOWER_ROM_ADDRESS + 1 ), 11 );
        assert_eq!( mapper.peek_rom( UPPER_ROM_ADDRESS + 0 ), 40 ); // This is hardcoded.
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
        mapper.inner.set_ppu_4k_writable( bank::PPU_4K::Ox0000, true );
        mapper.inner.set_ppu_4k_writable( bank::PPU_4K::Ox1000, true );
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

        let internal_video_rom_offset = mapper.inner.internal_video_rom_offset();
        assert_eq!( mapper.inner.memory()[ internal_video_rom_offset as usize + 0 ], 1 );
        assert_eq!( mapper.inner.memory()[ internal_video_rom_offset as usize + 1 ], 2 );
        assert_eq!( mapper.inner.memory()[ internal_video_rom_offset as usize + VROM_BANK_SIZE + 0 ], 3 );
        assert_eq!( mapper.inner.memory()[ internal_video_rom_offset as usize + VROM_BANK_SIZE + 1 ], 4 );
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
        mapper.inner.set_ppu_4k_writable( bank::PPU_4K::Ox0000, true );
        mapper.inner.set_ppu_4k_writable( bank::PPU_4K::Ox1000, true );
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
