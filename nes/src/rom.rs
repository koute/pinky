use core::cmp::max;
use core::fmt;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

fn decapitalize< M: fmt::Display >( msg: M ) -> String {
    let message = format!( "{}", msg );
    let mut out = String::new();
    for character in message.chars().take(1).flat_map( |character| character.to_lowercase() ).chain( message.chars().skip(1) ) {
        out.push( character );
    }

    out
}

#[derive(Debug)]
pub enum LoadError {
    Custom( String ),
}

impl LoadError {
    pub fn new< M: Into< String > >( msg: M ) -> LoadError {
        LoadError::Custom( msg.into() )
    }
}

impl fmt::Display for LoadError {
    fn fmt( &self, fmt: &mut fmt::Formatter ) -> fmt::Result {
        match *self {
            LoadError::Custom( ref message ) => write!( fmt, "Unable to load ROM - {}", decapitalize( message ))?,
        }

        Ok(())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for LoadError {}

pub const ROM_BANK_SIZE: usize = 16 * 1024;
pub const VROM_BANK_SIZE: usize = 8 * 1024;

pub struct NesRom {
    pub mapper: u8,
    pub rom: Vec< u8 >,
    pub video_rom: Vec< u8 >,
    pub save_ram_length: u32,
    pub mirroring: Mirroring
}

impl fmt::Debug for NesRom {
    fn fmt( &self, fmt: &mut fmt::Formatter ) -> fmt::Result {
        write!( fmt, "<NesRom mapper={}, rom={}k, video_rom={}k, ram={}k, mirroring={:?}>",
            self.mapper,
            self.rom.len() / 1024,
            self.video_rom.len() / 1024,
            self.save_ram_length / 1024,
            self.mirroring
        )?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen
}

impl NesRom {
    pub fn load( mut data: &[u8] ) -> Result< Self, LoadError > {
        if data.len() < 16 {
            return Err( LoadError::new( "unexpected end of file" ) );
        }

        let magic = u32::from_le_bytes( [data[0], data[1], data[2], data[3]] );
        if magic != 0x1a53454e {
            return Err( LoadError::new( format!( "Not an INES ROM file: magic number mismatch (got: 0x{:08X})", magic ) ) );
        }

        let rom_bank_count = data[ 4 ] as usize;
        let video_rom_bank_count = data[ 5 ] as usize;
        let flags_1 = data[ 6 ];
        let flags_2 = data[ 7 ];

        // For compatibility with older INES files we assume there must be always one RAM bank.
        let save_ram_length = max( 1, data[ 8 ] as u32 ) * 8 * 1024;

        let mirroring = {
            if flags_1 & 0b1000 != 0 {
                Mirroring::FourScreen
            } else if flags_1 & 0b1 == 0 {
                Mirroring::Horizontal
            } else {
                Mirroring::Vertical
            }
        };

        let has_trainer = flags_1 & 0b100 != 0;
        let mapper = (flags_2 & 0xF0) | ((flags_1 & 0xF0) >> 4);

        let rom_size = rom_bank_count * ROM_BANK_SIZE;
        let video_rom_size = video_rom_bank_count * VROM_BANK_SIZE;

        data = &data[ 16.. ];
        if has_trainer {
            if data.len() < 512 {
                return Err( LoadError::new( "unexpected end of file" ) );
            }
            data = &data[ 512.. ]; // Skip trainer.
        }

        if data.len() < rom_size + video_rom_size {
            return Err( LoadError::new( "unexpected end of file" ) );
        }

        let rom = data[ ..rom_size ].to_vec();
        let video_rom = data[ rom_size..rom_size + video_rom_size ].to_vec();

        Ok( NesRom {
            mapper: mapper,
            rom: rom,
            video_rom: video_rom,
            save_ram_length: save_ram_length,
            mirroring: mirroring
        })
    }

    pub fn rom_bank_count( &self ) -> usize {
        self.rom.len() / ROM_BANK_SIZE
    }

    pub fn video_rom_bank_count( &self ) -> usize {
        self.video_rom.len() / VROM_BANK_SIZE
    }

    pub fn check_rom_bank_count( &self, possibilities: &[usize] ) -> Result< (), LoadError > {
        if possibilities.contains( &self.rom_bank_count() ) == false {
            let error = LoadError::new(
                format!( "Unsupported ROM bank count; is {}, should be: {:?}.", self.rom_bank_count(), possibilities )
            );

            Err( error )
        } else {
            Ok(())
        }
    }

    pub fn check_video_rom_bank_count( &self, possibilities: &[usize] ) -> Result< (), LoadError > {
        if possibilities.contains( &self.video_rom_bank_count() ) == false {
            let error = LoadError::new(
                format!( "Unsupported VROM bank count; is {}, should be: {:?}.", self.video_rom_bank_count(), possibilities )
            );

            Err( error )
        } else {
            Ok(())
        }
    }
}
