use rom::{NesRom, LoadError};
use generic_mapper::BankedGenericMapper;
use mappers::Mapper;

// This is a very simple mapper. At 0x8000 - 0xBFFF
// we have a switchable ROM bank, and the 0xC000 - 0xFFFF
// is fixed to the last bank. The lower ROM bank is
// switched by writing to 0x8000 - 0xFFFF.

pub struct MapperUxROM {
    inner: BankedGenericMapper
}

impl MapperUxROM {
    pub fn from_rom( rom: NesRom ) -> Result< Self, LoadError > {
        let mut mapper = MapperUxROM {
            inner: BankedGenericMapper::from_rom( rom )?
        };

        let last_bank = mapper.inner.last_rom_16k_bank();
        mapper.inner.set_cpu_lower_16k_bank_to_bank( 0 );
        mapper.inner.set_cpu_upper_16k_bank_to_bank( last_bank );

        Ok( mapper )
    }
}

impl Mapper for MapperUxROM {
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
        self.inner.set_cpu_lower_16k_bank_to_bank( value );
    }

    fn peek_video_memory( &self, address: u16 ) -> u8 {
        self.inner.peek_video_memory( address )
    }

    fn poke_video_memory( &mut self, address: u16, value: u8 ) {
        self.inner.poke_video_memory( address, value )
    }
}
