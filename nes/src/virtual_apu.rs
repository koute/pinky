use emumisc::{WrappingExtra, PeekPoke, BitExtra, is_b7_set, is_b6_set, is_b5_set, is_b4_set, is_b3_set, is_b2_set, is_b1_set, is_b0_set, to_bit};
use filter::Filter;
use float::{F64, F32, u8_to_f32};

pub trait Context: Sized {
    fn state_mut( &mut self ) -> &mut State;
    fn state( &self ) -> &State;

    fn is_on_odd_cycle( &self ) -> bool;

    fn on_sample( &mut self, sample: F32 );
    fn set_irq_line( &mut self, state: bool );
    fn activate_dma( &mut self, address: u16 );
}

pub trait Interface: Sized + Context {
    #[inline]
    fn execute( &mut self ) {
        Private::execute( self )
    }

    #[inline]
    fn sample_rate( &self ) -> u32 {
        Private::sample_rate( self )
    }

    #[inline]
    fn poke_frame_sequencer_ctrl( &mut self, value: u8, is_odd_cycle: bool ) {
        Private::poke_frame_sequencer_ctrl( self, value, is_odd_cycle )
    }

    #[inline]
    fn poke_control( &mut self, value: u8 ) {
        Private::poke_control( self, value )
    }

    #[inline]
    fn peek_status( &mut self ) -> u8 {
        Private::peek_status( self )
    }

    #[inline]
    fn poke_square_1_ctrl( &mut self, value: u8 ) {
        Private::poke_square_1_ctrl( self, value )
    }

    #[inline]
    fn poke_square_1_frequency_generator( &mut self, value: u8 ) {
        Private::poke_square_1_frequency_generator( self, value )
    }

    #[inline]
    fn poke_square_1_period_low( &mut self, value: u8 ) {
        Private::poke_square_1_period_low( self, value )
    }

    #[inline]
    fn poke_square_1_period_high( &mut self, value: u8 ) {
        Private::poke_square_1_period_high( self, value )
    }

    #[inline]
    fn poke_square_2_ctrl( &mut self, value: u8 ) {
        Private::poke_square_2_ctrl( self, value )
    }

    #[inline]
    fn poke_square_2_frequency_generator( &mut self, value: u8 ) {
        Private::poke_square_2_frequency_generator( self, value )
    }

    #[inline]
    fn poke_square_2_period_low( &mut self, value: u8 ) {
        Private::poke_square_2_period_low( self, value )
    }

    #[inline]
    fn poke_square_2_period_high( &mut self, value: u8 ) {
        Private::poke_square_2_period_high( self, value )
    }

    #[inline]
    fn poke_triangle_ctrl( &mut self, value: u8 ) {
        Private::poke_triangle_ctrl( self, value )
    }

    #[inline]
    fn poke_triangle_period_low( &mut self, value: u8 ) {
        Private::poke_triangle_period_low( self, value )
    }

    #[inline]
    fn poke_triangle_period_high( &mut self, value: u8 ) {
        Private::poke_triangle_period_high( self, value )
    }

    #[inline]
    fn poke_noise_ctrl( &mut self, value: u8 ) {
        Private::poke_noise_ctrl( self, value )
    }

    #[inline]
    fn poke_noise_period( &mut self, value: u8 ) {
        Private::poke_noise_period( self, value )
    }

    #[inline]
    fn poke_noise_counter_ctrl( &mut self, value: u8 ) {
        Private::poke_noise_counter_ctrl( self, value )
    }

    #[inline]
    fn poke_dmc_ctrl( &mut self, value: u8 ) {
        Private::poke_dmc_ctrl( self, value )
    }

    #[inline]
    fn poke_dmc_direct_load( &mut self, value: u8 ) {
        Private::poke_dmc_direct_load( self, value )
    }

    #[inline]
    fn poke_dmc_sample_address( &mut self, value: u8 ) {
        Private::poke_dmc_sample_address( self, value )
    }

    #[inline]
    fn poke_dmc_sample_length( &mut self, value: u8 ) {
        Private::poke_dmc_sample_length( self, value )
    }

    #[inline]
    fn on_delta_modulation_channel_dma_finished( &mut self, value: u8 ) {
        Private::on_delta_modulation_channel_dma_finished( self, value )
    }
}

impl< T: Context > Interface for T {}
impl< T: Context > Private for T {}

pub struct State {
    cycles_to_next_step: u16,

    sequencer_reconfigured: Option< (SequencerMode, bool) >,
    sequencer_mode: SequencerMode,
    sequencer_table: &'static [Sequence],
    sequencer_current_step: u8,

    inhibit_interrupts: bool,
    interrupt_occured_flag: bool,

    channel_square_1: ChannelSquare,
    channel_square_2: ChannelSquare,
    channel_triangle: ChannelTriangle,
    channel_noise: ChannelNoise,
    channel_delta_modulation: ChannelDeltaModulation,

    filter: Filter,
    sampling_counter: F64,
    decimation_counter: u8
}

