use rom::{NesRom, LoadError};
use mapper_nrom::MapperNROM;
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

pub fn create_mapper( rom: NesRom ) -> Result< Box< Mapper >, LoadError > {
    match rom.mapper {
        0 => {
            MapperNROM::from_rom( rom ).map( |mapper| {
                let boxed: Box< Mapper > = Box::new( mapper );
                boxed
            })
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
