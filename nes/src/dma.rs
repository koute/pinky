pub trait Context: Sized {
    fn state_mut( &mut self ) -> &mut State;
    fn state( &self ) -> &State;

    fn dummy_fetch( &mut self, address: u16 ) {
        let _ = self.fetch( address );
    }

    fn fetch( &mut self, address: u16 ) -> u8;
    fn is_on_odd_cycle( &self ) -> bool;

    fn write_sprite_list_ram( &mut self, offset: u8, value: u8 );
    fn on_delta_modulation_dma_finished( &mut self, value: u8 );
}

pub trait Interface: Sized + Context {
    #[inline]
    fn activate_dmc_dma( &mut self, source_address: u16 ) {
        Private::activate_dmc_dma( self, source_address )
    }

    #[inline]
    fn activate_sprite_dma( &mut self, source_address: u16 ) {
        Private::activate_sprite_dma( self, source_address )
    }

    #[inline]
    fn execute( &mut self, address_cpu_tried_to_read: u16 ) {
        Private::execute( self, address_cpu_tried_to_read )
    }
}

impl< T: Context > Interface for T {}
impl< T: Context > Private for T {}

pub struct State {
    dmc_dma_requested: bool,
    dmc_dma_source_address: u16,
    sprite_dma_requested: bool,
    sprite_dma_source_address: u16
}

