use std::error::Error;

fn md5sum< T: AsRef< [u8] > >( data: T ) -> String {
    use md5;
    let raw_digest = md5::compute( data.as_ref() );
    let mut digest = String::with_capacity( 2 * 16 );
    for byte in raw_digest.iter() {
        digest.push_str( &format!( "{:02x}", byte ) );
    }
    digest
}

pub struct TestcaseState {
    cycle: u64,
    frame: u64,
    running: bool,
    frame_limit: u64,
    expected_framebuffer_md5sum: Option< String >,
    framebuffer_md5sum: Option< String >
}

impl TestcaseState {
    fn new() -> TestcaseState {
        TestcaseState {
            cycle: 0,
            frame: 0,
            running: true,
            frame_limit: 0,
            expected_framebuffer_md5sum: None,
            framebuffer_md5sum: None
        }
    }
}

pub trait EmulatorInterface: Sized {
    fn new( rom_data: &[u8], testcase_state: TestcaseState ) -> Result< Self, Box< Error >>;
    fn run( &mut self ) -> Result< (), Box< Error >>;
    fn peek_memory( &mut self, address: u16 ) -> u8;
    fn poke_memory( &mut self, address: u16, value: u8 );
    fn get_framebuffer( &mut self, output: &mut [u8] );

    fn testcase_state( &mut self ) -> &mut TestcaseState;
}

#[inline]
pub fn on_cycle< T: EmulatorInterface >( emulator: &mut T ) {
    emulator.testcase_state().cycle += 1;
    let current_cycle = emulator.testcase_state().cycle;
    if current_cycle >= 50_000_000 {
        emulator.testcase_state().running = false;
    }
}

#[inline]
pub fn on_frame< T: EmulatorInterface >( emulator: &mut T ) {
    emulator.testcase_state().frame += 1;

    let frame_limit = emulator.testcase_state().frame_limit * 2;
    if emulator.testcase_state().frame == frame_limit {
        emulator.testcase_state().running = false;

        let mut framebuffer = [0; 256 * 240];
        emulator.get_framebuffer( &mut framebuffer );
        emulator.testcase_state().framebuffer_md5sum = Some( md5sum( &framebuffer[..] ) );
    }
}

pub fn standard_testcase< T: EmulatorInterface >( rom: &'static [u8], expected_framebuffer_md5sum: &str, frame_limit: u64 ) {
    let mut state = TestcaseState::new();
    state.frame_limit = frame_limit;
    state.expected_framebuffer_md5sum = Some( expected_framebuffer_md5sum.to_owned() );

    let mut interface = T::new( rom, state ).unwrap();
    while interface.testcase_state().running {
        let cycles_before = interface.testcase_state().cycle;
        interface.run().unwrap();
        let cycles_after = interface.testcase_state().cycle;

        assert!( cycles_before != cycles_after, "You need to call nes_testsuite::on_cycle every cycle!" );
    }

    assert!( interface.testcase_state().frame > 0, "You need to call nes_testsuite::on_frame every frame!" );

    let expected_framebuffer_md5sum = interface.testcase_state().expected_framebuffer_md5sum.take();
    let actual_framebuffer_md5sum = interface.testcase_state().framebuffer_md5sum.take();
    assert_eq!( expected_framebuffer_md5sum, actual_framebuffer_md5sum );
}
