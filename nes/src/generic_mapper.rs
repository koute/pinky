use core::fmt;
use alloc::vec::Vec;

use emumisc::{PeekPoke, At};
use rom::Mirroring;
use mappers::Mapper;

// This is a generic memory mapper that is used to simplify actual
// emulated mappers.
pub struct GenericMapper {
    // 0x6000 .. 0xFFFF, 40kb in total, remapped in 8kb chunks
    cpu_offsets: [i32; 5],
    cpu_flags: [MapFlag; 5],

    // 0x0000 .. 0x2FFF, 12kb in total, remapped in 1kb chunks
    ppu_offsets: [i32; 12],
    ppu_flags: [MapFlag; 12],

    // For maximum flexibility we just dump everything into one big vector.
    memory: Vec< u8 >
}

impl fmt::Debug for GenericMapper {
    fn fmt( &self, fmt: &mut fmt::Formatter ) -> fmt::Result {
        fn print_maps( fmt: &mut fmt::Formatter, offsets: &[i32], flags: &[MapFlag], prebaked_offsets: &[i32] ) -> fmt::Result {
            let increment = -(prebaked_offsets[1] - prebaked_offsets[0]) / 1024;
            for ((offset, flags), prebaked) in offsets.iter().zip( flags.iter() ).zip( prebaked_offsets.iter() ) {
                write!( fmt, "        0x{:04X}: ", -prebaked )?;
                if flags.contains( MapFlag::Mapped ) {
                    if flags.contains( MapFlag::Writable ) {
                        write!( fmt, "[w] " )?;
                    } else {
                        write!( fmt, "[-] " )?;
                    }
                    let offset = (offset - prebaked) / 1024;
                    writeln!( fmt, "{}k .. {}k", offset, offset + increment )?;
                } else {
                    writeln!( fmt, "[unmapped]" )?;
                }
            }

            Ok(())
        }

        writeln!( fmt, "GenericMapper {{" )?;
        writeln!( fmt, "    Total memory: {}k,", self.memory.len() / 1024 )?;
        writeln!( fmt, "    CPU {{" )?;
        print_maps( fmt, &self.cpu_offsets[..], &self.cpu_flags[..], &PREBAKED_CPU_OFFSETS[..] )?;
        writeln!( fmt, "    }}," )?;
        writeln!( fmt, "    PPU {{" )?;
        print_maps( fmt, &self.ppu_offsets[..], &self.ppu_flags[..], &PREBAKED_PPU_OFFSETS[..] )?;
        writeln!( fmt, "    }}" )?;
        write!( fmt, "}}" )?;

        Ok(())
    }
}

bitflags!(
    #[derive(Copy, Clone)]
    pub struct MapFlag: u8 {
        const Mapped      = 1 << 0;
        const Writable    = 1 << 1;
    }
);

// We prebake the memory address of a given memory region in our offsets,
// effectively saving us one extra operation per memory access.
const PREBAKED_CPU_OFFSETS: [i32; 5] = [
    -0x6000,
    -0x8000,
    -0xA000,
    -0xC000,
    -0xE000
];

const PREBAKED_PPU_OFFSETS: [i32; 12] = [
    -0x0000,
    -0x0400,
    -0x0800,
    -0x0C00,
    -0x1000,
    -0x1400,
    -0x1800,
    -0x1C00,
    -0x2000,
    -0x2400,
    -0x2800,
    -0x2C00
];

pub mod bank {
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum CPU_8K {
        Ox6000 = 0,
        Ox8000,
        OxA000,
        OxC000,
        OxE000
    }

    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum PPU_1K {
        Ox0000 = 0,
        Ox0400,
        Ox0800,
        Ox0C00,
        Ox1000,
        Ox1400,
        Ox1800,
        Ox1C00,
        Ox2000,
        Ox2400,
        Ox2800,
        Ox2C00
    }

    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum PPU_4K {
        Ox0000 = 0,
        Ox1000 = 4,
        Ox2000 = 8
    }
}

#[inline]
fn get_ppu_bank( address: u16 ) -> u16 {
    debug_assert!( address <= 0x2FFF );
    let bank = address >> 10;
    debug_assert!( bank < PREBAKED_PPU_OFFSETS.len() as u16 );
    bank
}

#[inline]
fn get_cpu_bank( address: u16 ) -> u16 {
    debug_assert!( address >= 0x6000 );
    let bank = (address - 0x6000) >> 13;
    debug_assert!( bank < PREBAKED_CPU_OFFSETS.len() as u16 );
    bank
}

