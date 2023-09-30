use std::env;
use std::path::PathBuf;

use sdl2;
use sdl2::keyboard::Keycode;
use sdl2::audio::{AudioQueue, AudioSpecDesired};

use serde_json;

use nes;
use nes::{Interface, Framebuffer, ControllerPort, Button, Palette};
use frame_limiter::FrameLimiter;
use renderer::{Renderer, Texture, ImageBuffer};

macro_rules! json_object {
    ( $( $key:expr => $value:expr ),* ) => ({
        use serde_json;
        use std::collections::BTreeMap;

        let mut object = BTreeMap::new();
        $( object.insert( $key.to_owned(), serde_json::value::to_value( $value ) ); )*
        serde_json::Value::Object( object )
    })
}

fn keycode_to_button( keycode: Keycode ) -> Option< Button > {
    let button = match keycode {
        Keycode::Left => Button::Left,
        Keycode::Right => Button::Right,
        Keycode::Up => Button::Up,
        Keycode::Down => Button::Down,
        Keycode::Return => Button::Start,
        Keycode::RShift => Button::Select,
        Keycode::LCtrl => Button::A,
        Keycode::LAlt => Button::B,
        _ => return None
    };

    Some( button )
}

struct VirtualNES {
    state: nes::State,
    cycle: u64,
    frame: u64,
    audio_buffer: Vec< f32 >
}

impl nes::Context for VirtualNES {
    fn state( &self ) -> &nes::State {
        &self.state
    }

    fn state_mut( &mut self ) -> &mut nes::State {
        &mut self.state
    }

    fn on_cycle( &mut self ) {
        self.cycle += 1;
    }

    fn on_frame( &mut self ) {
        self.frame += 1;
    }

    #[inline]
    fn on_audio_sample( &mut self, sample: f32 ) {
        self.audio_buffer.push( sample );
    }
}

fn md5sum< T: AsRef< [u8] > >( data: T ) -> String {
    let raw_digest = md5::compute( data.as_ref() );
    let mut digest = String::with_capacity( 2 * 16 );
    for byte in raw_digest.iter() {
        digest.push_str( &format!( "{:02x}", byte ) );
    }
    digest
}

pub struct UserInterface {
    #[allow(dead_code)]
    sdl_context: sdl2::Sdl,
    audio_context: sdl2::AudioSubsystem,
    event_pump: sdl2::EventPump,
    audio_device: Option< AudioQueue< f32 > >,
    renderer: Renderer,
    texture: Texture,
    image_buffer: ImageBuffer,
    running: bool,
    last_framebuffer: Option< Framebuffer >,
    frame_limiter: FrameLimiter,
    is_emulating: bool,
    palette: Palette,
    nes: VirtualNES,
    rom_filename: PathBuf,
    replaying: bool,
    recording: Vec< Button >,
    recording_position: usize,
    no_limiter: bool,
}

impl UserInterface {
    fn create() -> UserInterface {
        let sdl_context = sdl2::init().unwrap();
        let video_context = sdl_context.video().unwrap();
        let audio_context = sdl_context.audio().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();
        let window = video_context.window( "Pinky", 256 * 4, 240 * 4 ).position_centered().opengl().build().unwrap();

        let mut renderer = Renderer::new( window.renderer().build().unwrap() );
        let texture = Texture::new_streaming( &mut renderer, 256, 240 );

        if !video_context.gl_set_swap_interval( -1 ) {
            video_context.gl_set_swap_interval( 1 );
        }

        UserInterface {
            sdl_context: sdl_context,
            audio_context: audio_context,
            event_pump: event_pump,
            audio_device: None,
            renderer: renderer,
            texture: texture,
            image_buffer: ImageBuffer::new( 256, 240 ),
            running: true,
            last_framebuffer: Some( Framebuffer::default() ),
            frame_limiter: FrameLimiter::new( 60 ),
            is_emulating: false,
            palette: Palette::default(),
            nes: VirtualNES {
                cycle: 0,
                frame: 0,
                state: nes::State::new(),
                audio_buffer: Vec::new()
            },
            rom_filename: PathBuf::new(),
            replaying: false,
            recording: Vec::new(),
            recording_position: 0,
            no_limiter: false,
        }

    }

