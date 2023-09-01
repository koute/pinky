use emumisc::BitExtra;
use rom::{NesRom, LoadError};
use generic_mapper::BankedGenericMapper;
use mappers::Mapper;

pub struct MapperAxROM {
    inner: BankedGenericMapper
}

impl MapperAxROM {
    pub fn from_rom( rom: NesRom ) -> Result< Self, LoadError > {
        let mut mapper = MapperAxROM {
            inner: BankedGenericMapper::from_rom( rom )?
        };

        mapper.inner.set_cpu_32k_bank_to_bank( 0 );
        mapper.inner.set_only_lower_bank_mirroring();

        Ok( mapper )
    }
}

impl Mapper for MapperAxROM {
    fn peek_sram( &self, address: u16 ) -> u8 {
        self.inner.peek_sram( address )
    }

    fn poke_sram( &mut self, address: u16, value: u8 ) {
        self.inner.poke_sram( address, value )
    }

    fn peek_rom( &self, address: u16 ) -> u8 {
        self.inner.peek_rom( address )
    }

    fn poke_rom( &mut self, _: u16, value: u8 ) {
        let rom_bank = value.get_bits( 0b0000_0111 );
        let tilemap_bank = value.get_bits( 0b0001_0000 );

        self.inner.set_cpu_32k_bank_to_bank( rom_bank );
        if tilemap_bank == 0 {
            self.inner.set_only_lower_bank_mirroring();
        } else {
            self.inner.set_only_upper_bank_mirroring();
        }
    }

    fn peek_video_memory( &self, address: u16 ) -> u8 {
        self.inner.peek_video_memory( address )
    }

    fn poke_video_memory( &mut self, address: u16, value: u8 ) {
        self.inner.poke_video_memory( address, value )
    }
}