#[test]
fn test_get_cpu_bank() {
    assert_eq!( get_cpu_bank( 0x6000 ), 0 );
    assert_eq!( get_cpu_bank( 0x7FFF ), 0 );
    assert_eq!( get_cpu_bank( 0x8000 ), 1 );
    assert_eq!( get_cpu_bank( 0x9FFF ), 1 );
    assert_eq!( get_cpu_bank( 0xA000 ), 2 );
    assert_eq!( get_cpu_bank( 0xBFFF ), 2 );
    assert_eq!( get_cpu_bank( 0xC000 ), 3 );
    assert_eq!( get_cpu_bank( 0xDFFF ), 3 );
    assert_eq!( get_cpu_bank( 0xE000 ), 4 );
    assert_eq!( get_cpu_bank( 0xFFFF ), 4 );
}

impl GenericMapper {
    pub fn new() -> Self {
        GenericMapper {
            cpu_offsets: PREBAKED_CPU_OFFSETS,
            cpu_flags: [MapFlag::empty(); 5],
            ppu_offsets: PREBAKED_PPU_OFFSETS,
            ppu_flags: [MapFlag::empty(); 12],

            memory: Vec::new()
        }
    }

    #[cfg(test)]
    pub fn memory( &mut self ) -> &[u8] {
        &self.memory[..]
    }

    #[cfg(test)]
    pub fn memory_mut( &mut self ) -> &mut [u8] {
        &mut self.memory[..]
    }

    #[inline]
    pub fn extend( &mut self, memory: &[u8] ) {
        self.memory.extend_from_slice( memory );
    }

    #[inline]
    pub fn extend_empty( &mut self, size: u32 ) {
        let new_size = self.memory.len() + size as usize;
        self.memory.resize( new_size, 0 );
    }

    #[inline]
    pub fn total_memory_size( &self ) -> usize {
        self.memory.len()
    }

    #[inline]
    pub fn initialize_save_ram( &mut self ) {
        let offset = self.memory.len() as u32;
        self.extend_empty( 8 * 1024 );
        self.set_cpu_8k_bank( bank::CPU_8K::Ox6000, offset );
        self.set_cpu_8k_writable( bank::CPU_8K::Ox6000, true );
    }

    #[inline]
    pub fn initialize_rom( &mut self, rom: &[u8] ) -> u32 {
        assert!( rom.len() >= 16 * 1024 );

        let offset = self.memory.len() as u32;
        self.extend( rom );
        self.set_cpu_lower_16k_bank( offset );

        if rom.len() >= 32 * 1024 {
            // We have two independent banks.
            self.set_cpu_upper_16k_bank( offset + 16 * 1024 );
        } else {
            assert_eq!( rom.len(), 16 * 1024 );

            // Only one bank, so we mirror it.
            self.set_cpu_upper_16k_bank( offset );
        }

        offset
    }

    #[inline]
    pub fn initialize_video_rom( &mut self, video_rom: &[u8] ) -> u32 {
        let offset = self.memory.len() as u32;
        if video_rom.len() > 0 {
            self.extend( video_rom );
        } else {
            self.extend_empty( 8 * 1024 );
            self.set_ppu_4k_writable( bank::PPU_4K::Ox0000, true );
            self.set_ppu_4k_writable( bank::PPU_4K::Ox1000, true );
        }

        self.set_ppu_4k_bank( bank::PPU_4K::Ox0000, offset );
        self.set_ppu_4k_bank( bank::PPU_4K::Ox1000, offset + 4 * 1024 );

        offset
    }

    #[inline]
    pub fn initialize_background_tilemaps( &mut self, mirroring: Mirroring ) -> u32 {
        let offset = self.memory.len() as u32;
        match mirroring {
            Mirroring::Horizontal | Mirroring::Vertical => {
                self.extend_empty( 2 * 1024 );
                if mirroring == Mirroring::Horizontal {
                    self.set_horizontal_mirroring( offset );
                } else {
                    self.set_vertical_mirroring( offset );
                }
            },
            Mirroring::FourScreen => {
                self.extend_empty( 4 * 1024 );
                self.set_four_screen_mirroring( offset );
            }
        }

        self.set_ppu_4k_writable( bank::PPU_4K::Ox2000, true );

        offset
    }

    #[inline]
    fn translate_cpu_address( &self, address: u16 ) -> i32 {
        (&self.cpu_offsets[..]).peek( get_cpu_bank( address ) ) + address as i32
    }