    fn open_audio( &mut self ) {
        let desired_spec = AudioSpecDesired {
            freq: Some( 44100 ),
            channels: Some( 1 ),
            samples: None
        };

        if let Ok( device ) = self.audio_context.open_queue( None, &desired_spec ) {
            device.resume();
            self.audio_device = Some( device );
        }
    }

    fn generate_testfile( &mut self ) {
        use std::fs::File;
        use std::io::Write;

        let mut array = [0; 256 * 240];
        for (pixel, out) in self.last_framebuffer.as_ref().unwrap().iter().zip( array.iter_mut() ) {
            *out = pixel.base_color_index();
        }

        let mut path = PathBuf::from( "/tmp" );
        path.push( self.rom_filename.file_name().unwrap() );

        self.nes.cycle = 0;
        self.nes.frame = 0;

        self.nes = VirtualNES {
            cycle: 0,
            frame: 0,
            state: nes::State::new(),
            audio_buffer: Vec::new()
        };
        self.nes.load_rom( &std::fs::read( &self.rom_filename ).unwrap() ).unwrap();

        let frame;
        loop {
            if let Err( error ) = self.nes.execute_until_vblank() {
                println!( "Execution error: {}", error );
                self.is_emulating = false;
                return;
            }

            let framebuffer = self.nes.framebuffer();
            if self.last_framebuffer.as_ref().unwrap() == framebuffer {
                frame = self.nes.frame;
                println!( "Cycle: {}", self.nes.cycle );
                println!( "Frame: {}", self.nes.frame );
                break;
            }
        }

        let romfile_md5sum = {
            use std::io::Read;
            let mut fp = File::open( &self.rom_filename ).unwrap();
            let mut data = Vec::new();
            fp.read_to_end( &mut data ).unwrap();

            md5sum( data )
        };

        let framebuffer_md5sum = md5sum( &array[..] );
        path.set_extension( "json" );

        let obj = json_object!(
            "romfile_md5sum" => &romfile_md5sum,
            "test" => &json_object!(
                "elapsed_frames" => &frame,
                "expected_framebuffer_md5sum" => &framebuffer_md5sum
            )
        );

        let mut fp = File::create( &path ).unwrap();
        serde_json::to_writer_pretty( &mut fp, &obj ).unwrap();
        writeln!( &mut fp, "" ).unwrap();
    }

