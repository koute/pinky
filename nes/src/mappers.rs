use rom::{NesRom, LoadError, Mirroring, ROM_BANK_SIZE, VROM_BANK_SIZE};
use emumisc::{PeekPoke, copy_memory};
use memory_map::{translate_address_rom, translate_address_save_ram, translate_address_background_tilemap, horizontal_mirroring, vertical_mirroring};
use mapper_mmc1::MapperMMC1;

pub trait Mapper {
    fn peek_rom( &self, address: u16 ) -> u8;
    fn poke_rom( &mut self, address: u16, value: u8 );

    fn peek_sram( &self, address: u16 ) -> u8 {
        warn!( "Unhandled read from the save RAM at 0x{:04X}", address );
        0
    }

    fn poke_sram( &mut self, address: u16, value: u8 ) {
        warn!( "Unhandled write to the save RAM at 0x{:04X} (value=0x{:02X})", address, value );
    }

    fn peek_expansion_rom( &self, address: u16 ) -> u8 {
        warn!( "Unhandled read from the expansion ROM at 0x{:04X}", address );
        0
    }

    fn poke_expansion_rom( &self, address: u16, value: u8 ) {
        warn!( "Unhandled write to the expansion ROM at 0x{:04X} (value=0x{:02X})", address, value );
    }

    fn peek_video_memory( &self, address: u16 ) -> u8;
    fn poke_video_memory( &mut self, address: u16, value: u8 );
}

pub struct MapperNull;

impl Mapper for MapperNull {
    fn peek_rom( &self, address: u16 ) -> u8 {
        warn!( "Unhandled read from the ROM at 0x{:04X}", address );
        0
    }

    fn poke_rom( &mut self, address: u16, value: u8 ) {
        warn!( "Unhandled write to the ROM at 0x{:04X} (value=0x{:02X})", address, value );
    }

    fn peek_video_memory( &self, address: u16 ) -> u8 {
        warn!( "Unhandled read from the VROM at 0x{:04X}", address );
        0
    }

    fn poke_video_memory( &mut self, address: u16, value: u8 ) {
        warn!( "Unhandled write to the VROM at 0x{:04X} (value=0x{:02X})", address, value );
    }
}

struct MapperNROM {
    rom_data: [u8; ROM_BANK_SIZE * 2],

    // This mapper technically doesn't include any RAM but some homebrew programs depend on it being available.
    save_ram: [u8; 8192],
    pattern_tables: [u8; VROM_BANK_SIZE],
    background_tilemaps: [u8; 2048],

    mirroring: fn( u16 ) -> u16
}

impl Mapper for MapperNROM {
    fn peek_rom( &self, address: u16 ) -> u8 {
        self.rom_data.peek( translate_address_rom( address ) )
    }

    fn poke_rom( &mut self, address: u16, value: u8 ) {
        warn!( "Unhandled write to the ROM at 0x{:04X} (value=0x{:02X})", address, value );
    }

    fn peek_sram( &self, address: u16 ) -> u8 {
        self.save_ram.peek( translate_address_save_ram( address ) )
    }

    fn poke_sram( &mut self, address: u16, value: u8 ) {
        self.save_ram.poke( translate_address_save_ram( address ), value );
    }

    fn peek_video_memory( &self, address: u16 ) -> u8 {
        if address <= 0x1FFF {
            self.pattern_tables.peek( address )
        } else {
            let translated_address = (self.mirroring)( translate_address_background_tilemap( address ) );
            self.background_tilemaps.peek( translated_address )
        }
    }

    fn poke_video_memory( &mut self, address: u16, value: u8 ) {
        if address <= 0x1FFF {
            self.pattern_tables.poke( address, value );
        } else {
            let translated_address = (self.mirroring)( translate_address_background_tilemap( address ) );
            self.background_tilemaps.poke( translated_address, value );
        }
    }
}

pub fn create_mapper( rom: NesRom ) -> Result< Box< Mapper >, LoadError > {
    match rom.mapper {
        0 => {
            let mut rom_data: [u8; ROM_BANK_SIZE * 2] = [0; ROM_BANK_SIZE * 2];
            let mut vrom_data: [u8; VROM_BANK_SIZE] = [0; VROM_BANK_SIZE];
            match rom.rom_banks.len() {
                1 => {
                    copy_memory( &rom.rom_banks[0], &mut rom_data[ 0..ROM_BANK_SIZE ] );
                    copy_memory( &rom.rom_banks[0], &mut rom_data[ ROM_BANK_SIZE.. ] );
                },
                2 => {
                    copy_memory( &rom.rom_banks[0], &mut rom_data[ 0..ROM_BANK_SIZE ] );
                    copy_memory( &rom.rom_banks[1], &mut rom_data[ ROM_BANK_SIZE.. ] );
                },
                _ => return Err( LoadError::new( format!( "Invalid ROM bank count for given mapper; should be 1 or 2, is {}.", rom.rom_banks.len() ) ) )
            }

            match rom.vrom_banks.len() {
                0 => {},
                1 => {
                    copy_memory( &rom.vrom_banks[0], &mut vrom_data[..] );
                },
                _ => return Err( LoadError::new( format!( "Invalid VROM bank count for given mapper; should be 1, is {}.", rom.vrom_banks.len() ) ) )
            }

            let mirroring = match rom.mirroring {
                Mirroring::Horizontal => horizontal_mirroring as fn( u16 ) -> u16,
                Mirroring::Vertical => vertical_mirroring as fn( u16 ) -> u16,
                _ => return Err( LoadError::new( format!( "Invalid mirroring mode for given mapper; should be either Horizontal or Vertical, is {:?}.", rom.mirroring ) ) )
            };

            Ok( Box::new( MapperNROM {
                rom_data: rom_data,
                save_ram: [0; 8192],
                pattern_tables: vrom_data,
                background_tilemaps: [0; 2048],
                mirroring: mirroring
            }))
        },
        1 => {
            MapperMMC1::from_rom( rom ).map( |mapper| {
                let boxed: Box< Mapper > = Box::new( mapper );
                boxed
            })
        },
        _ => Err( LoadError::new( format!( "Unhandled mapper: {}", rom.mapper ) ) )
    }
}