    #[inline]
    fn is_cpu_address_writable( &self, address: u16 ) -> bool {
        (&self.cpu_flags[..]).peek( get_cpu_bank( address ) ).contains( MapFlag::Writable )
    }

    #[inline]
    fn is_cpu_address_mapped( &self, address: u16 ) -> bool {
        (&self.cpu_flags[..]).peek( get_cpu_bank( address ) ).contains( MapFlag::Mapped )
    }

    #[inline]
    pub fn set_cpu_8k_bank( &mut self, bank: bank::CPU_8K, internal_address: u32 ) {
        debug_assert!( (internal_address + 8 * 1024) <= self.memory.len() as u32 );
        let bank = bank as usize;
        let new_offset = internal_address as i32 + PREBAKED_CPU_OFFSETS.peek( bank );
        (&mut self.cpu_offsets[..]).poke( bank, new_offset );
        (&mut self.cpu_flags[..]).at_mut( bank ).insert( MapFlag::Mapped );
    }

    #[inline]
    pub fn set_cpu_lower_16k_bank( &mut self, internal_address: u32 ) {
        self.set_cpu_8k_bank( bank::CPU_8K::Ox8000, internal_address );
        self.set_cpu_8k_bank( bank::CPU_8K::OxA000, internal_address + 8 * 1024 );
    }

    #[inline]
    pub fn set_cpu_upper_16k_bank( &mut self, internal_address: u32 ) {
        self.set_cpu_8k_bank( bank::CPU_8K::OxC000, internal_address );
        self.set_cpu_8k_bank( bank::CPU_8K::OxE000, internal_address + 8 * 1024 );
    }

    #[inline]
    pub fn set_cpu_32k_bank( &mut self, internal_address: u32 ) {
        self.set_cpu_8k_bank( bank::CPU_8K::Ox8000, internal_address );
        self.set_cpu_8k_bank( bank::CPU_8K::OxA000, internal_address + 8 * 1024 );
        self.set_cpu_8k_bank( bank::CPU_8K::OxC000, internal_address + 16 * 1024 );
        self.set_cpu_8k_bank( bank::CPU_8K::OxE000, internal_address + 24 * 1024 );
    }

    #[inline]
    pub fn set_cpu_8k_writable( &mut self, bank: bank::CPU_8K, is_writable: bool ) {
        let bank = bank as usize;
        let flags = (&mut self.cpu_flags[..]).at_mut( bank );
        if is_writable {
            flags.insert( MapFlag::Writable );
        } else {
            flags.remove( MapFlag::Writable );
        }
    }

    #[inline]
    fn translate_ppu_address( &self, address: u16 ) -> i32 {
        (&self.ppu_offsets[..]).peek( get_ppu_bank( address ) ) + address as i32
    }

    #[inline]
    fn is_ppu_address_writable( &self, address: u16 ) -> bool {
        (&self.ppu_flags[..]).peek( get_ppu_bank( address ) ).contains( MapFlag::Writable )
    }

    #[inline]
    fn is_ppu_address_mapped( &self, address: u16 ) -> bool {
        (&self.ppu_flags[..]).peek( get_ppu_bank( address ) ).contains( MapFlag::Mapped )
    }

    #[inline]
    pub fn set_ppu_1k_bank( &mut self, bank: bank::PPU_1K, internal_address: u32 ) {
        debug_assert!( (internal_address + 1024) <= self.memory.len() as u32 );
        let bank = bank as usize;
        let new_offset = internal_address as i32 + PREBAKED_PPU_OFFSETS.peek( bank );
        (&mut self.ppu_offsets[..]).poke( bank, new_offset );
        (&mut self.ppu_flags[..]).at_mut( bank ).insert( MapFlag::Mapped );
    }

    #[inline]
    pub fn set_ppu_1k_writable( &mut self, bank: bank::PPU_1K, is_writable: bool ) {
        let bank = bank as usize;
        let flags = (&mut self.ppu_flags[..]).at_mut( bank );
        if is_writable {
            flags.insert( MapFlag::Writable );
        } else {
            flags.remove( MapFlag::Writable );
        }
    }

