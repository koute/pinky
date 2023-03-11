extern crate nes;
extern crate emumisc;

#[macro_use]
extern crate libretro_backend;

use libretro_backend::{CoreInfo, AudioVideoInfo, PixelFormat, GameData, LoadGameResult, Region, RuntimeHandle, JoypadButton};
use nes::{Palette, ControllerPort, Button};

struct PinkyCore {
    state: nes::State,
    palette: [u32; 512],
    framebuffer: [u32; 256 * 240],
    audio_buffer: Vec< i16 >,
    game_data: Option< GameData >
}

impl nes::Context for PinkyCore {
    #[inline]
    fn state_mut( &mut self ) -> &mut nes::State {
        &mut self.state
    }

    #[inline]
    fn state( &self ) -> &nes::State {
        &self.state
    }

    #[inline]
    fn on_audio_sample( &mut self, sample: f32 ) {
        let value = if sample >= 1.0 {
            32767
        } else if sample <= -1.0 {
            -32768
        } else {
            (sample * 32767.0) as i16
        };

        self.audio_buffer.push( value );
        self.audio_buffer.push( value );
    }
}

fn palette_to_argb( palette: &Palette ) -> [u32; 512] {
    let mut output = [0; 512];
    for (index, out) in output.iter_mut().enumerate() {
        let (r, g, b) = palette.get_rgb( index as u16 );
        *out = ((r as u32) << 16) |
               ((g as u32) <<  8) |
               ((b as u32)      );
    }

    output
}

impl PinkyCore {
    fn new() -> PinkyCore {
        PinkyCore {
            state: nes::State::new(),
            palette: palette_to_argb( &Palette::default() ),
            framebuffer: [0; 256 * 240],
            audio_buffer: Vec::with_capacity( 44100 ),
            game_data: None
        }
    }
}

impl Default for PinkyCore {
    fn default() -> Self {
        Self::new()
    }
}

impl libretro_backend::Core for PinkyCore {
    fn info() -> CoreInfo {
        CoreInfo::new( "Pinky", env!( "CARGO_PKG_VERSION" ) )
            .supports_roms_with_extension( "nes" )
    }

    fn on_load_game( &mut self, game_data: GameData ) -> LoadGameResult {
        if game_data.is_empty() {
            return LoadGameResult::Failed( game_data );
        }

        let result = if let Some( data ) = game_data.data() {
            nes::Interface::load_rom( self, data )
        } else if let Some( path ) = game_data.path() {
            let data = match std::fs::read( path ) {
                Ok( data ) => data,
                Err( _ ) => {
                    return LoadGameResult::Failed( game_data );
                }
            };
            nes::Interface::load_rom( self, &data )
        } else {
            unreachable!();
        };

        match result {
            Ok( _ ) => {
                self.game_data = Some( game_data );
                let av_info = AudioVideoInfo::new()
                    .video( 256, 240, 60.0, PixelFormat::ARGB8888 )
                    .audio( 44100.0 )
                    .region( Region::NTSC );

                LoadGameResult::Success( av_info )
            },
            Err( _ ) => {
                LoadGameResult::Failed( game_data )
            }
        }
    }

    fn on_unload_game( &mut self ) -> GameData {
        self.game_data.take().unwrap()
    }

    fn on_run( &mut self, handle: &mut RuntimeHandle ) {
        macro_rules! update_controllers {
            ( $( $button:ident ),+ ) => (
                $(
                    nes::Interface::set_button_state( self, ControllerPort::First, Button::$button, handle.is_joypad_button_pressed( 0, JoypadButton::$button ) );
                    nes::Interface::set_button_state( self, ControllerPort::Second, Button::$button, handle.is_joypad_button_pressed( 1, JoypadButton::$button ) );
                )+
            )
        }

        update_controllers!( A, B, Start, Select, Left, Up, Right, Down );

        if let Err( error ) = nes::Interface::execute_for_a_frame( self ) {
            println!( "Execution error: {}", error );
            return;
        }

        let framebuffer = self.state.framebuffer();
        for (pixel_in, pixel_out) in framebuffer.iter().zip( self.framebuffer.iter_mut() ) {
            *pixel_out = self.palette[ pixel_in.full_color_index() as usize ];
        }

        let video_frame = emumisc::as_bytes( &self.framebuffer[..] );
        handle.upload_video_frame( video_frame );

        handle.upload_audio_frame( &self.audio_buffer[..] );
        self.audio_buffer.clear();
    }

    fn on_reset( &mut self ) {
        nes::Interface::soft_reset( self );
    }
}

libretro_core!( PinkyCore );
