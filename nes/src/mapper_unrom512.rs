use emumisc::BitExtra;
use rom::{NesRom, LoadError};
use generic_mapper::BankedGenericMapper;
use mappers::Mapper;

pub struct MapperUNROM512 {
    inner: BankedGenericMapper
}

impl MapperUNROM512 {
    pub fn from_rom( rom: NesRom ) -> Result< Self, LoadError > {
        let mut mapper = MapperUNROM512 {
            inner: BankedGenericMapper::from_rom( rom )?
        };

        let last_bank = mapper.inner.last_rom_16k_bank();
        mapper.inner.set_cpu_lower_16k_bank_to_bank( 0 );
        mapper.inner.set_cpu_upper_16k_bank_to_bank( last_bank );

        Ok( mapper )
    }
}

impl Mapper for MapperUNROM512 {
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
        let rom_bank = value.get_bits( 0b0001_1111 );
        let vrom_bank = value.get_bits( 0b0110_0000 );
        self.inner.set_cpu_lower_16k_bank_to_bank( rom_bank );
        self.inner.set_ppu_8k_bank_to_bank( vrom_bank );
        let one_screen_mirroring = value.get_bits( 0b1000_0000 ) != 0;
        if one_screen_mirroring {
            self.inner.set_only_lower_bank_mirroring();
        } else {
            self.inner.set_default_mirroring();
        }
    }

    fn peek_video_memory( &self, address: u16 ) -> u8 {
        self.inner.peek_video_memory( address )
    }

    fn poke_video_memory( &mut self, address: u16, value: u8 ) {
        self.inner.poke_video_memory( address, value )
    }
}