    #[inline]
    pub fn set_ppu_4k_bank( &mut self, bank: bank::PPU_4K, internal_address: u32 ) {
        match bank {
            bank::PPU_4K::Ox0000 => {
                self.set_ppu_1k_bank( bank::PPU_1K::Ox0000, internal_address );
                self.set_ppu_1k_bank( bank::PPU_1K::Ox0400, internal_address + 0x0400 );
                self.set_ppu_1k_bank( bank::PPU_1K::Ox0800, internal_address + 0x0800 );
                self.set_ppu_1k_bank( bank::PPU_1K::Ox0C00, internal_address + 0x0C00 );
            },
            bank::PPU_4K::Ox1000 => {
                self.set_ppu_1k_bank( bank::PPU_1K::Ox1000, internal_address );
                self.set_ppu_1k_bank( bank::PPU_1K::Ox1400, internal_address + 0x0400 );
                self.set_ppu_1k_bank( bank::PPU_1K::Ox1800, internal_address + 0x0800 );
                self.set_ppu_1k_bank( bank::PPU_1K::Ox1C00, internal_address + 0x0C00 );
            },
            bank::PPU_4K::Ox2000 => {
                self.set_ppu_1k_bank( bank::PPU_1K::Ox2000, internal_address );
                self.set_ppu_1k_bank( bank::PPU_1K::Ox2400, internal_address + 0x0400 );
                self.set_ppu_1k_bank( bank::PPU_1K::Ox2800, internal_address + 0x0800 );
                self.set_ppu_1k_bank( bank::PPU_1K::Ox2C00, internal_address + 0x0C00 );
            }
        }
    }

    #[inline]
    pub fn set_ppu_4k_writable( &mut self, bank: bank::PPU_4K, is_writable: bool ) {
        match bank {
            bank::PPU_4K::Ox0000 => {
                self.set_ppu_1k_writable( bank::PPU_1K::Ox0000, is_writable );
                self.set_ppu_1k_writable( bank::PPU_1K::Ox0400, is_writable );
                self.set_ppu_1k_writable( bank::PPU_1K::Ox0800, is_writable );
                self.set_ppu_1k_writable( bank::PPU_1K::Ox0C00, is_writable );
            },
            bank::PPU_4K::Ox1000 => {
                self.set_ppu_1k_writable( bank::PPU_1K::Ox1000, is_writable );
                self.set_ppu_1k_writable( bank::PPU_1K::Ox1400, is_writable );
                self.set_ppu_1k_writable( bank::PPU_1K::Ox1800, is_writable );
                self.set_ppu_1k_writable( bank::PPU_1K::Ox1C00, is_writable );
            },
            bank::PPU_4K::Ox2000 => {
                self.set_ppu_1k_writable( bank::PPU_1K::Ox2000, is_writable );
                self.set_ppu_1k_writable( bank::PPU_1K::Ox2400, is_writable );
                self.set_ppu_1k_writable( bank::PPU_1K::Ox2800, is_writable );
                self.set_ppu_1k_writable( bank::PPU_1K::Ox2C00, is_writable );
            }
        }
    }

    #[inline]
    pub fn set_ppu_lower_4k_bank( &mut self, internal_address: u32 ) {
        self.set_ppu_4k_bank( bank::PPU_4K::Ox0000, internal_address );
    }

    #[inline]
    pub fn set_ppu_upper_4k_bank( &mut self, internal_address: u32 ) {
        self.set_ppu_4k_bank( bank::PPU_4K::Ox1000, internal_address );
    }

    #[inline]
    pub fn set_ppu_8k_bank( &mut self, internal_address: u32 ) {
        self.set_ppu_4k_bank( bank::PPU_4K::Ox0000, internal_address );
        self.set_ppu_4k_bank( bank::PPU_4K::Ox1000, internal_address + 0x1000 );
    }

    #[inline]
    pub fn set_horizontal_mirroring( &mut self, internal_address: u32 ) {
        self.set_ppu_1k_bank( bank::PPU_1K::Ox2000, internal_address );
        self.set_ppu_1k_bank( bank::PPU_1K::Ox2400, internal_address );
        self.set_ppu_1k_bank( bank::PPU_1K::Ox2800, internal_address + 0x0400 );
        self.set_ppu_1k_bank( bank::PPU_1K::Ox2C00, internal_address + 0x0400 );
    }

    #[inline]
    pub fn set_vertical_mirroring( &mut self, internal_address: u32 ) {
        self.set_ppu_1k_bank( bank::PPU_1K::Ox2000, internal_address );
        self.set_ppu_1k_bank( bank::PPU_1K::Ox2400, internal_address + 0x0400 );
        self.set_ppu_1k_bank( bank::PPU_1K::Ox2800, internal_address );
        self.set_ppu_1k_bank( bank::PPU_1K::Ox2C00, internal_address + 0x0400 );
    }