impl State {
    pub const fn new() -> State {
        State {
            cycles_to_next_step: SEQUENCER_STEPS_MODE_0_CONST[ 2 ].0,
            sequencer_reconfigured: None,
            sequencer_table: SEQUENCER_STEPS_MODE_0_CONST,
            sequencer_current_step: 2,
            sequencer_mode: SequencerMode::Mode0,
            inhibit_interrupts: false,
            interrupt_occured_flag: false,
            channel_square_1: ChannelSquare::new( false ),
            channel_square_2: ChannelSquare::new( true ),
            channel_triangle: ChannelTriangle::new(),
            channel_noise: ChannelNoise::new(),
            channel_delta_modulation: ChannelDeltaModulation::new(),

            filter: Filter::new(),
            sampling_counter: f64!(0.0),
            decimation_counter: 0
        }
    }
}

/*
    The NES has an Audio Processing Unit integrated into the 6502 package.

    It has five channels: two square wave channels, one triangle wave, one noise and one
    delta modulation channel which can play sampled audio - it can even be used to play
    PCM samples directly, however at a very high CPU usage.

    The wave channels have the following units: (not all of the channels have all four)
        - length counter,
        - linear counter,
        - volume generator,
        - frequency generator

    Besides playing sounds the APU can also be used for timing.

    Every unit is clocked by the frame sequencer. The sequencer has two modes, controlled
    by the 0x4017 I/O register; by default it starts in mode 0/state #2. An internal cycle
    counter is initialized with the cycle count corresponding to a given state; every CPU
    cycle that internal counter is decremented, and when it reaches zero the units corresponding
    to the current state are clocked while the frame sequencer jumps to the next state.

                                MODE 0

        #  |  CYCLES  |   NEXT  | SET | CLK | CLK | CLK | CLK |
           |          |  STATE  | IRQ | LEN | FRQ | LIN | VOL |
        ---|----------|---------|-----|-----|-----|-----|-----|
        #0 |   0001   |   #1    |     |     |     |     |     |
        #1 |   0001   |   #2    |     |     |     |     |     |
        #2 |   0001   |   #3    |     |     |     |     |     |
        #3 |   7457   |   #4    |     |     |     |  Y  |  Y  |
        #4 |   7456   |   #5    |     |  Y  |  Y  |  Y  |  Y  |
        #5 |   7458   |   #6    |     |     |     |  Y  |  Y  |
        #6 |   7457   |   #7    |  Y  |     |     |     |     |
        #7 |   0001   |   #8    |  Y  |  Y  |  Y  |  Y  |  Y  |
        #8 |   0001   |   #3    |  Y  |     |     |     |     |

                                MODE 1

        #  |  CYCLES  |   NEXT  | SET | CLK | CLK | CLK | CLK |
           |          |  STATE  | IRQ | LEN | FRQ | LIN | VOL |
        ---|----------|---------|-----|-----|-----|-----|-----|
        #0 |   0001   |   #1    |     |     |     |     |     |
        #1 |   0003   |   #2    |     |  Y  |  Y  |  Y  |  Y  |
        #2 |   7456   |   #5    |     |     |     |  Y  |  Y  |
        #3 |   7458   |   #4    |     |  Y  |  Y  |  Y  |  Y  |
        #4 |   7458   |   #5    |     |     |     |  Y  |  Y  |
        #5 |   7456   |   #6    |     |  Y  |  Y  |  Y  |  Y  |
        #6 |   7458   |   #7    |     |     |     |  Y  |  Y  |
        #7 |   7452   |   #3    |     |     |     |     |     |

    One cycle after a write to 0x4017 (any write) the sequence
    is set to either #0 if the write was made on an odd CPU cycle
    or to #1 if the write was made on an even CPU cycle.

*/

struct Sequence( u16, u8, bool, bool, bool, bool, bool );

impl Sequence {
    fn cycles( &self ) -> u16 { self.0 }
    fn next_sequence( &self ) -> u8 { self.1 }
    fn clk_irq( &self ) -> bool { self.2 }
    fn clk_length_counter( &self ) -> bool { self.3 }
    fn clk_frequency_generator( &self ) -> bool { self.4 }
    fn clk_linear_counter( &self ) -> bool { self.5 }
    fn clk_volume_generator( &self ) -> bool { self.6 }
}

const SEQUENCER_STEPS_MODE_0_CONST: &'static [Sequence] = &[
    Sequence(    1, 1, false, false, false, false, false ),
    Sequence(    1, 2, false, false, false, false, false ),
    Sequence(    1, 3, false, false, false, false, false ),
    Sequence( 7457, 4, false, false, false,  true,  true ),
    Sequence( 7456, 5, false,  true,  true,  true,  true ),
    Sequence( 7458, 6, false, false, false,  true,  true ),
    Sequence( 7457, 7,  true, false, false, false, false ),
    Sequence(    1, 8,  true,  true,  true,  true,  true ),
    Sequence(    1, 3,  true, false, false, false, false )
];

static SEQUENCER_STEPS_MODE_0: &'static [Sequence] = SEQUENCER_STEPS_MODE_0_CONST;