    fn handle_sdl2_event( &mut self, event: sdl2::event::Event ) {
        use sdl2::event::Event;

        match event {
            Event::Quit {..} | Event::KeyDown { keycode: Some( Keycode::Escape ), .. } => {
                self.running = false
            },
            Event::KeyDown { keycode: keycode @ Some( .. ), .. } => {
                if let Some( button ) = keycode_to_button( keycode.unwrap() ) {
                    if !self.replaying {
                        self.nes.press( ControllerPort::First, button );
                    }
                } else if keycode == Some( Keycode::F10 ) {
                    self.generate_testfile();
                }
            },
            Event::KeyUp { keycode: keycode @ Some( .. ), .. } => {
                if !self.replaying {
                    if let Some( button ) = keycode_to_button( keycode.unwrap() ) {
                        self.nes.release( ControllerPort::First, button );
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_events( &mut self ) {
        while let Some( event ) = self.event_pump.poll_event() {
            self.handle_sdl2_event( event );
        }
    }

    fn emulate( &mut self ) {
        if !self.is_emulating {
            return;
        }

        if let Err( error ) = self.nes.execute_until_vblank() {
            println!( "Execution error: {}", error );
            self.is_emulating = false;
            return
        }

        let last_framebuffer = self.last_framebuffer.take().unwrap();
        let framebuffer = self.nes.swap_framebuffer( last_framebuffer );
        framebuffer.convert_to_abgr( &self.palette, &mut self.image_buffer );

        self.texture.update( &self.image_buffer );
        self.last_framebuffer = Some( framebuffer );
    }

    pub fn run( &mut self ) {
        self.renderer.set_logical_size( 256, 240 ).unwrap();

        let mut no_audio = false;
        for arg in env::args().skip(1) {
            if arg == "--no-limiter" {
                self.no_limiter = true;
                continue;
            }

            if arg == "--no-audio" {
                no_audio = true;
                continue;
            }

            println!( "Loading '{}'...", arg );
            let data = std::fs::read( &arg ).unwrap();
            if data.len() >= 16 && u32::from_le_bytes( [data[0], data[1], data[2], data[3]] ) == 0x1a53454e {
                self.rom_filename = PathBuf::from( &arg );
                self.nes.load_rom( &data ).unwrap();

                self.is_emulating = true;
                self.image_buffer.clear();
                self.texture.update( &self.image_buffer );
            } else if data.starts_with( b"[Input]" ) || arg.ends_with( "Input Log.txt" ) {
                let mut input = Vec::new();
                for line in std::str::from_utf8( &data ).unwrap().trim().lines().skip(2) {
                    if line == "[/Input]" {
                        break;
                    }

                    let mut button = Button::empty();
                    for ch in line.chars() {
                        if ch == '|' || ch == '.' {
                            continue;
                        }

                        button |= match ch {
                            'U' => Button::Up,
                            'D' => Button::Down,
                            'L' => Button::Left,
                            'R' => Button::Right,
                            'S' => Button::Start,
                            's' => Button::Select,
                            'B' => Button::B,
                            'A' => Button::A,
                            _ => panic!("unknown key: '{}'", ch)
                        };
                    }

                    input.push( button );
                }

                self.replaying = true;
                self.recording = input;
            } else if arg.ends_with( ".fm2" ) {
                let mut input = Vec::new();
                for line in std::str::from_utf8( &data ).unwrap().trim().lines().skip(2) {
                    if !line.starts_with( '|' ) {
                        continue;
                    }

                    let mut button = Button::empty();
                    for ch in line.chars() {
                        if ch == '|' || ch == '.' || ch == '0' {
                            continue;
                        }

                        button |= match ch {
                            'U' => Button::Up,
                            'D' => Button::Down,
                            'L' => Button::Left,
                            'R' => Button::Right,
                            'T' => Button::Start,
                            'S' => Button::Select,
                            'B' => Button::B,
                            'A' => Button::A,
                            _ => panic!("unknown key: '{}'", ch)
                        };
                    }

                    input.push( button );
                }

                self.replaying = true;
                self.recording = input;
            } else {
                panic!( "unhandled argument: {}", arg );
            }
        }

        if !no_audio {
            self.open_audio();
        }

        while self.running {
            self.handle_events();

            let force = self.no_limiter || self.audio_device.as_ref().map( |device| device.size() < 44100 / 2 ).unwrap_or( false );
            if force {
                self.run_for_a_frame();
            } else if self.frame_limiter.begin() {
                self.run_for_a_frame();
                self.frame_limiter.end();
            }
        }
    }

    fn run_for_a_frame( &mut self ) {
        if self.replaying && self.recording_position < self.recording.len() {
            let buttons = self.recording[ self.recording_position ];
            self.recording_position += 1;
            self.nes.set_all_buttons( ControllerPort::First, buttons );
        }

        self.emulate();

        self.renderer.clear();
        if self.is_emulating {
            self.renderer.blit( &self.texture );
        }

        if let Some( audio_device ) = self.audio_device.as_mut() {
            audio_device.queue( self.nes.audio_buffer.as_slice() );
            self.nes.audio_buffer.clear();
        }

        if self.audio_device.as_ref().map( |device| device.size() < 44100 / 2 ).unwrap_or( false ) && self.nes.frame % 2 == 0 {
            return;
        }
        self.renderer.present(); // This might block due to vsync.
    }
}

pub fn create() -> UserInterface {
    UserInterface::create()
}