    #[inline]
    pub fn set_only_lower_bank_mirroring( &mut self, internal_address: u32 ) {
        self.set_ppu_1k_bank( bank::PPU_1K::Ox2000, internal_address );
        self.set_ppu_1k_bank( bank::PPU_1K::Ox2400, internal_address );
        self.set_ppu_1k_bank( bank::PPU_1K::Ox2800, internal_address );
        self.set_ppu_1k_bank( bank::PPU_1K::Ox2C00, internal_address );
    }

    #[inline]
    pub fn set_only_upper_bank_mirroring( &mut self, internal_address: u32 ) {
        self.set_ppu_1k_bank( bank::PPU_1K::Ox2000, internal_address + 0x0400 );
        self.set_ppu_1k_bank( bank::PPU_1K::Ox2400, internal_address + 0x0400 );
        self.set_ppu_1k_bank( bank::PPU_1K::Ox2800, internal_address + 0x0400 );
        self.set_ppu_1k_bank( bank::PPU_1K::Ox2C00, internal_address + 0x0400 );
    }

    #[inline]
    pub fn set_four_screen_mirroring( &mut self, internal_address: u32 ) {
        if internal_address + 4 * 1024 > self.memory.len() as u32 {
            let extra_space = internal_address + 4 * 1024 - self.memory.len() as u32;
            self.extend_empty( extra_space );
        }

        self.set_ppu_4k_bank( bank::PPU_4K::Ox2000, internal_address );
    }

    #[inline]
    pub fn peek_cpu_memory_space( &self, address: u16 ) -> u8 {
        let actual_address = self.translate_cpu_address( address );
        self.memory.peek( actual_address )
    }

    #[inline]
    pub fn poke_cpu_memory_space( &mut self, address: u16, value: u8 ) {
        if self.is_cpu_address_writable( address ) == false {
            #[cfg(feature = "log")]
            warn!( "Unhandled write to 0x{:04X} (value=0x{:02X})", address, value );
            return;
        }

        let actual_address = self.translate_cpu_address( address );
        self.memory.poke( actual_address, value )
    }

    #[inline]
    pub fn peek_ppu_memory_space( &self, address: u16 ) -> u8 {
        let address = if address >= 0x3000 { address - 0x1000 } else { address };
        let actual_address = self.translate_ppu_address( address );
        self.memory.peek( actual_address )
    }

    #[inline]
    pub fn poke_ppu_memory_space( &mut self, address: u16, value: u8 ) {
        let address = if address >= 0x3000 { address - 0x1000 } else { address };
        if self.is_ppu_address_writable( address ) == false {
            return;
        }

        let actual_address = self.translate_ppu_address( address );
        self.memory.poke( actual_address, value )
    }
}

impl Mapper for GenericMapper {
    #[inline]
    fn peek_rom( &self, address: u16 ) -> u8 {
        self.peek_cpu_memory_space( address )
    }

    #[inline]
    fn poke_rom( &mut self, address: u16, value: u8 ) {
        self.poke_cpu_memory_space( address, value )
    }

    #[inline]
    fn peek_sram( &self, address: u16 ) -> u8 {
        self.peek_cpu_memory_space( address )
    }

    #[inline]
    fn poke_sram( &mut self, address: u16, value: u8 ) {
        self.poke_cpu_memory_space( address, value )
    }

    #[inline]
    fn peek_video_memory( &self, address: u16 ) -> u8 {
        self.peek_ppu_memory_space( address )
    }

    #[inline]
    fn poke_video_memory( &mut self, address: u16, value: u8 ) {
        self.poke_ppu_memory_space( address, value )
    }
}

use core::ops::Sub;
use rom::{NesRom, LoadError};
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

// This is an even more simplified generic mapper.
pub struct BankedGenericMapper {
    inner: GenericMapper,

    internal_rom_bank_offset: u32,
    internal_video_rom_offset: u32,
    internal_background_tilemaps_offset: u32,

    rom_size: u32,
    video_rom_size: u32,
    default_mirroring: Mirroring
}

impl BankedGenericMapper {
    fn empty() -> Self {
        BankedGenericMapper {
            inner: GenericMapper::new(),

            internal_rom_bank_offset: 0,
            internal_video_rom_offset: 0,
            internal_background_tilemaps_offset: 0,

            rom_size: 0,
            video_rom_size: 0,
            default_mirroring: Mirroring::Horizontal
        }
    }

