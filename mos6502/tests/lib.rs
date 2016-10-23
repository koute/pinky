extern crate mos6502;
extern crate emumisc;

use emumisc::copy_memory;
use mos6502::{Interface, State, Context, Address, EmulationStatus};

static ROM_BCD_VERIFY: &'static [u8] = include_bytes!( "../roms/bcd_verify.bin" );
static ROM_6502_FUNCTIONAL_TEST: &'static [u8] = include_bytes!( "../roms/6502_functional_test.bin" );
static ROM_TTL6502: &'static [u8] = include_bytes!( "../roms/TTL6502.bin" );

struct DummyHost {
    ram: [u8; 0xFFFF + 1],
    cycle: usize,
    cpu_state: State
}

impl DummyHost {
    pub fn new() -> DummyHost {
        DummyHost {
            ram: [0; 0xFFFF + 1],
            cycle: 0,
            cpu_state: State::new()
        }
    }

    pub fn load_binary( &mut self, addr: Address, data: &[u8] ) {
        assert!( addr as usize + data.len() <= self.ram.len() );
        copy_memory( data, &mut self.ram[ addr as usize.. ] );
    }
}

impl Context for DummyHost {
    #[inline(always)]
    fn state_mut( &mut self ) -> &mut State {
        &mut self.cpu_state
    }

    #[inline(always)]
    fn state( &self ) -> &State {
        &self.cpu_state
    }

    fn peek( &mut self, addr: u16 ) -> u8 {
        self.cycle += 1;
        self.ram[ addr as usize ]
    }

    fn poke( &mut self, addr: u16, value: u8 ) {
        self.cycle += 1;
        self.ram[ addr as usize ] = value;
    }

    fn bcd_mode_supported() -> bool {
        true
    }
}

#[test]
fn test_sample_bcd_verify() {
    let mut context = DummyHost::new();
    context.load_binary( 0xE000, ROM_BCD_VERIFY );
    context.reset();

    loop {
        let status = context.execute();
        match status {
            Ok( EmulationStatus::Normal ) => continue,
            Ok( EmulationStatus::InfiniteLoop( pc ) ) => {
                assert_eq!( pc, 0xF100 );
                break;
            },
            Err(_) => assert!( false, format!( "{:?}", status ) )
        }
    }

    let result = context.ram[ 0xE004 ];
    if result != 0 {
        let culprit = context.ram[ 0xE011 ];
        let culprit_2 = context.ram[ 0xE012 ];
        let num_1 = context.ram[ 0xE007 ];
        let num_2 = context.ram[ 0xE00A ];
        let expected = context.ram[ 0xE000 ];
        let got = context.ram[ 0xE002 ];
        let got_flags = context.ram[ 0xE003 ];
        let expected_n = context.ram[ 0xE00C ];
        let expected_v = context.ram[ 0xE00D ];
        let expected_z = context.ram[ 0xE00E ];
        let expected_c = context.ram[ 0xE001 ];

        println!( "Broken BCD handling:" );
        print!( "    Instruction: " );

        match culprit {
            0x50 => println!( "ADC" ),
            0x51 => println!( "SBC" ),
            _ => println!( "WTF: {}", culprit )
        }

        print!( "    Culprit: " );
        match culprit_2 {
            0x60 => println!( "result" ),
            0x61 => println!( "N flag" ),
            0x62 => println!( "V flag" ),
            0x63 => println!( "Z flag" ),
            0x64 => println!( "C flag" ),
            _ => println!( "WTF: {}", culprit_2 )
        }

        println!( "    Number #1: 0x{:02X}", num_1 );
        println!( "    Number #2: 0x{:02X}", num_2 );
        println!( "    Expected: 0x{:02X}, got 0x{:02X}", expected, got );
        println!( "    Flags:" );
        println!( "                  NV----ZC" );
        println!( "             Got: {:08b}", got_flags );
        println!( "        Expected: {:}{}----{}{}",
            ((expected_n & (1 << 7)) != 0) as u8,
            ((expected_v & (1 << 6)) != 0) as u8,
            ((expected_z & (1 << 1)) != 0) as u8,
            ((expected_c & (1 << 0)) != 0) as u8
        );
        println!( "    Cycle count: {}", context.cycle );
        panic!();
    }
}

#[test]
fn test_sample_6502_functional_test() {
    let mut context = DummyHost::new();
    context.load_binary( 0x0000, ROM_6502_FUNCTIONAL_TEST );
    context.set_pc( 0x400 );

    loop {
        let status = context.execute();
        match status {
            Ok( EmulationStatus::Normal ) => continue,
            Ok( EmulationStatus::InfiniteLoop( pc ) ) => {
                assert_eq!( pc, 0x3399 ); // Infinite loop at this address means success.
                break;
            },
            Err(_) => assert!( false, format!( "{:?}", status ) )
        }
    }
}

#[test]
fn test_sample_ttl6502() {
    let mut context = DummyHost::new();
    context.load_binary( 0xE000, ROM_TTL6502 );
    context.reset();

    loop {
        /*
            This is where TTL6502 starts to test the BCD mode which has different behavior
            from the one which we emulate.
        */
        if context.pc() == 0xF5B6 {
            break;
        }

        let status = context.execute();
        assert_eq!( status, Ok( EmulationStatus::Normal ) );
    }
}
