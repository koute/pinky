use std::path::Path;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek};
use std::mem;

use mos6502;
use rp2c02;
use virtual_apu;
use dma;
use mappers::{Mapper, MapperNull, create_mapper};
use rom::{NesRom, LoadError};
use emumisc::{WrappingExtra, PeekPoke, copy_memory};
use memory_map::{translate_address_ram, translate_address_ioreg_ppu, translate_address_ioreg_other};

pub trait Context: Sized {
    fn state_mut( &mut self ) -> &mut State;
    fn state( &self ) -> &State;

    fn on_cycle( &mut self ) {}
    fn on_frame( &mut self ) {}
    fn on_audio_sample( &mut self, _: f32 ) {}
    fn on_audio_frame( &mut self ) {}
}

pub trait Interface: Sized + Context {
    fn load_rom< T: Read + Seek >( &mut self, stream: &mut T ) -> Result< (), LoadError > {
        Private::load_rom( self, stream )
    }

    fn load_rom_from_memory( &mut self, buffer: &[u8] ) -> Result< (), LoadError > {
        Private::load_rom_from_memory( self, buffer )
    }

    fn load_rom_from_file< P: AsRef< Path >>( &mut self, path: P ) -> Result< (), LoadError > {
        Private::load_rom_from_file( self, path )
    }

    fn hard_reset( &mut self ) {
        Private::hard_reset( self )
    }

    fn soft_reset( &mut self ) {
        Private::soft_reset( self )
    }

    fn copy_into_memory( &mut self, offset: u16, data: &[u8] ) {
        Private::copy_into_memory( self, offset, data )
    }

    fn execute_until_vblank( &mut self ) -> Result< (), Box< Error > > {
        Private::execute_until_vblank( self )
    }

    fn execute_for_a_frame( &mut self ) -> Result< (), Box< Error > > {
        Private::execute_for_a_frame( self )
    }

    fn execute_cycle( &mut self ) -> Result< bool, Box< Error > > {
        Private::execute_cycle( self )
    }

    fn framebuffer( &self ) -> &rp2c02::Framebuffer {
        Private::framebuffer( self )
    }

    fn swap_framebuffer( &mut self, other: rp2c02::Framebuffer ) -> rp2c02::Framebuffer {
        Private::swap_framebuffer( self, other )
    }

    fn set_button_state( &mut self, port: ControllerPort, button: Button::Ty, is_pressed: bool ) {
        Private::set_button_state( self, port, button, is_pressed )
    }

    fn press( &mut self, port: ControllerPort, button: Button::Ty ) {
        Private::press( self, port, button )
    }

    fn release( &mut self, port: ControllerPort, button: Button::Ty ) {
        Private::release( self, port, button )
    }

    fn peek_memory( &mut self, address: u16 ) -> u8 {
        Private::peek_memory( self, address )
    }

    fn poke_memory( &mut self, address: u16, value: u8 ) {
        Private::poke_memory( self, address, value )
    }
}

impl< T: Context > Interface for T {}
impl< T: Context > Private for T {}

pub struct State {
    ram: [u8; 2048],
    cpu_state: mos6502::State,
    ppu_state: rp2c02::State,
    apu_state: virtual_apu::State,
    dma_state: dma::State,
    mapper: Box< Mapper >,
    error: Option< Box< Error > >,
    ready: bool,
    cpu_cycle: u32,
    frame_counter: u32,
    full_frame_counter: u32,
    audio_samples_counter: u32,
    gamepad_1: Button::Ty,
    gamepad_2: Button::Ty,
    gamepad_shift_register_1: u8,
    gamepad_shift_register_2: u8,
    gamepad_shift_register_update: bool
}

impl State {
    pub fn new() -> State {
        State {
            ram: [0; 2048],
            cpu_state: mos6502::State::new(),
            ppu_state: rp2c02::State::new(),
            apu_state: virtual_apu::State::new(),
            dma_state: dma::State::new(),
            mapper: Box::new( MapperNull ),
            error: None,
            ready: false,
            cpu_cycle: 0,
            frame_counter: 0,
            full_frame_counter: 0,
            audio_samples_counter: 0,
            gamepad_1: Button::Ty::empty(),
            gamepad_2: Button::Ty::empty(),
            gamepad_shift_register_1: 0,
            gamepad_shift_register_2: 0,
            gamepad_shift_register_update: false
        }
    }