static SEQUENCER_STEPS_MODE_1: &'static [Sequence] = &[
    Sequence(     1, 1, false, false, false, false, false ),
    Sequence(     3, 2, false,  true,  true,  true,  true ),
    Sequence(  7456, 5, false, false, false,  true,  true ),

    Sequence(  7458, 4, false,  true,  true,  true,  true ),
    Sequence(  7458, 5, false, false, false,  true,  true ),
    Sequence(  7456, 6, false,  true,  true,  true,  true ),
    Sequence(  7458, 7, false, false, false,  true,  true ),
    Sequence(  7452, 3, false,  false, false, false, false )
];

#[derive(Copy, Clone, PartialEq, Eq)]
enum SequencerMode {
    Mode0,
    Mode1
}

// Taken from: http://wiki.nesdev.com/w/index.php/APU_Length_Counter
static LENGTH_COUNTER_LOOKUP_TABLE: &'static [u8; 32] = &[
    10, 254, 20,  2, 40,  4, 80,  6, 160,  8, 60, 10, 14, 12, 26, 14,
    12,  16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30
];

// Taken from: http://wiki.nesdev.com/w/index.php/APU_Pulse
static SQUARE_CHANNEL_DUTY_TABLE: &'static [[u8; 8]; 4] = &[
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1]
];

// Taken from: http://wiki.nesdev.com/w/index.php/APU_Noise
static NOISE_CHANNEL_NTSC_PERIOD_TABLE: &'static [u16; 16] = &[
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068
];

// Taken from: http://wiki.nesdev.com/w/index.php/APU_DMC
static DMC_CHANNEL_NTSC_DELTA_PLAYBACK_PERIOD_TABLE: &'static [u16; 16] = &[
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54
];

// TODO: Support PAL.

struct Channel {
    pub length_counter: u8,
    pub length_counter_disabled: bool,
    pub enabled: bool
}

impl Channel {
    const fn new() -> Channel {
        Channel {
            length_counter: 0,
            length_counter_disabled: false,
            enabled: false
        }
    }