    pub fn from_rom( rom: NesRom ) -> Result< Self, LoadError > {
        let mut mapper = Self::empty();
        mapper.inner.initialize_save_ram();
        mapper.internal_rom_bank_offset = mapper.inner.initialize_rom( &rom.rom[..] );
        mapper.internal_video_rom_offset = mapper.inner.initialize_video_rom( &rom.video_rom[..] );
        mapper.internal_background_tilemaps_offset = mapper.inner.initialize_background_tilemaps( rom.mirroring );

        mapper.rom_size = mapper.internal_video_rom_offset - mapper.internal_rom_bank_offset;
        mapper.video_rom_size = mapper.internal_background_tilemaps_offset - mapper.internal_video_rom_offset;
        mapper.default_mirroring = rom.mirroring;

        assert!( mapper.rom_size > 0 );
        assert!( mapper.video_rom_size > 0 );

        Ok( mapper )
    }

    #[cfg(test)]
    pub fn memory( &mut self ) -> &[u8] {
        self.inner.memory()
    }

    #[cfg(test)]
    pub fn memory_mut( &mut self ) -> &mut [u8] {
        self.inner.memory_mut()
    }

    #[cfg(test)]
    pub fn internal_video_rom_offset( &self ) -> u32 {
        self.internal_video_rom_offset
    }

    #[inline]
    pub fn last_rom_16k_bank( &self ) -> u8 {
        (self.rom_size / 16 * 1024 - 1) as u8
    }

    #[inline]
    pub fn rom_16k_bank_count( &self ) -> u8 {
        (self.rom_size / (16 * 1024)) as u8
    }

    #[inline]
    pub fn rom_32k_bank_count( &self ) -> u8 {
        (self.rom_size / (32 * 1024)) as u8
    }

    #[inline]
    pub fn video_rom_4k_bank_count( &self ) -> u8 {
        (self.video_rom_size / (4 * 1024)) as u8
    }

    #[inline]
    pub fn video_rom_8k_bank_count( &self ) -> u8 {
        (self.video_rom_size / (8 * 1024)) as u8
    }

    #[inline]
    pub fn set_cpu_lower_16k_bank_to_bank( &mut self, bank: u8 ) {
        let bank = wraparound( self.rom_16k_bank_count(), bank ) as u32;
        self.inner.set_cpu_lower_16k_bank( self.internal_rom_bank_offset + bank * 16 * 1024 );
    }

    #[inline]
    pub fn set_cpu_upper_16k_bank_to_bank( &mut self, bank: u8 ) {
        let bank = wraparound( self.rom_16k_bank_count(), bank ) as u32;
        self.inner.set_cpu_upper_16k_bank( self.internal_rom_bank_offset + bank * 16 * 1024 );
    }

    #[inline]
    pub fn set_cpu_32k_bank_to_bank( &mut self, bank: u8 ) {
        let bank = wraparound( self.rom_32k_bank_count(), bank ) as u32;
        self.inner.set_cpu_32k_bank( self.internal_rom_bank_offset + bank * 32 * 1024 );
    }

    #[inline]
    pub fn set_ppu_lower_4k_bank_to_bank( &mut self, bank: u8 ) {
        let bank = wraparound( self.video_rom_4k_bank_count(), bank ) as u32;
        self.inner.set_ppu_lower_4k_bank( self.internal_video_rom_offset + bank * 4 * 1024 );
    }

    #[inline]
    pub fn set_ppu_upper_4k_bank_to_bank( &mut self, bank: u8 ) {
        let bank = wraparound( self.video_rom_4k_bank_count(), bank ) as u32;
        self.inner.set_ppu_upper_4k_bank( self.internal_video_rom_offset + bank * 4 * 1024 );
    }

    #[inline]
    pub fn set_ppu_8k_bank_to_bank( &mut self, bank: u8 ) {
        let bank = wraparound( self.video_rom_8k_bank_count(), bank ) as u32;
        self.inner.set_ppu_8k_bank( self.internal_video_rom_offset + bank * 8 * 1024 );
    }

    #[inline]
    pub fn set_horizontal_mirroring( &mut self ) {
        self.inner.set_horizontal_mirroring( self.internal_background_tilemaps_offset );
    }

    #[inline]
    pub fn set_vertical_mirroring( &mut self ) {
        self.inner.set_vertical_mirroring( self.internal_background_tilemaps_offset );
    }

    #[inline]
    pub fn set_only_lower_bank_mirroring( &mut self ) {
        self.inner.set_only_lower_bank_mirroring( self.internal_background_tilemaps_offset );
    }

    #[inline]
    pub fn set_only_upper_bank_mirroring( &mut self ) {
        self.inner.set_only_upper_bank_mirroring( self.internal_background_tilemaps_offset );
    }

    #[inline]
    pub fn set_four_screen_mirroring( &mut self ) {
        self.inner.set_four_screen_mirroring( self.internal_background_tilemaps_offset );
    }