    #[inline]
    pub fn framebuffer( &self ) -> &rp2c02::Framebuffer {
        self.ppu_state.framebuffer()
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ControllerPort {
    First,
    Second
}

// These values are deliberately picked to be the same as the ones in NES' input registers.
pub mod Button {
    bitflags! {
        pub flags Ty: u8 {
            const A          = 1 << 0,
            const B          = 1 << 1,
            const Select     = 1 << 2,
            const Start      = 1 << 3,
            const Up         = 1 << 4,
            const Down       = 1 << 5,
            const Left       = 1 << 6,
            const Right      = 1 << 7
        }
    }
}

/*
    The NES's master clock frequency is 21.477272 Mhz.
    The CPU divides it by 12, hence runs at 1.7897727 Mhz.
    The PPU divides it by 4, hence runs at 5.369318 Mhz.
    The APU divides it by 89490, hence runs at 239.996335 Hz.
    Since 12 / 4 = 3 there are 3 PPU clocks per 1 CPU clock.
    Since 89490 / 12 = 7457.5 there are 7457.5 CPU clocks per 1 APU clock.
*/

// A shim like this is necessary to implement an orphaned instance in Rust.
use orphan::Orphan;

impl< C: Context > mos6502::Context for Orphan< C > {
    #[inline]
    fn state_mut( &mut self ) -> &mut mos6502::State {
        &mut self.as_mut().state_mut().cpu_state
    }

    #[inline]
    fn state( &self ) -> &mos6502::State {
        &self.as_ref().state().cpu_state
    }

    fn peek( &mut self, address: u16 ) -> u8 {
        self.as_mut().on_cpu_cycle();
        dma::Interface::execute( self, address );
        Private::peek_memory( self.as_mut(), address )
    }

    #[inline]
    fn poke( &mut self, address: u16, value: u8 ) {
        self.as_mut().on_cpu_cycle();
        Private::poke_memory( self.as_mut(), address, value );
    }

    #[inline]
    fn bcd_mode_supported() -> bool {
        false // The 6502 inside NES doesn't support BCD mode.
    }
}

impl< C: Context > rp2c02::Context for Orphan< C > {
    #[inline]
    fn state_mut( &mut self ) -> &mut rp2c02::State {
        &mut self.as_mut().state_mut().ppu_state
    }

    #[inline]
    fn state( &self ) -> &rp2c02::State {
        &self.as_ref().state().ppu_state
    }

    #[inline]
    fn on_frame_was_generated( &mut self ) {
        self.as_mut().state_mut().frame_counter += 1;
        Context::on_frame( self.as_mut() );
    }

    #[inline]
    fn set_vblank_nmi( &mut self, state: bool ) {
        mos6502::Interface::set_nmi_latch( self, state );
    }

    #[inline]
    fn peek_video_memory( &self, address: u16 ) -> u8 {
        self.as_ref().state().mapper.peek_video_memory( address )
    }

    #[inline]
    fn poke_video_memory( &mut self, address: u16, value: u8 ) {
        self.as_mut().state_mut().mapper.poke_video_memory( address, value );
    }
}

impl< C: Context > virtual_apu::Context for Orphan< C > {
    #[inline]
    fn state_mut( &mut self ) -> &mut virtual_apu::State {
        &mut self.as_mut().state_mut().apu_state
    }

    #[inline]
    fn state( &self ) -> &virtual_apu::State {
        &self.as_ref().state().apu_state
    }

    #[inline]
    fn is_on_odd_cycle( &self ) -> bool {
        self.as_ref().is_on_odd_cycle()
    }

    #[inline]
    fn on_sample( &mut self, sample: f32 ) {
        self.as_mut().state_mut().audio_samples_counter += 1;
        if self.as_ref().state().audio_samples_counter >= (virtual_apu::Interface::sample_rate( self ) / 60) {
            self.as_mut().state_mut().audio_samples_counter = 0;
            self.as_mut().state_mut().full_frame_counter += 1;
        }
        Context::on_audio_sample( self.as_mut(), sample );
    }

    #[inline]
    fn set_irq_line( &mut self, state: bool ) {
        mos6502::Interface::set_irq_line( self, state );
    }

    #[inline]
    fn activate_dma( &mut self, address: u16 ) {
        dma::Interface::activate_dmc_dma( self, address );
    }
}

impl< C: Context > dma::Context for Orphan< C > {
    #[inline]
    fn state_mut( &mut self ) -> &mut dma::State {
        &mut self.as_mut().state_mut().dma_state
    }

    #[inline]
    fn state( &self ) -> &dma::State {
        &self.as_ref().state().dma_state
    }

    #[inline]
    fn fetch( &mut self, address: u16 ) -> u8 {
        let value = Interface::peek_memory( self.as_mut(), address );
        self.as_mut().on_cpu_cycle();

        value
    }

    #[inline]
    fn is_on_odd_cycle( &self ) -> bool {
        self.as_ref().is_on_odd_cycle()
    }

    #[inline]
    fn write_sprite_list_ram( &mut self, offset: u8, value: u8 ) {
        rp2c02::Interface::poke_sprite_list_ram( self, offset, value );
        self.as_mut().on_cpu_cycle();
    }

    #[inline]
    fn on_delta_modulation_dma_finished( &mut self, value: u8 ) {
        virtual_apu::Interface::on_delta_modulation_channel_dma_finished( self, value );
    }
}

trait Private: Sized + Context {
    fn newtype( &self ) -> &Orphan< Self > {
        Orphan::< Self >::cast( self )
    }

    fn newtype_mut( &mut self ) -> &mut Orphan< Self > {
        Orphan::< Self >::cast_mut( self )
    }

    fn load_rom< T: Read + Seek >( &mut self, stream: &mut T ) -> Result< (), LoadError > {
        let rom = try!( NesRom::load( stream ) );
        info!( "Loaded ROM: {:?}", rom );

        let mapper = try!( create_mapper( rom ) );

        self.state_mut().mapper = mapper;
        self.state_mut().ready = true;
        self.hard_reset();

        Ok(())
    }

    fn load_rom_from_memory( &mut self, buffer: &[u8] ) -> Result< (), LoadError > {
        use std::io::Cursor;
        let mut cursor = Cursor::new( buffer );
        self.load_rom( &mut cursor )
    }

    fn load_rom_from_file< P: AsRef< Path >>( &mut self, path: P ) -> Result< (), LoadError > {
        let mut fp = try!( File::open( path ) );
        self.load_rom( &mut fp )
    }

    fn hard_reset( &mut self ) {
        let mut mapper: Box< Mapper + 'static > = Box::new( MapperNull );
        mem::swap( &mut mapper, &mut self.state_mut().mapper );
        let ready = self.state().ready;

        // FIXME: This doesn't reset the mapper.
        *self.state_mut() = State {
            mapper: mapper,
            ready: ready,
            .. State::new()
        };
        self.soft_reset();
    }

    fn soft_reset( &mut self ) {
        // TODO: This is probably not accurate.
        self.state_mut().ppu_state = rp2c02::State::new();
        self.state_mut().apu_state = virtual_apu::State::new();
        self.state_mut().dma_state = dma::State::new();

        mos6502::Interface::reset( self.newtype_mut() );
    }

    fn copy_into_memory( &mut self, offset: u16, data: &[u8] ) {
        copy_memory( data, &mut self.state_mut().ram[ offset as usize.. ] );
    }

    fn execute_until_vblank( &mut self ) -> Result< (), Box< Error > > {
        if !self.state().ready {
            return Ok(());
        }

        let last_counter_value = self.state().frame_counter;
        while self.state().frame_counter == last_counter_value {
            let result = mos6502::Interface::execute( self.newtype_mut() );
            if let Err( error ) = result {
                self.state_mut().error = Some( Box::new( error ) );
                break;
            }
        }

        if self.state().error.is_none() {
            Ok(())
        } else {
            self.state_mut().ready = false;
            Err( self.state_mut().error.take().unwrap() )
        }
    }

    fn execute_for_a_frame( &mut self ) -> Result< (), Box< Error > > {
        if !self.state().ready {
            return Ok(());
        }

        let last_counter_value = self.state().full_frame_counter;
        while self.state().full_frame_counter == last_counter_value {
            let result = mos6502::Interface::execute( self.newtype_mut() );
            if let Err( error ) = result {
                self.state_mut().error = Some( Box::new( error ) );
                break;
            }
        }

        if self.state().error.is_none() {
            Ok(())
        } else {
            self.state_mut().ready = false;
            Err( self.state_mut().error.take().unwrap() )
        }
    }

    fn execute_cycle( &mut self ) -> Result< bool, Box< Error > > {
        let last_counter_value = self.state().full_frame_counter;
        mos6502::Interface::execute( self.newtype_mut() )?;

        Ok( self.state().full_frame_counter != last_counter_value )
    }

    fn framebuffer( &self ) -> &rp2c02::Framebuffer {
        rp2c02::Interface::framebuffer( self.newtype() )
    }

    fn swap_framebuffer( &mut self, other: rp2c02::Framebuffer ) -> rp2c02::Framebuffer {
        rp2c02::Interface::swap_framebuffer( self.newtype_mut(), other )
    }

    fn set_button_state( &mut self, port: ControllerPort, button: Button::Ty, is_pressed: bool ) {
        if is_pressed {
            self.press( port, button );
        } else {
            self.release( port, button );
        }
    }

    fn press( &mut self, port: ControllerPort, button: Button::Ty ) {
        self.gamepad( port ).insert( button );
    }

    fn release( &mut self, port: ControllerPort, button: Button::Ty ) {
        self.gamepad( port ).remove( button );
    }

    #[allow(unused_assignments)]
    fn peek_memory( &mut self, address: u16 ) -> u8 {
        match_cpu_address!( address,
            {
                self.state().ram.peek( translate_address_ram( address ) )
            }, {
                match translate_address_ioreg_ppu( address ) {
                    2 => rp2c02::Interface::peek_ppustatus( self.newtype_mut() ),
                    4 => rp2c02::Interface::peek_oamdata( self.newtype_mut() ),
                    7 => rp2c02::Interface::peek_ppudata( self.newtype_mut() ),
                    _ => {
                        warn!( "Unhandled read from PPU register 0x{:04X}", address );
                        0
                    }
                }
            }, {
                match translate_address_ioreg_other( address ) {
                     0 |  1 |  2 |  3 |  4 |  5 |  6 |  7 |  8 |  9 |
                    10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 |
                    20 => { 0 }, // Write-only or unused.
                    21 => virtual_apu::Interface::peek_status( self.newtype_mut() ),
                    22 => self.peek_gamepad( ControllerPort::First ),
                    23 => self.peek_gamepad( ControllerPort::Second ),
                    24 | 25 | 26 | 27 | 28 | 29 | 30 | 31 => { 0 }, // Write-only or unused.
                    _ => unsafe { fast_unreachable!() }
                }
            }, {
                self.state().mapper.peek_expansion_rom( address )
            }, {
                self.state().mapper.peek_sram( address )
            }, {
                self.state().mapper.peek_rom( address )
            }
        )
    }

    #[allow(unused_assignments)]
    fn poke_memory( &mut self, address: u16, value: u8 ) {
        match_cpu_address!( address,
            {
                self.state_mut().ram.poke( translate_address_ram( address ), value );
            }, {
                match translate_address_ioreg_ppu( address ) {
                    0 => rp2c02::Interface::poke_ppuctrl( self.newtype_mut(), value ),
                    1 => rp2c02::Interface::poke_ppumask( self.newtype_mut(), value ),
                    3 => rp2c02::Interface::poke_oamaddr( self.newtype_mut(), value ),
                    4 => rp2c02::Interface::poke_oamdata( self.newtype_mut(), value ),
                    5 => rp2c02::Interface::poke_ppuscroll( self.newtype_mut(), value ),
                    6 => rp2c02::Interface::poke_ppuaddr( self.newtype_mut(), value ),
                    7 => rp2c02::Interface::poke_ppudata( self.newtype_mut(), value ),
                    _ => warn!( "Unhandled write to PPU register 0x{:04X} (value=0x{:02X})", address, value )
                }
            }, {
                match translate_address_ioreg_other( address ) {
                     0 => virtual_apu::Interface::poke_square_1_ctrl( self.newtype_mut(), value ),
                     1 => virtual_apu::Interface::poke_square_1_frequency_generator( self.newtype_mut(), value ),
                     2 => virtual_apu::Interface::poke_square_1_period_low( self.newtype_mut(), value ),
                     3 => virtual_apu::Interface::poke_square_1_period_high( self.newtype_mut(), value ),
                     4 => virtual_apu::Interface::poke_square_2_ctrl( self.newtype_mut(), value ),
                     5 => virtual_apu::Interface::poke_square_2_frequency_generator( self.newtype_mut(), value ),
                     6 => virtual_apu::Interface::poke_square_2_period_low( self.newtype_mut(), value ),
                     7 => virtual_apu::Interface::poke_square_2_period_high( self.newtype_mut(), value ),
                     8 => virtual_apu::Interface::poke_triangle_ctrl( self.newtype_mut(), value ),
                     9 => { /* Unused. */ },
                    10 => virtual_apu::Interface::poke_triangle_period_low( self.newtype_mut(), value ),
                    11 => virtual_apu::Interface::poke_triangle_period_high( self.newtype_mut(), value ),
                    12 => virtual_apu::Interface::poke_noise_ctrl( self.newtype_mut(), value ),
                    13 => { /* Unused. */ },
                    14 => virtual_apu::Interface::poke_noise_period( self.newtype_mut(), value ),
                    15 => virtual_apu::Interface::poke_noise_counter_ctrl( self.newtype_mut(), value ),
                    16 => virtual_apu::Interface::poke_dmc_ctrl( self.newtype_mut(), value ),
                    17 => virtual_apu::Interface::poke_dmc_direct_load( self.newtype_mut(), value ),
                    18 => virtual_apu::Interface::poke_dmc_sample_address( self.newtype_mut(), value ),
                    19 => virtual_apu::Interface::poke_dmc_sample_length( self.newtype_mut(), value ),
                    20 => {
                        // This will trigger a DMA transfer on the next read cycle.
                        dma::Interface::activate_sprite_dma( self.newtype_mut(), (value as u16) << 8 );
                    },
                    21 => virtual_apu::Interface::poke_control( self.newtype_mut(), value ),
                    22 => {
                        if self.state().gamepad_shift_register_update {
                            self.update_gamepad_shift_register( ControllerPort::First );
                            self.update_gamepad_shift_register( ControllerPort::Second );
                        }
                        self.state_mut().gamepad_shift_register_update = (value & 1) != 0;
                    },
                    23 => {
                        let is_on_odd_cycle = self.is_on_odd_cycle();
                        virtual_apu::Interface::poke_frame_sequencer_ctrl( self.newtype_mut(), value, is_on_odd_cycle )
                    },
                    _ => warn!( "Unhandled write to IO register 0x{:04X} (value=0x{:02X})", address, value )
                }
            }, {
                self.state().mapper.poke_expansion_rom( address, value );
            }, {
                self.state_mut().mapper.poke_sram( address, value );
            }, {
                self.state_mut().mapper.poke_rom( address, value );
            }
        )
    }

    fn update_gamepad_shift_register( &mut self, port: ControllerPort ) {
        let mut flags = *self.gamepad( port );

        // It's not possible to press two opposite directions on an original NES controller.
        if flags.contains( Button::Left | Button::Right ) {
            flags.remove( Button::Left | Button::Right );
        }

        if flags.contains( Button::Up | Button::Down ) {
            flags.remove( Button::Up | Button::Down );
        }

        *self.gamepad_shift_register( port ) = flags.bits();
    }

    fn peek_gamepad( &mut self, port: ControllerPort ) -> u8 {
        if self.state().gamepad_shift_register_update {
            self.update_gamepad_shift_register( port );
        }

        let shift_register = self.gamepad_shift_register( port );
        let value = *shift_register & 1;
        *shift_register = (*shift_register >> 1) | 0b10000000;

        // While reading the gamepad I/O register the upper 5-bits on the data
        // bus are not driven, so they are equal to the residual bits
        // from the last time the bus was used, and the bus was last used
        // to select the address of the gamepad I/O register, which is 0x4016.
        value | 0x40
    }

    fn gamepad( &mut self, port: ControllerPort ) -> &mut Button::Ty {
        match port {
            ControllerPort::First => &mut self.state_mut().gamepad_1,
            ControllerPort::Second => &mut self.state_mut().gamepad_2
        }
    }

    fn gamepad_shift_register( &mut self, port: ControllerPort ) -> &mut u8 {
        match port {
            ControllerPort::First => &mut self.state_mut().gamepad_shift_register_1,
            ControllerPort::Second => &mut self.state_mut().gamepad_shift_register_2
        }
    }

    #[inline]
    fn is_on_odd_cycle( &self ) -> bool {
        (self.state().cpu_cycle & 1) != 0
    }

    fn on_cpu_cycle( &mut self ) {
        self.state_mut().cpu_cycle.wrapping_inc();
        virtual_apu::Interface::execute( self.newtype_mut() );
        for _ in 0..3 {
            rp2c02::Interface::execute( self.newtype_mut() );
        }

        Context::on_cycle( self );
    }
}