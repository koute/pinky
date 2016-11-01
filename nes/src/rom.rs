use std::io;
use std::io::{Read, Error, ErrorKind, Seek, SeekFrom};
use std::cmp::max;
use std::fmt;
use std::error;

use byteorder::{ReadBytesExt, LittleEndian};

fn fill_array< T: Read >( fp: &mut T, out: &mut [u8] ) -> io::Result<()> {
    match fp.read( out ) {
        Ok( size ) => {
            if size == out.len() {
                Ok(())
            } else {
                Err( Error::new( ErrorKind::Other, "Unexpected end of file found" ) )
            }
        },
        Err( error ) => Err( error )
    }
}

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
    IO( io::Error )
}

impl LoadError {
    pub fn new< M: Into< String > >( msg: M ) -> LoadError {
        LoadError::Custom( msg.into() )
    }
}

impl fmt::Display for LoadError {
    fn fmt( &self, fmt: &mut fmt::Formatter ) -> fmt::Result {
        match *self {
            LoadError::Custom( ref message ) => try!( write!( fmt, "Unable to load ROM - {}", decapitalize( message ))),
            LoadError::IO( ref error ) => try!( write!( fmt, "Unable to load ROM - {}", decapitalize( error ))),
        }

        Ok(())
    }
}

impl error::Error for LoadError {
    fn description( &self ) -> &str {
        match *self {
            LoadError::Custom( ref message ) => &message[..],
            LoadError::IO( ref error ) => error.description()
        }
    }
}

impl From< io::Error > for LoadError {
    fn from( error: io::Error ) -> LoadError {
        LoadError::IO( error )
    }
}

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
        try!( write!( fmt, "<NesRom mapper={}, rom={}k, video_rom={}k, ram={}k, mirroring={:?}>",
            self.mapper,
            self.rom.len() / 1024,
            self.video_rom.len() / 1024,
            self.save_ram_length / 1024,
            self.mirroring
        ));

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
    pub fn load< T: Read + Seek >( fp: &mut T ) -> Result< Self, LoadError > {
        let magic = try!( fp.read_u32::< LittleEndian >() );

        if magic != 0x1a53454e {
            return Err( LoadError::new( format!( "Not an INES ROM file: magic number mismatch (got: 0x{:08X})", magic ) ) );
        }

        let rom_bank_count = try!( fp.read_u8() ) as usize;
        let video_rom_bank_count = try!( fp.read_u8() ) as usize;
        let flags_1 = try!( fp.read_u8() );
        let flags_2 = try!( fp.read_u8() );

        // For compatibility with older INES files we assume there must be always one RAM bank.
        let save_ram_length = max( 1, try!( fp.read_u8() ) as u32 ) * 8 * 1024;

        // Skip padding.
        try!( fp.seek( SeekFrom::Current(7) ) );

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

        let mut rom = Vec::< u8 >::with_capacity( rom_bank_count * ROM_BANK_SIZE );
        let mut video_rom = Vec::< u8 >::with_capacity( video_rom_bank_count * VROM_BANK_SIZE );

        unsafe {
            rom.set_len( rom_bank_count * ROM_BANK_SIZE );
            video_rom.set_len( video_rom_bank_count * VROM_BANK_SIZE );
        }

        if has_trainer {
            try!( fp.seek( SeekFrom::Current( 512 ) ) ); // Skip trainer.
        }

        try!( fp.read_exact( &mut rom[..] ) );
        try!( fp.read_exact( &mut video_rom[..] ) );

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