    #[inline]
    pub fn set_default_mirroring( &mut self ) {
        match self.default_mirroring {
            Mirroring::Horizontal => self.set_horizontal_mirroring(),
            Mirroring::Vertical => self.set_vertical_mirroring(),
            Mirroring::FourScreen => self.set_four_screen_mirroring()
        }
    }

    #[inline]
    pub fn set_cpu_8k_writable( &mut self, bank: bank::CPU_8K, is_writable: bool ) {
        self.inner.set_cpu_8k_writable( bank, is_writable );
    }

    #[inline]
    pub fn set_ppu_4k_writable( &mut self, bank: bank::PPU_4K, is_writable: bool ) {
        self.inner.set_ppu_4k_writable( bank, is_writable );
    }
}

impl Mapper for BankedGenericMapper {
    #[inline]
    fn peek_rom( &self, address: u16 ) -> u8 {
        self.inner.peek_cpu_memory_space( address )
    }

    #[inline]
    fn poke_rom( &mut self, address: u16, value: u8 ) {
        self.inner.poke_cpu_memory_space( address, value )
    }

    #[inline]
    fn peek_sram( &self, address: u16 ) -> u8 {
        self.inner.peek_cpu_memory_space( address )
    }

    #[inline]
    fn poke_sram( &mut self, address: u16, value: u8 ) {
        self.inner.poke_cpu_memory_space( address, value )
    }

    #[inline]
    fn peek_video_memory( &self, address: u16 ) -> u8 {
        self.inner.peek_ppu_memory_space( address )
    }

    #[inline]
    fn poke_video_memory( &mut self, address: u16, value: u8 ) {
        self.inner.poke_ppu_memory_space( address, value )
    }
}

#[test]
fn test_generic_mapper_cpu_banks() {
    let mut mapper = GenericMapper::new();
    mapper.extend_empty( 40 * 1024 );
    mapper.set_cpu_8k_bank( bank::CPU_8K::Ox8000, 8 * 1024 );
    mapper.set_cpu_8k_bank( bank::CPU_8K::OxA000, 16 * 1024 );
    mapper.set_cpu_8k_bank( bank::CPU_8K::OxC000, 24 * 1024 );
    mapper.set_cpu_8k_bank( bank::CPU_8K::OxE000, 32 * 1024 );
    mapper.memory[ 8 * 1024 + 0 ] = 1;
    mapper.memory[ 8 * 1024 + 1 ] = 2;
    mapper.memory[ 16 * 1024 + 0 ] = 3;
    mapper.memory[ 16 * 1024 + 1 ] = 4;
    mapper.memory[ 24 * 1024 + 0 ] = 5;
    mapper.memory[ 24 * 1024 + 1 ] = 6;
    mapper.memory[ 32 * 1024 + 0 ] = 7;
    mapper.memory[ 32 * 1024 + 1 ] = 8;

    assert_eq!( mapper.peek_rom( 0x8000 ), 1 );
    assert_eq!( mapper.peek_rom( 0x8001 ), 2 );
    assert_eq!( mapper.peek_rom( 0xA000 ), 3 );
    assert_eq!( mapper.peek_rom( 0xA001 ), 4 );
    assert_eq!( mapper.peek_rom( 0xC000 ), 5 );
    assert_eq!( mapper.peek_rom( 0xC001 ), 6 );
    assert_eq!( mapper.peek_rom( 0xE000 ), 7 );
    assert_eq!( mapper.peek_rom( 0xE001 ), 8 );

    mapper.set_cpu_8k_writable( bank::CPU_8K::Ox8000, true );
    mapper.set_cpu_8k_writable( bank::CPU_8K::OxA000, true );
    mapper.set_cpu_8k_writable( bank::CPU_8K::OxC000, true );
    mapper.set_cpu_8k_writable( bank::CPU_8K::OxE000, true );
    mapper.poke_rom( 0x8000, 11 );
    mapper.poke_rom( 0xA000, 22 );
    mapper.poke_rom( 0xC000, 33 );
    mapper.poke_rom( 0xE000, 44 );

    assert_eq!( mapper.memory[ 8 * 1024 ], 11 );
    assert_eq!( mapper.memory[ 16 * 1024 ], 22 );
    assert_eq!( mapper.memory[ 24 * 1024 ], 33 );
    assert_eq!( mapper.memory[ 32 * 1024 ], 44 );

    mapper.set_cpu_8k_writable( bank::CPU_8K::Ox8000, false );
    mapper.poke_rom( 0x8000, 255 );
    assert_eq!( mapper.memory[ 8 * 1024 ], 11 );
}