    fn reload_length_counter( &mut self, value: u8 ) {
        let length_counter_value_index = (value & 0b11111000) >> 3;
        let length_counter = LENGTH_COUNTER_LOOKUP_TABLE.peek( length_counter_value_index );
        self.length_counter = length_counter;
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum SquareChannelDuty {
    Duty125,            // 12.5%
    Duty250,            // 25.0%
    Duty500,            // 50.0%
    DutyInverted250     // Inverted 25%, which gives 75% with a different phase.
}

impl SquareChannelDuty {
    fn to_index( self ) -> usize {
        use self::SquareChannelDuty::*;
        match self {
            Duty125 => 0,
            Duty250 => 1,
            Duty500 => 2,
            DutyInverted250 => 3
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Direction {
    Positive,
    Negative
}

struct VolumeGenerator {
    is_manually_controlled: bool,
    should_loop: bool,

    period: u8,
    generated_volume: u8,
    countdown: u8,
    reset_on_next_clock: bool
}

impl VolumeGenerator {
    const fn new() -> VolumeGenerator {
        VolumeGenerator {
            is_manually_controlled: false,
            should_loop: false,

            period: 0,
            generated_volume: 0,
            countdown: 0,
            reset_on_next_clock: false
        }
    }

    fn poke_control( &mut self, value: u8 ) {
        self.period = value.get_bits( 0b00001111 );
        self.is_manually_controlled = is_b4_set( value );
        self.should_loop = is_b5_set( value ); // This is also mapped to the length_counter_disabled.
    }

    fn reset( &mut self ) {
        self.reset_on_next_clock = true;
    }

    fn volume( &self ) -> u8 {
        let value = if self.is_manually_controlled {
            self.period
        } else {
            self.generated_volume
        };

        debug_assert!( value <= 15 );
        value
    }

    fn clock( &mut self ) {
        if self.reset_on_next_clock == true {
            self.reset_on_next_clock = false;
            self.generated_volume = 15;
            self.countdown = self.period + 1;
            return;
        }

        if self.countdown != 0 {
            self.countdown -= 1;
        } else {
            // While the period was originally reloaded with V + 1
            // this is deliberately set to V.
            self.countdown = self.period;
            if self.generated_volume != 0 {
                self.generated_volume -= 1;
            } else {
                if self.should_loop == true {
                    self.generated_volume = 15;
                }
            }
        }
    }
}

struct ChannelSquare {
    base: Channel,
    duty: SquareChannelDuty,
    period: u16,

    volume_generator: VolumeGenerator,

    frequency_generator_enabled: bool,
    frequency_generator_was_reset: bool,
    frequency_generator_countdown_before_reset: u8,
    frequency_generator_countdown: u8,
    frequency_generator_period: u8,
    frequency_generator_direction: Direction,
    frequency_generator_shift_count: u8,

    timer: u16,
    current_duty_position: u8,

    is_second_channel: bool
}

impl ChannelSquare {
    const fn new( is_second_channel: bool ) -> ChannelSquare {
        ChannelSquare {
            base: Channel::new(),
            duty: SquareChannelDuty::Duty125,
            period: 0,

            volume_generator: VolumeGenerator::new(),

            frequency_generator_enabled: false,
            frequency_generator_was_reset: false,
            frequency_generator_countdown_before_reset: 0,
            frequency_generator_countdown: 0,
            frequency_generator_period: 0,
            frequency_generator_direction: Direction::Positive,
            frequency_generator_shift_count: 0,

            timer: 0,
            current_duty_position: 0,

            is_second_channel: is_second_channel
        }
    }

    fn poke_control( &mut self, value: u8 ) {
        self.volume_generator.poke_control( value );
        self.length_counter_disabled  = is_b5_set( value );
        self.duty = match (value & 0b11000000) >> 6 {
            0b00 => SquareChannelDuty::Duty125,
            0b01 => SquareChannelDuty::Duty250,
            0b10 => SquareChannelDuty::Duty500,
            0b11 => SquareChannelDuty::DutyInverted250,
            _ => unsafe { fast_unreachable!() }
        };
    }

    fn poke_period_low( &mut self, value: u8 ) {
        self.period.replace_bits( 0b00011111111, value as u16 );
    }

    fn poke_period_high( &mut self, value: u8 ) {
        if self.enabled {
            self.reload_length_counter( value );
        }

        self.period.replace_bits( 0b11100000000, value.get_bits( 0b00000111 ) as u16 );
        self.timer = self.period + 1;
        self.current_duty_position = 0;
        self.volume_generator.reset();
    }

    fn poke_frequency_generator( &mut self, value: u8 ) {
        self.frequency_generator_was_reset = true;
        self.frequency_generator_countdown_before_reset = self.frequency_generator_countdown;
        self.frequency_generator_enabled = is_b7_set( value );
        self.frequency_generator_period = value.get_bits( 0b01110000 );
        self.frequency_generator_direction = if is_b3_set( value ) { Direction::Negative } else { Direction::Positive };
        self.frequency_generator_shift_count = value.get_bits( 0b00000111 );
    }

    fn is_silent( &self ) -> bool {
        !self.enabled || self.period < 8 || self.length_counter == 0 || (self.frequency_generator_enabled == true && self.frequency_generator_output() >= 0x7FF)
    }

    fn output( &self ) -> u8 {
        if self.is_silent() {
            0
        } else {
            SQUARE_CHANNEL_DUTY_TABLE.peek( self.duty.to_index() ).peek( self.current_duty_position ) * self.volume()
        }
    }

    fn clock_volume_generator( &mut self ) {
        self.volume_generator.clock();
    }

    fn volume( &self ) -> u8 {
        self.volume_generator.volume()
    }

    fn clock_timer( &mut self ) {
        if self.timer == 0 {
            if self.current_duty_position == 0 {
                self.current_duty_position = 7;
            } else {
                self.current_duty_position -= 1;
            }
            self.timer = self.period.wrapping_add( 1 );
        } else {
            self.timer -= 1;
        }
    }

    fn adjust_period_based_on_frequency_generator( &mut self ) {
        if self.frequency_generator_shift_count != 0 {
            let target_period = self.frequency_generator_output();
            if self.period < 8 || target_period > 0x7FF {
                return;
            }

            self.period = target_period;
        }
    }

    fn clock_frequency_generator( &mut self ) {
        if self.frequency_generator_was_reset == true {
            self.frequency_generator_was_reset = false;
            self.frequency_generator_countdown = self.frequency_generator_period;

            if self.frequency_generator_countdown_before_reset == 0 && self.frequency_generator_enabled == true {
                self.adjust_period_based_on_frequency_generator();
            }
        } else {
            if self.frequency_generator_countdown != 0 {
                self.frequency_generator_countdown -= 1;
            } else if self.frequency_generator_enabled == true {
                self.frequency_generator_countdown = self.frequency_generator_period;
                self.adjust_period_based_on_frequency_generator();
            }
        }
    }

    fn frequency_generator_output( &self ) -> u16 {
        let period_delta = self.period >> self.frequency_generator_shift_count;
        match self.frequency_generator_direction {
            Direction::Positive => self.period.wrapping_add( period_delta ),
            Direction::Negative => {
                if self.is_second_channel == true {
                    self.period.wrapping_sub( period_delta )
                } else {
                    self.period.wrapping_sub( period_delta ).wrapping_sub( 1 )
                }
            }
        }
    }
}

struct ChannelTriangle {
    base: Channel,
    period: u16,
    linear_counter: u8,
    linear_counter_reload_value: u8,
    linear_counter_reload_flag: bool,
    current_position: u8,
    timer: u16
}

impl ChannelTriangle {
    const fn new() -> ChannelTriangle {
        ChannelTriangle {
            base: Channel::new(),
            period: 0,

            linear_counter: 0,
            linear_counter_reload_value: 0,
            linear_counter_reload_flag: false,

            current_position: 0,
            timer: 0
        }
    }

    fn poke_control( &mut self, value: u8 ) {
        self.length_counter_disabled = is_b7_set( value );
        self.linear_counter_reload_value = value.get_bits( 0b01111111 );
    }

    fn poke_period_low( &mut self, value: u8 ) {
        self.period.replace_bits( 0b00011111111, value as u16 );
    }

    fn poke_period_high( &mut self, value: u8 ) {
        if self.enabled {
            self.reload_length_counter( value );
        }

        self.period.replace_bits( 0b11100000000, value.get_bits( 0b00000111 ) as u16 );
        self.timer = self.period + 1;
        self.linear_counter_reload_flag = true;
    }

    // The frequency of the triangle is as follows:
    //     f = f_cpu / (32 * (period + 1))
    // where f_cpu (for NTSC) is 1789.773 kHz, so that gives us:
    //   0 => 55.93 kHz (55.93 Khz, 167.69 Khz, 279.65 Khz, ...)
    //   1 => 27.97 kHz (27.97 Khz, 83.91 Khz, 139.85 Khz, ...)
    //   2 => 18.64 kHz (18.64 Khz, 55.92 Khz, 93.20 Khz, ...)
    //   3 => 13.98 kHz (13.98 Khz, 41.94 Khz, 69.9 Khz, ...)
    //   4 => 11.19 kHz (11.19 kHz, 33.57 Khz, 55.95 Khz, ...)
    //   ...
    // In general anything over ~20 kHz is ultrasonic, and cannot be heard
    // by a human ear, which makes the period of 2 the minimum value when
    // the triangle can actually be heard (in which case it'd be a single
    // sine wave rather than a triangle, since the next harmonic after
    // the fundamental one is already over 20 kHz).
    //
    // Silencing the triangle on periods less than 2 isn't exactly accurate
    // as then the triangle output is supposedly effectively equal to 7.5.
    fn output( &self ) -> u8 {
        if self.period < 2 {
            return 7;
        }

        // The sequencer here goes like this:
        //  15, 14, ..., 1, 0, 0, 1, 2, ..., 15, 15, ...
        // TODO: Make a proper triangle wave here as a hack.
        if self.current_position <= 15 {
            15 - self.current_position
        } else {
            self.current_position - 15 - 1
        }
    }

    fn should_linear_counter_be_perpetually_reloaded( &self ) -> bool {
        self.length_counter_disabled // The same bit is mapped to two flags.
    }

    fn clock_timer( &mut self ) {
        if self.timer == 0 {
            if self.linear_counter != 0 && self.length_counter != 0 {
                self.current_position += 1;
                if self.current_position >= 32 {
                    self.current_position = 0;
                }
            }

            self.timer = self.period + 1;
        } else {
            self.timer -= 1;
        }
    }

    fn clock_linear_counter( &mut self ) {
        if self.linear_counter_reload_flag {
            self.linear_counter = self.linear_counter_reload_value;
        } else if self.linear_counter != 0 {
            self.linear_counter -= 1;
        }

        if self.should_linear_counter_be_perpetually_reloaded() == false {
            self.linear_counter_reload_flag = false;
        }
    }
}

struct ChannelNoise {
    base: Channel,
    volume_generator: VolumeGenerator,
    mode_flag: bool,
    period: u16,
    timer: u16,
    feedback_register: u16 // A 15-bit shift register used to generate noise.
}

impl ChannelNoise {
    const fn new() -> ChannelNoise {
        ChannelNoise {
            base: Channel::new(),
            volume_generator: VolumeGenerator::new(),
            mode_flag: false,
            period: 0,
            timer: 0,
            feedback_register: 1 // Set to 1 on power-up.
        }
    }

    fn poke_control( &mut self, value: u8 ) {
        self.length_counter_disabled = is_b5_set( value );
        self.volume_generator.poke_control( value );
    }

    fn poke_period( &mut self, value: u8 ) {
        self.mode_flag = is_b7_set( value );

        let period_table_index = value.get_bits( 0b00001111 );
        self.period = NOISE_CHANNEL_NTSC_PERIOD_TABLE.peek( period_table_index );
    }

    fn poke_counter( &mut self, value: u8 ) {
        if self.enabled {
            self.reload_length_counter( value );
        }
        self.volume_generator.reset();
    }

    fn is_silent( &self ) -> bool {
        !self.enabled || self.length_counter == 0 || (self.feedback_register & 1) == 1
    }

    fn output( &self ) -> u8 {
        if self.is_silent() {
            0
        } else {
            self.volume_generator.volume()
        }
    }

    fn clock_volume_generator( &mut self ) {
        self.volume_generator.clock();
    }

    fn clock_timer( &mut self ) {
        if self.timer == 0 {
            let bit_a = self.feedback_register.get_bits( 0b00000001 );
            let bit_b = if self.mode_flag {
                self.feedback_register.get_bits( 0b01000000 )
            } else {
                self.feedback_register.get_bits( 0b00000010 )
            };

            self.feedback_register = (self.feedback_register >> 1) | ((bit_a ^ bit_b) << 14);
            self.timer = self.period + 1;
        } else {
            self.timer -= 1;
        }
    }
}

struct ChannelDeltaModulation {
    pub enabled: bool,

    generate_irq_when_sample_ends: bool,
    loop_when_sample_ends: bool,
    delta_playback_period: u16,
    initial_sample_address: u16,
    initial_sample_byte_length: u16,

    sample_buffer: Option< u8 >,
    sample_shift_register: u8,
    sample_shift_register_bits_remaining: u8,
    sample_bytes_remaining: u16,
    current_sample_address: u16,

    output: u8,
    timer: u16,
    silence_flag: bool,
    interrupt_occured_flag: bool
}

// The DMC channel plays back DPCM samples stored in memory as 1-bit deltas.
// A bit of 1 will increment the output; a bit of 0 will decrement the output.
impl ChannelDeltaModulation {
    const fn new() -> ChannelDeltaModulation {
        ChannelDeltaModulation {
            enabled: false,

            generate_irq_when_sample_ends: false,
            loop_when_sample_ends: false,
            delta_playback_period: 0,
            initial_sample_address: 0xC000,
            initial_sample_byte_length: 1,

            sample_buffer: None,
            sample_shift_register: 0,
            sample_shift_register_bits_remaining: 8,
            sample_bytes_remaining: 0,
            current_sample_address: 0xC000,

            output: 0,
            timer: 0,
            silence_flag: true,
            interrupt_occured_flag: false
        }
    }

    fn poke_control( &mut self, value: u8 ) {
        self.generate_irq_when_sample_ends = is_b7_set( value );
        self.loop_when_sample_ends = is_b6_set( value );

        let index = value.get_bits( 0b00001111 );
        // Divided by two since the timer runs only on every second CPU cycle.
        self.delta_playback_period = DMC_CHANNEL_NTSC_DELTA_PLAYBACK_PERIOD_TABLE.peek( index ) / 2 - 1;

        if self.generate_irq_when_sample_ends == false {
            self.interrupt_occured_flag = false;
        }
    }

    fn poke_direct_load( &mut self, value: u8 ) {
        self.output = value & 0b01111111;
    }

    fn poke_sample_address( &mut self, value: u8 ) {
        self.initial_sample_address = 0xC000 | (value as u16 * 64);
    }

    fn poke_sample_length( &mut self, value: u8 ) {
        self.initial_sample_byte_length = value as u16 * 16 + 1;
    }

    fn set_enabled( &mut self, is_enabled: bool ) {
        self.enabled = is_enabled;

        self.interrupt_occured_flag = false;
        if self.enabled == false {
            self.sample_bytes_remaining = 0;
        } else {
            if self.sample_bytes_remaining == 0 {
                self.current_sample_address = self.initial_sample_address;
                self.sample_bytes_remaining = self.initial_sample_byte_length;
            }
        }
    }

    fn output( &self ) -> u8 {
        self.output
    }

    fn are_any_samples_remaining( &self ) -> bool {
        self.sample_bytes_remaining > 0
    }

    fn should_activate_dma( &self ) -> bool {
        if self.sample_buffer.is_some() {
            return false;
        }

        if self.sample_bytes_remaining == 0 {
            return false;
        }

        return true;
    }

    fn on_dma_finished( &mut self, sample: u8 ) {
        self.sample_buffer = Some( sample );

        self.current_sample_address.wrapping_inc();
        self.current_sample_address = self.current_sample_address | 0x8000;
        self.sample_bytes_remaining -= 1;

        if self.sample_bytes_remaining == 0 {
            if self.loop_when_sample_ends {
                self.current_sample_address = self.initial_sample_address;
                self.sample_bytes_remaining = self.initial_sample_byte_length;
            } else if self.generate_irq_when_sample_ends {
                self.interrupt_occured_flag = true;
            }
        }
    }

    fn clock_timer( &mut self ) {
        if self.timer == 0 {
            self.timer = self.delta_playback_period;

            if self.silence_flag == false {
                if (self.sample_shift_register & 1) == 1 {
                    if self.output <= 125 {
                        self.output += 2;
                    }
                } else {
                    if self.output >= 2 {
                        self.output -= 2;
                    }
                }

                self.sample_shift_register >>= 1;
            }

            self.sample_shift_register_bits_remaining -= 1;
            if self.sample_shift_register_bits_remaining == 0 {
                self.sample_shift_register_bits_remaining = 8;

                if let Some( sample ) = self.sample_buffer.take() {
                    self.silence_flag = false;
                    self.sample_shift_register = sample;
                } else {
                    self.silence_flag = true;
                }
            }
        } else {
            self.timer -= 1;
        }
    }

    fn has_interrupt_occured( &self ) -> bool {
        self.interrupt_occured_flag
    }
}

impl_deref!( ChannelSquare, Channel, base );
impl_deref!( ChannelTriangle, Channel, base );
impl_deref!( ChannelNoise, Channel, base );

trait Private: Sized + Context {
    fn sample_rate( &self ) -> u32 {
        44100 // TODO: Make this somewhat configurable.
    }

    fn execute( &mut self ) {
        self.clock_channel_timers();
        self.clock_output();
        self.clock_sequencer();
    }

    fn clock_channel_timers( &mut self ) {
        // Only the triangle channel's timer is clocked on every CPU cycle;
        // the rest are only clocked on every second CPU cycle.
        self.state_mut().channel_triangle.clock_timer();

        if self.is_on_odd_cycle() {
            self.state_mut().channel_square_1.clock_timer();
            self.state_mut().channel_square_2.clock_timer();
            self.state_mut().channel_noise.clock_timer();
            self.state_mut().channel_delta_modulation.clock_timer();

            if self.state().channel_delta_modulation.should_activate_dma() {
                let address = self.state().channel_delta_modulation.current_sample_address;
                self.activate_dma( address );
            }
        }
    }

    fn get_output( &self ) -> F32 {
        let raw_sample_square_1 = self.state().channel_square_1.output();
        let raw_sample_square_2 = self.state().channel_square_2.output();
        let raw_sample_triangle = self.state().channel_triangle.output();
        let raw_sample_noise = self.state().channel_noise.output();
        let raw_sample_dmc = self.state().channel_delta_modulation.output();

        let output_square = if raw_sample_square_1 == 0 && raw_sample_square_2 == 0 {
            f32!(0.0)
        } else {
            f32!(95.88) / (f32!(8128.0) / (u8_to_f32( raw_sample_square_1 ) + u8_to_f32( raw_sample_square_2 )) + f32!(100.0))
        };

        let output_rest = if raw_sample_triangle == 0 && raw_sample_noise == 0 && raw_sample_dmc == 0 {
            f32!(0.0)
        } else {
            f32!(159.79) / (f32!(1.0) / ((u8_to_f32( raw_sample_triangle ) / f32!(8227.0)) + (u8_to_f32( raw_sample_noise ) / f32!(12241.0)) + (u8_to_f32( raw_sample_dmc ) / f32!(22638.0))) + f32!(100.0))
        };

        return output_square + output_rest;
    }

    fn clock_output( &mut self ) {
        let native_sampling_rate: F64 = f64!(21477.272) / f64!(12.0); // TODO: PAL support.
        let internal_sampling_rate: F64 = f64!(352.8); // TODO: This is closely related to the filter used; make the filter configurable.
        let sample_every: F64 = native_sampling_rate / internal_sampling_rate;

        self.state_mut().sampling_counter += f64!(1.0);
        if self.state().sampling_counter < sample_every {
            return;
        }
        self.state_mut().sampling_counter -= sample_every;

        let output = self.get_output();
        let filtered_output = self.state_mut().filter.apply( output );
        self.state_mut().decimation_counter += 1;
        if self.state().decimation_counter >= 8 {
            self.state_mut().decimation_counter = 0;
            self.on_sample( filtered_output );
        }
    }

    fn clock_sequencer( &mut self ) {
        self.state_mut().cycles_to_next_step -= 1;
        if self.state_mut().cycles_to_next_step == 0 {
            let entry = &self.state().sequencer_table[ self.state().sequencer_current_step as usize ];
            if entry.clk_linear_counter() {
                self.state_mut().channel_triangle.clock_linear_counter();
            }

            if entry.clk_frequency_generator() {
                self.state_mut().channel_square_1.clock_frequency_generator();
                self.state_mut().channel_square_2.clock_frequency_generator();
            }

            if entry.clk_volume_generator() {
                self.state_mut().channel_square_1.clock_volume_generator();
                self.state_mut().channel_square_2.clock_volume_generator();
                self.state_mut().channel_noise.clock_volume_generator();
            }

            if entry.clk_length_counter() {
                self.clock_counter_units();
            }

            if entry.clk_irq() {
                if !self.state().inhibit_interrupts {
                    self.state_mut().interrupt_occured_flag = true;
                    self.set_irq_line( true );
                }
            }

            self.state_mut().sequencer_current_step = entry.next_sequence();
            self.state_mut().cycles_to_next_step = self.state().sequencer_table[ self.state().sequencer_current_step as usize ].cycles();
        }

        if let Some( (sequencer_mode, made_on_odd_cycle) ) = self.state_mut().sequencer_reconfigured.take() {
            let table = match sequencer_mode {
                SequencerMode::Mode0 => SEQUENCER_STEPS_MODE_0,
                SequencerMode::Mode1 => SEQUENCER_STEPS_MODE_1
            };

            self.state_mut().sequencer_table = table;
            self.state_mut().sequencer_current_step = if made_on_odd_cycle { 0 } else { 1 };
            self.state_mut().cycles_to_next_step = self.state().sequencer_table[ self.state().sequencer_current_step as usize ].cycles();
        }
    }

    fn clock_counter_units( &mut self ) {
        if self.state_mut().channel_square_1.length_counter_disabled == false && self.state().channel_square_1.length_counter != 0 {
            self.state_mut().channel_square_1.length_counter -= 1;
        }

        if self.state().channel_square_2.length_counter_disabled == false && self.state().channel_square_2.length_counter != 0 {
            self.state_mut().channel_square_2.length_counter -= 1;
        }

        if self.state().channel_triangle.length_counter_disabled == false && self.state().channel_triangle.length_counter != 0 {
            self.state_mut().channel_triangle.length_counter -= 1;
        }

        if self.state().channel_noise.length_counter_disabled == false && self.state().channel_noise.length_counter != 0 {
            self.state_mut().channel_noise.length_counter -= 1;
        }
    }

    fn update_irq_line( &mut self ) {
        let irq_state = self.state().interrupt_occured_flag || self.state().channel_delta_modulation.has_interrupt_occured();
        self.set_irq_line( irq_state );
    }

    fn poke_frame_sequencer_ctrl( &mut self, value: u8, is_odd_cycle: bool ) {
        self.state_mut().sequencer_mode = if is_b7_set( value ) { SequencerMode::Mode1 } else { SequencerMode::Mode0 };
        self.state_mut().inhibit_interrupts = is_b6_set( value );
        // Rest of the bits are unused.

        if self.state().inhibit_interrupts {
            self.state_mut().interrupt_occured_flag = false;
            self.update_irq_line();
        }

        self.state_mut().sequencer_reconfigured = Some( (self.state().sequencer_mode, is_odd_cycle) );
    }

    fn poke_control( &mut self, value: u8 ) {
        self.state_mut().channel_square_1.enabled = is_b0_set( value );
        self.state_mut().channel_square_2.enabled = is_b1_set( value );
        self.state_mut().channel_triangle.enabled = is_b2_set( value );
        self.state_mut().channel_noise.enabled    = is_b3_set( value );
        self.state_mut().channel_delta_modulation.set_enabled( is_b4_set( value ) );
        // Rest of the bits are unused.

        if !self.state().channel_square_1.enabled {
            self.state_mut().channel_square_1.length_counter = 0;
        }
        if !self.state().channel_square_2.enabled {
            self.state_mut().channel_square_2.length_counter = 0;
        }
        if !self.state().channel_triangle.enabled {
            self.state_mut().channel_triangle.length_counter = 0;
        }
        if !self.state().channel_noise.enabled {
            self.state_mut().channel_noise.length_counter = 0;
        }
    }

    fn peek_status( &mut self ) -> u8 {
        let value =
            to_bit( 0, self.state().channel_square_1.length_counter != 0 ) |
            to_bit( 1, self.state().channel_square_2.length_counter != 0 ) |
            to_bit( 2, self.state().channel_triangle.length_counter != 0 ) |
            to_bit( 3, self.state().channel_noise.length_counter != 0 ) |
            to_bit( 4, self.state().channel_delta_modulation.are_any_samples_remaining() ) |
            to_bit( 6, self.state().interrupt_occured_flag ) |
            to_bit( 7, self.state().channel_delta_modulation.has_interrupt_occured() );

        self.state_mut().interrupt_occured_flag = false;
        self.update_irq_line();
        value
    }

    fn poke_square_1_ctrl( &mut self, value: u8 ) {
        self.state_mut().channel_square_1.poke_control( value );
    }

    fn poke_square_1_frequency_generator( &mut self, value: u8 ) {
        self.state_mut().channel_square_1.poke_frequency_generator( value );
    }

    fn poke_square_1_period_low( &mut self, value: u8 ) {
        self.state_mut().channel_square_1.poke_period_low( value );
    }

    fn poke_square_1_period_high( &mut self, value: u8 ) {
        self.state_mut().channel_square_1.poke_period_high( value );
    }

    fn poke_square_2_ctrl( &mut self, value: u8 ) {
        self.state_mut().channel_square_2.poke_control( value );
    }

    fn poke_square_2_frequency_generator( &mut self, value: u8 ) {
        self.state_mut().channel_square_2.poke_frequency_generator( value );
    }

    fn poke_square_2_period_low( &mut self, value: u8 ) {
        self.state_mut().channel_square_2.poke_period_low( value );
    }

    fn poke_square_2_period_high( &mut self, value: u8 ) {
        self.state_mut().channel_square_2.poke_period_high( value );
    }

    fn poke_triangle_ctrl( &mut self, value: u8 ) {
        self.state_mut().channel_triangle.poke_control( value );
    }

    fn poke_triangle_period_low( &mut self, value: u8 ) {
        self.state_mut().channel_triangle.poke_period_low( value );
    }

    fn poke_triangle_period_high( &mut self, value: u8 ) {
        self.state_mut().channel_triangle.poke_period_high( value );
    }

    fn poke_noise_ctrl( &mut self, value: u8 ) {
        self.state_mut().channel_noise.poke_control( value );
    }

    fn poke_noise_period( &mut self, value: u8 ) {
        self.state_mut().channel_noise.poke_period( value );
    }

    fn poke_noise_counter_ctrl( &mut self, value: u8 ) {
        self.state_mut().channel_noise.poke_counter( value );
    }

    fn poke_dmc_ctrl( &mut self, value: u8 ) {
        self.state_mut().channel_delta_modulation.poke_control( value );
    }

    fn poke_dmc_direct_load( &mut self, value: u8 ) {
        self.state_mut().channel_delta_modulation.poke_direct_load( value );
    }

    fn poke_dmc_sample_address( &mut self, value: u8 ) {
        self.state_mut().channel_delta_modulation.poke_sample_address( value );
    }

    fn poke_dmc_sample_length( &mut self, value: u8 ) {
        self.state_mut().channel_delta_modulation.poke_sample_length( value );
    }

    fn on_delta_modulation_channel_dma_finished( &mut self, value: u8 ) {
        self.state_mut().channel_delta_modulation.on_dma_finished( value );
    }
}