impl State {
    pub const fn new() -> State {
        State {
            dmc_dma_requested: false,
            dmc_dma_source_address: 0,
            sprite_dma_requested: false,
            sprite_dma_source_address: 0
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Progress {
    None,
    HaltCycleFinished,
    DummyCycleFinished
}

trait Private: Sized + Context {
    fn activate_dmc_dma( &mut self, source_address: u16 ) {
        self.state_mut().dmc_dma_requested = true;
        self.state_mut().dmc_dma_source_address = source_address;
    }

    fn activate_sprite_dma( &mut self, source_address: u16 ) {
        self.state_mut().sprite_dma_requested = true;
        self.state_mut().sprite_dma_source_address = source_address;
    }

    #[inline]
    fn is_in_get_phase( &self ) -> bool {
        !self.is_on_odd_cycle()
    }

    #[inline]
    fn is_in_put_phase( &self ) -> bool {
        !self.is_in_get_phase()
    }

    fn progress_dmc_dma_if_necessary( &mut self, dmc_progress: &mut Progress ) {
        if self.state().dmc_dma_requested == true {
            *dmc_progress = match *dmc_progress {
                // If the DMC starts during the sprite DMA it needs a halt cycle too.
                Progress::None => Progress::HaltCycleFinished,
                // The DMC DMA needs a dummy cycle before it's allowed to continue;
                // that dummy cycle can simultaneously by a sprite DMA.
                Progress::HaltCycleFinished => Progress::DummyCycleFinished,
                Progress::DummyCycleFinished => Progress::DummyCycleFinished
            }
        }
    }

    // DMA information: http://forums.nesdev.com/viewtopic.php?f=3&t=14120
    fn execute( &mut self, address_cpu_tried_to_read: u16 ) {
        if self.state_mut().dmc_dma_requested == false && self.state_mut().sprite_dma_requested == false {
            return;
        }

        let mut dmc_dma_progress = Progress::None;
        let mut sprite_dma_value = None;
        let mut sprite_dma_current_offset = 0;

        if self.state().dmc_dma_requested {
            dmc_dma_progress = Progress::HaltCycleFinished;
        }

        // If this is a sprite DMA we can assume that its
        // halt cycle finished too; we don't need to keep
        // that information inside a variable since only
        // the CPU can trigger sprite DMA, and we are
        // blocking the CPU when executing DMA.

        // Hijack and discard the value the CPU originally tried to read.
        self.dummy_fetch( address_cpu_tried_to_read );

        loop {
            let dmc_dma_ready_to_read =
                self.state().dmc_dma_requested &&
                dmc_dma_progress == Progress::DummyCycleFinished &&
                self.is_in_get_phase();

            // If we want to read we need to be in the "get" phase;
            // if we want to write we need to be in the "put" phase.

            let sprite_dma_ready_to_read =
                self.state().sprite_dma_requested &&
                sprite_dma_value.is_none() &&
                self.is_in_get_phase();

            let sprite_dma_ready_to_write =
                self.state().sprite_dma_requested &&
                sprite_dma_value.is_some() &&
                self.is_in_put_phase();

            if dmc_dma_ready_to_read == true {
                let address = self.state().dmc_dma_source_address;
                let value = self.fetch( address );

                self.state_mut().dmc_dma_requested = false;
                self.on_delta_modulation_dma_finished( value );
            } else if sprite_dma_ready_to_read == true {
                self.progress_dmc_dma_if_necessary( &mut dmc_dma_progress );
                let address = self.state().sprite_dma_source_address + sprite_dma_current_offset;
                sprite_dma_value = Some( self.fetch( address ) );
            } else if sprite_dma_ready_to_write == true {
                self.progress_dmc_dma_if_necessary( &mut dmc_dma_progress );
                self.write_sprite_list_ram( sprite_dma_current_offset as u8, sprite_dma_value.take().unwrap() );

                sprite_dma_current_offset += 1;
                if sprite_dma_current_offset == 256 {
                    self.state_mut().sprite_dma_requested = false;
                }
            } else if self.state().dmc_dma_requested == true || self.state().sprite_dma_requested == true {
                self.progress_dmc_dma_if_necessary( &mut dmc_dma_progress );
                self.dummy_fetch( address_cpu_tried_to_read );
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;
    use dma;
    use super::{Context, Interface, State};

    #[derive(Copy, Clone, PartialEq, Eq)]
    enum Action {
        Fetch( u16 ),
        WriteSpriteList( u8, u8 ),
        DeltaModulationDmaFinished( u8 )
    }

    #[derive(Copy, Clone, PartialEq, Eq)]
    enum InitialPhase {
        Get,
        Put
    }

    impl fmt::Debug for Action {
        fn fmt( &self, fmt: &mut fmt::Formatter ) -> fmt::Result {
            use self::Action::*;
            match *self {
                Fetch( address ) => write!( fmt, "fetch of 0x{:04X}", address ),
                WriteSpriteList( offset, value ) => write!( fmt, "write to sprite list with offset {} and value 0x{:02X}", offset, value ),
                DeltaModulationDmaFinished( value ) => write!( fmt, "DMC DMA finished callback with value 0x{:02X}", value )
            }
        }
    }

    struct Instance {
        state: State,
        actions: Vec< Action >,
        cycle: u64,
        initial_phase: InitialPhase
    }

    impl Instance {
        fn new() -> Instance {
            Instance {
                state: State::new(),
                actions: Vec::new(),
                cycle: 0,
                initial_phase: InitialPhase::Get
            }
        }
    }

    impl Context for Instance {
        fn state_mut( &mut self ) -> &mut State {
            &mut self.state
        }

        fn state( &self ) -> &State {
            &self.state
        }

        fn fetch( &mut self, address: u16 ) -> u8 {
            self.actions.push( Action::Fetch( address ) );
            self.cycle += 1;

            (address & 0xff) as u8
        }

        fn is_on_odd_cycle( &self ) -> bool {
            let result = (self.cycle % 2) == 1;
            if self.initial_phase == InitialPhase::Put {
                return !result;
            } else {
                return result;
            }
        }

        fn write_sprite_list_ram( &mut self, offset: u8, value: u8 ) {
            self.actions.push( Action::WriteSpriteList( offset, value ) );
            self.cycle += 1;
        }

        fn on_delta_modulation_dma_finished( &mut self, value: u8 ) {
            self.actions.push( Action::DeltaModulationDmaFinished( value ) );
        }
    }

    fn verify_actions( actual_actions: &[Action], expected_actions: &[Action] ) {
        use std::cmp::min;
        let common_length = min( actual_actions.len(), expected_actions.len() );
        for (index, (actual, expected)) in actual_actions.iter().zip( expected_actions.iter() ).enumerate() {
            assert!( actual == expected,  "Action #{}: expected {:?}, got {:?}", index, expected, actual );
        }

        if actual_actions.len() > common_length {
            panic!( "Action #{}: unexpected: {:?}", common_length, actual_actions[ common_length ] );
        }

        if expected_actions.len() > common_length {
            panic!( "Action #{}: expected {:?}, got none", common_length, expected_actions[ common_length ] );
        }
    }

    #[test]
    fn sprite_dma_starting_on_a_put_cycle() {
        let mut expected_actions = Vec::new();
        expected_actions.push( Action::Fetch( 0x3333 ) ); // Halt cycle. (PUT)

        // 256 read-write cycles.
        for nth in 0..256 {
            expected_actions.push( Action::Fetch( 0xabcd + nth ) );
            expected_actions.push( Action::WriteSpriteList( nth as u8, ((0xabcd + nth) & 0xff) as u8 ) );
        }

        let mut instance = Instance::new();
        instance.initial_phase = InitialPhase::Put;
        assert_eq!( dma::Private::is_in_put_phase( &instance ), true );
        instance.activate_sprite_dma( 0xabcd );
        instance.execute( 0x3333 );

        verify_actions( &instance.actions[..], &expected_actions[..] );
    }

    #[test]
    fn sprite_dma_starting_on_a_get_cycle() {
        let mut expected_actions = Vec::new();
        expected_actions.push( Action::Fetch( 0x3333 ) ); // Halt cycle. (GET)
        expected_actions.push( Action::Fetch( 0x3333 ) ); // Alignment cycle. (PUT)

        // 256 read-write cycles.
        for nth in 0..256 {
            expected_actions.push( Action::Fetch( 0xabcd + nth ) );
            expected_actions.push( Action::WriteSpriteList( nth as u8, ((0xabcd + nth) & 0xff) as u8 ) );
        }

        let mut instance = Instance::new();
        instance.initial_phase = InitialPhase::Get;
        assert_eq!( dma::Private::is_in_get_phase( &instance ), true );
        instance.activate_sprite_dma( 0xabcd );
        instance.execute( 0x3333 );

        verify_actions( &instance.actions[..], &expected_actions[..] );
    }

    #[test]
    fn dmc_dma_starting_on_a_put_cycle() {
        let expected_actions = vec![
            Action::Fetch( 0x3333 ), // Halt cycle. (PUT)
            Action::Fetch( 0x3333 ), // Extra DMC dummy read. (GET)
            Action::Fetch( 0x3333 ), // Dummy cycle for alignment. (PUT)
            Action::Fetch( 0xabcd ), // Actual read (GET)
            Action::DeltaModulationDmaFinished( 0xcd ) // Not an actual cycle; just calls the callback.
        ];

        let mut instance = Instance::new();
        instance.initial_phase = InitialPhase::Put;
        instance.activate_dmc_dma( 0xabcd );
        instance.execute( 0x3333 );

        verify_actions( &instance.actions[..], &expected_actions[..] );
    }

    #[test]
    fn dmc_dma_and_sprite_dma_on_the_same_cycle_starting_on_a_get_cycle() {
        let mut expected_actions = Vec::new();
        expected_actions.push( Action::Fetch( 0x3333 ) ); // Halt cycle of both sprite DMA and DMC DMA. (GET)
        expected_actions.push( Action::Fetch( 0x3333 ) ); // DMC dummy cycle. (PUT)
        expected_actions.push( Action::Fetch( 0xcc22 ) ); // Actual DMC read. (GET)
        expected_actions.push( Action::DeltaModulationDmaFinished( 0x22 ) );
        expected_actions.push( Action::Fetch( 0x3333 ) ); // Sprite DMA realignment. (PUT)

        // 256 read-write cycles.
        for nth in 0..256 {
            expected_actions.push( Action::Fetch( 0xaa11 + nth ) );
            expected_actions.push( Action::WriteSpriteList( nth as u8, ((0xaa11 + nth) & 0xff) as u8 ) );
        }

        let mut instance = Instance::new();
        instance.initial_phase = InitialPhase::Get;
        instance.activate_sprite_dma( 0xaa11 );
        instance.activate_dmc_dma( 0xcc22 );
        instance.execute( 0x3333 );

        verify_actions( &instance.actions[..], &expected_actions[..] );
    }

    #[test]
    fn dmc_dma_and_sprite_dma_on_the_same_cycle_starting_on_a_put_cycle() {
        let mut expected_actions = Vec::new();
        expected_actions.push( Action::Fetch( 0x3333 ) ); // Halt cycle of both sprite DMA and DMC DMA. (PUT)
        expected_actions.push( Action::Fetch( 0xaa11 ) ); // Read sprite cycle/DMC dummy cycle. (GET)
        expected_actions.push( Action::WriteSpriteList( 0, 0x11 ) ); // Write sprite cycle/DMC alignment cycle. (PUT)
        expected_actions.push( Action::Fetch( 0xcc22 ) ); // Actual DMC read. (GET)
        expected_actions.push( Action::DeltaModulationDmaFinished( 0x22 ) );
        expected_actions.push( Action::Fetch( 0x3333 ) ); // Realignment cycle for sprite DMA. (PUT)

        // 255 read-write cycles.
        for nth in 1..256 {
            expected_actions.push( Action::Fetch( 0xaa11 + nth ) );
            expected_actions.push( Action::WriteSpriteList( nth as u8, ((0xaa11 + nth) & 0xff) as u8 ) );
        }

        let mut instance = Instance::new();
        instance.initial_phase = InitialPhase::Put;
        instance.activate_sprite_dma( 0xaa11 );
        instance.activate_dmc_dma( 0xcc22 );
        instance.execute( 0x3333 );

        verify_actions( &instance.actions[..], &expected_actions[..] );
    }
}