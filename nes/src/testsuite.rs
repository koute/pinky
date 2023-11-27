use std::error::Error;
use nes_testsuite;

mod nes {
    pub use virtual_nes::{State, Interface, Context};
}

struct Instance {
    state: nes::State,
    testcase_state: nes_testsuite::TestcaseState
}

impl nes::Context for Instance {
    #[inline]
    fn state_mut( &mut self ) -> &mut nes::State {
        &mut self.state
    }

    #[inline]
    fn state( &self ) -> &nes::State {
        &self.state
    }

    #[inline]
    fn on_cycle( &mut self ) {
        nes_testsuite::on_cycle( self );
    }

    #[inline]
    fn on_frame( &mut self ) {
        nes_testsuite::on_frame( self );
    }
}

impl nes_testsuite::EmulatorInterface for Instance {
    fn new( rom_data: &[u8], testcase_state: nes_testsuite::TestcaseState ) -> Result< Self, Box< dyn Error >> {
        let mut instance = Instance {
            state: nes::State::new(),
            testcase_state: testcase_state
        };

        nes::Interface::load_rom( &mut instance, rom_data )?;
        Ok( instance )
    }

    fn run( &mut self ) -> Result< (), Box< dyn Error >> {
        Ok(nes::Interface::execute_until_vblank( self )?)
    }

    fn peek_memory( &mut self, address: u16 ) -> u8 {
        nes::Interface::peek_memory( self, address )
    }

    fn poke_memory( &mut self, address: u16, value: u8 ) {
        nes::Interface::poke_memory( self, address, value )
    }

    fn get_framebuffer( &mut self, output: &mut [u8] ) {
        let framebuffer = nes::Interface::framebuffer( self );
        for (out, pixel) in output.iter_mut().zip( framebuffer.iter() ) {
            *out = pixel.base_color_index();
        }
    }

    #[inline]
    fn testcase_state( &mut self ) -> &mut nes_testsuite::TestcaseState {
        &mut self.testcase_state
    }
}

nes_testsuite!( Instance );