#[test]
fn test_generic_mapper_ppu_banks() {
    let mut mapper = GenericMapper::new();
    mapper.extend_empty( 40 * 1024 );

    mapper.set_ppu_1k_bank( bank::PPU_1K::Ox0400, 1 * 1024 );
    mapper.set_ppu_1k_bank( bank::PPU_1K::Ox2000, 2 * 1024 );
    mapper.set_ppu_1k_bank( bank::PPU_1K::Ox2400, 3 * 1024 );
    mapper.set_ppu_1k_bank( bank::PPU_1K::Ox2800, 4 * 1024 );
    mapper.set_ppu_1k_bank( bank::PPU_1K::Ox2C00, 5 * 1024 );

    mapper.memory[ 1 * 1024 + 0 ] = 1;
    mapper.memory[ 1 * 1024 + 1 ] = 2;
    mapper.memory[ 2 * 1024 + 0 ] = 3;
    mapper.memory[ 2 * 1024 + 1 ] = 4;
    mapper.memory[ 3 * 1024 + 0 ] = 5;
    mapper.memory[ 3 * 1024 + 1 ] = 6;
    mapper.memory[ 4 * 1024 + 0 ] = 7;
    mapper.memory[ 4 * 1024 + 1 ] = 8;
    mapper.memory[ 5 * 1024 + 0 ] = 9;
    mapper.memory[ 5 * 1024 + 1 ] = 10;

    assert_eq!( mapper.peek_video_memory( 0x0400 ), 1 );
    assert_eq!( mapper.peek_video_memory( 0x0401 ), 2 );
    assert_eq!( mapper.peek_video_memory( 0x2000 ), 3 );
    assert_eq!( mapper.peek_video_memory( 0x2001 ), 4 );
    assert_eq!( mapper.peek_video_memory( 0x2400 ), 5 );
    assert_eq!( mapper.peek_video_memory( 0x2401 ), 6 );
    assert_eq!( mapper.peek_video_memory( 0x2800 ), 7 );
    assert_eq!( mapper.peek_video_memory( 0x2801 ), 8 );
    assert_eq!( mapper.peek_video_memory( 0x2C00 ), 9 );
    assert_eq!( mapper.peek_video_memory( 0x2C01 ), 10 );

    assert_eq!( mapper.peek_video_memory( 0x3000 ), 3 );
    assert_eq!( mapper.peek_video_memory( 0x3001 ), 4 );
    assert_eq!( mapper.peek_video_memory( 0x3400 ), 5 );
    assert_eq!( mapper.peek_video_memory( 0x3401 ), 6 );
    assert_eq!( mapper.peek_video_memory( 0x3800 ), 7 );
    assert_eq!( mapper.peek_video_memory( 0x3801 ), 8 );
    assert_eq!( mapper.peek_video_memory( 0x3C00 ), 9 );
    assert_eq!( mapper.peek_video_memory( 0x3C01 ), 10 );

    mapper.set_horizontal_mirroring( 2 * 1024 );
    assert_eq!( mapper.peek_video_memory( 0x2000 ), 3 );
    assert_eq!( mapper.peek_video_memory( 0x2001 ), 4 );
    assert_eq!( mapper.peek_video_memory( 0x2400 ), 3 );
    assert_eq!( mapper.peek_video_memory( 0x2401 ), 4 );
    assert_eq!( mapper.peek_video_memory( 0x2800 ), 5 );
    assert_eq!( mapper.peek_video_memory( 0x2801 ), 6 );
    assert_eq!( mapper.peek_video_memory( 0x2C00 ), 5 );
    assert_eq!( mapper.peek_video_memory( 0x2C01 ), 6 );

    mapper.set_vertical_mirroring( 2 * 1024 );
    assert_eq!( mapper.peek_video_memory( 0x2000 ), 3 );
    assert_eq!( mapper.peek_video_memory( 0x2001 ), 4 );
    assert_eq!( mapper.peek_video_memory( 0x2400 ), 5 );
    assert_eq!( mapper.peek_video_memory( 0x2401 ), 6 );
    assert_eq!( mapper.peek_video_memory( 0x2800 ), 3 );
    assert_eq!( mapper.peek_video_memory( 0x2801 ), 4 );
    assert_eq!( mapper.peek_video_memory( 0x2C00 ), 5 );
    assert_eq!( mapper.peek_video_memory( 0x2C01 ), 6 );
}
