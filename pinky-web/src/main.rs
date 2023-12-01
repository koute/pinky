#![recursion_limit="2048"]

#[macro_use]
extern crate stdweb;
extern crate nes;

#[macro_use]
extern crate serde_derive;
extern crate serde;

use std::cell::RefCell;
use std::rc::Rc;
use std::ops::DerefMut;
use std::cmp::max;
use std::error::Error;

use stdweb::web::{
    self,
    FileReader,
    FileReaderResult,
    FileList,
    Element,
    ArrayBuffer
};

use stdweb::web::event::{
    ClickEvent,
    ChangeEvent,
    ProgressLoadEvent,
    KeyDownEvent,
    KeyUpEvent,
    IKeyboardEvent,
};

use stdweb::traits::*;

use stdweb::web::html_element::InputElement;
use stdweb::unstable::TryInto;
use stdweb::{Value, UnsafeTypedArray, Once};

macro_rules! enclose {
    ( [$( $x:ident ),*] $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

fn palette_to_abgr( palette: &nes::Palette ) -> [u32; 512] {
    let mut output = [0; 512];
    for (index, out) in output.iter_mut().enumerate() {
        let (r, g, b) = palette.get_rgb( index as u16 );
        *out = ((b as u32) << 16) |
               ((g as u32) <<  8) |
               ((r as u32)      ) |
               0xFF000000;
    }

    output
}

struct PinkyWeb {
    state: nes::State,
    palette: [u32; 512],
    framebuffer: [u32; 256 * 240],
    audio_buffer: Vec< f32 >,
    audio_chunk_counter: u32,
    audio_underrun: Option< usize >,
    paused: bool,
    busy: bool,
    js_ctx: Value
}

impl nes::Context for PinkyWeb {
    #[inline]
    fn state_mut( &mut self ) -> &mut nes::State {
        &mut self.state
    }

    #[inline]
    fn state( &self ) -> &nes::State {
        &self.state
    }

    // Ugh, trying to get gapless low-latency PCM playback
    // out of the Web Audio APIs is like driving a rusty
    // nail through my hand.
    //
    // This blog post probably sums it up pretty well:
    //   http://blog.mecheye.net/2017/09/i-dont-know-who-the-web-audio-api-is-designed-for/
    //
    // So, this is not perfect, but it's as good as I can make it.
    #[inline]
    fn on_audio_sample( &mut self, sample: f32 ) {
        self.audio_buffer.push( sample );
        if self.audio_buffer.len() == 2048 {
            self.audio_chunk_counter += 1;
            let audio_buffered: f64 = js! {
                var h = @{&self.js_ctx};
                var samples = @{unsafe { UnsafeTypedArray::new( &self.audio_buffer ) }};
                var sample_rate = 44100;
                var sample_count = samples.length;
                var latency = 0.032;

                var audio_buffer;
                if( h.empty_audio_buffers.length === 0 ) {
                    audio_buffer = h.audio.createBuffer( 1, sample_count, sample_rate );
                } else {
                    audio_buffer = h.empty_audio_buffers.pop();
                }

                audio_buffer.getChannelData( 0 ).set( samples );

                var node = h.audio.createBufferSource();
                node.connect( h.audio.destination );
                node.buffer = audio_buffer;
                node.onended = function() {
                    h.empty_audio_buffers.push( audio_buffer );
                };

                var buffered = h.play_timestamp - (h.audio.currentTime + latency);
                var play_timestamp = Math.max( h.audio.currentTime + latency, h.play_timestamp );
                node.start( play_timestamp );
                h.play_timestamp = play_timestamp + sample_count / sample_rate;

                return buffered;
            }.try_into().unwrap();

            // Since we're using `request_animation_frame` for synchronization
            // we **will** go out of sync with the audio sooner or later,
            // which will result in sudden, and very unpleasant, audio pops.
            //
            // So here we check how much audio exactly we have queued up,
            // and if we don't have enough then we'll try to compensate
            // by running the emulator for a few more cycles at the end
            // of the current frame.
            if audio_buffered < 0.000 {
                self.audio_underrun = Some( max( self.audio_underrun.unwrap_or( 0 ), 3 ) );
            } else if audio_buffered < 0.010 {
                self.audio_underrun = Some( max( self.audio_underrun.unwrap_or( 0 ), 2 ) );
            } else if audio_buffered < 0.020 {
                self.audio_underrun = Some( max( self.audio_underrun.unwrap_or( 0 ), 1 ) );
            }

            self.audio_buffer.clear();
        }
    }
}

// This creates a really basic WebGL context for blitting a single texture.
// On some web browsers this is faster than using a 2d canvas.
fn setup_webgl( canvas: &Element ) -> Value {
    const FRAGMENT_SHADER: &'static str = r#"
        precision mediump float;
        varying vec2 v_texcoord;
        uniform sampler2D u_sampler;
        void main() {
            gl_FragColor = vec4( texture2D( u_sampler, vec2( v_texcoord.s, v_texcoord.t ) ).rgb, 1.0 );
        }
    "#;

    const VERTEX_SHADER: &'static str = r#"
        attribute vec2 a_position;
        attribute vec2 a_texcoord;
        uniform mat4 u_matrix;
        varying vec2 v_texcoord;
        void main() {
            gl_Position = u_matrix * vec4( a_position, 0.0, 1.0 );
            v_texcoord = a_texcoord;
        }
    "#;

    fn ortho( left: f64, right: f64, bottom: f64, top: f64 ) -> Vec< f64 > {
        let mut m = vec![ 1.0, 0.0, 0.0, 0.0,
                          0.0, 1.0, 0.0, 0.0,
                          0.0, 0.0, 1.0, 0.0,
                          0.0, 0.0, 0.0, 1.0 ];

        m[ 0 * 4 + 0 ] = 2.0 / (right - left);
        m[ 1 * 4 + 1 ] = 2.0 / (top - bottom);
        m[ 3 * 4 + 0 ] = (right + left) / (right - left) * -1.0;
        m[ 3 * 4 + 1 ] = (top + bottom) / (top - bottom) * -1.0;

        return m;
    }

    js!(
        var gl;
        var webgl_names = ["webgl", "experimental-webgl", "webkit-3d", "moz-webgl"];
        for( var i = 0; i < webgl_names.length; ++i ) {
            var name = webgl_names[ i ];
            try {
                gl = @{canvas}.getContext( name );
            } catch( err ) {}

            if( gl ) {
                console.log( "WebGL support using context:", name );
                break;
            }
        }

        if( gl === null ) {
            console.error( "WebGL rendering context not found." );
            return null;
        }

        var vertex_shader = gl.createShader( gl.VERTEX_SHADER );
        var fragment_shader = gl.createShader( gl.FRAGMENT_SHADER );
        gl.shaderSource( vertex_shader, @{VERTEX_SHADER} );
        gl.shaderSource( fragment_shader, @{FRAGMENT_SHADER} );
        gl.compileShader( vertex_shader );
        gl.compileShader( fragment_shader );

        if( !gl.getShaderParameter( vertex_shader, gl.COMPILE_STATUS ) ) {
            console.error( "WebGL vertex shader compilation failed:", gl.getShaderInfoLog( vertex_shader ) );
            return null;
        }

        if( !gl.getShaderParameter( fragment_shader, gl.COMPILE_STATUS ) ) {
            console.error( "WebGL fragment shader compilation failed:", gl.getShaderInfoLog( fragment_shader ) );
            return null;
        }

        var program = gl.createProgram();
        gl.attachShader( program, vertex_shader );
        gl.attachShader( program, fragment_shader );
        gl.linkProgram( program );
        if( !gl.getProgramParameter( program, gl.LINK_STATUS ) ) {
            console.error( "WebGL program linking failed!" );
            return null;
        }

        gl.useProgram( program );

        var vertex_attr = gl.getAttribLocation( program, "a_position" );
        var texcoord_attr = gl.getAttribLocation( program, "a_texcoord" );

        gl.enableVertexAttribArray( vertex_attr );
        gl.enableVertexAttribArray( texcoord_attr );

        var sampler_uniform = gl.getUniformLocation( program, "u_sampler" );
        gl.uniform1i( sampler_uniform, 0 );

        var matrix = @{ortho( 0.0, 256.0, 240.0, 0.0 )};
        var matrix_uniform = gl.getUniformLocation( program, "u_matrix" );
        gl.uniformMatrix4fv( matrix_uniform, false, matrix );

        var texture = gl.createTexture();
        gl.bindTexture( gl.TEXTURE_2D, texture );
        gl.texImage2D( gl.TEXTURE_2D, 0, gl.RGBA, 256, 256, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array( 256 * 256 * 4 ) );
        gl.texParameteri( gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST );
        gl.texParameteri( gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST );

        var vertex_buffer = gl.createBuffer();
        gl.bindBuffer( gl.ARRAY_BUFFER, vertex_buffer );
        var vertices = [
            0.0, 0.0,
            0.0, 240.0,
            256.0, 0.0,
            256.0, 240.0
        ];
        gl.bufferData( gl.ARRAY_BUFFER, new Float32Array( vertices ), gl.STATIC_DRAW );
        gl.vertexAttribPointer( vertex_attr, 2, gl.FLOAT, false, 0, 0 );

        var texcoord_buffer = gl.createBuffer();
        gl.bindBuffer( gl.ARRAY_BUFFER, texcoord_buffer );
        var texcoords = [
            0.0, 0.0,
            0.0, 240.0 / 256.0,
            1.0, 0.0,
            1.0, 240.0 / 256.0
        ];
        gl.bufferData( gl.ARRAY_BUFFER, new Float32Array( texcoords ), gl.STATIC_DRAW );
        gl.vertexAttribPointer( texcoord_attr, 2, gl.FLOAT, false, 0, 0 );

        var index_buffer = gl.createBuffer();
        gl.bindBuffer( gl.ELEMENT_ARRAY_BUFFER, index_buffer );
        var indices = [
            0, 1, 2,
            2, 3, 1
        ];
        gl.bufferData( gl.ELEMENT_ARRAY_BUFFER, new Uint16Array( indices ), gl.STATIC_DRAW );

        gl.clearColor( 0.0, 0.0, 0.0, 1.0 );
        gl.enable( gl.DEPTH_TEST );
        gl.viewport( 0, 0, 256, 240 );

        return gl;
    )
}

impl PinkyWeb {
    fn new( canvas: &Element ) -> Self {
        let gl = setup_webgl( &canvas );

        let js_ctx = js!(
            var h = {};
            var canvas = @{canvas};

            h.gl = @{gl};
            h.audio = new AudioContext();
            h.empty_audio_buffers = [];
            h.play_timestamp = 0;

            if( !h.gl ) {
                console.log( "No WebGL; using Canvas API" );

                // If the WebGL **is** supported but something else
                // went wrong the web browser won't let us create
                // a normal canvas context on a WebGL-ified canvas,
                // so we recreate a new canvas here to work around that.
                var new_canvas = canvas.cloneNode( true );
                canvas.parentNode.replaceChild( new_canvas, canvas );
                canvas = new_canvas;

                h.ctx = canvas.getContext( "2d" );
                h.img = h.ctx.createImageData( 256, 240 );
                h.buffer = new Uint32Array( h.img.data.buffer );
            }

            return h;
        );

        PinkyWeb {
            state: nes::State::new(),
            palette: palette_to_abgr( &nes::Palette::default() ),
            framebuffer: [0; 256 * 240],
            audio_buffer: Vec::with_capacity( 44100 ),
            audio_chunk_counter: 0,
            audio_underrun: None,
            paused: true,
            busy: false,
            js_ctx
        }
    }

    fn pause( &mut self ) {
        self.paused = true;
    }

    fn unpause( &mut self ) {
        self.paused = false;
        self.busy = false;
    }

    // This will run the emulator either until we've finished
    // a frame, or until we've generated one audio chunk,
    // in which case we'll temporairly give back the control
    // of the main thread back to the web browser so that
    // it can handle other events and process audio.
    fn run_a_bit( &mut self ) -> Result< bool, Box< Error > > {
        if self.paused {
            return Ok( true );
        }

        let audio_chunk_counter = self.audio_chunk_counter;
        loop {
            let result = nes::Interface::execute_cycle( self );
            match result {
                Ok( processed_whole_frame ) => {
                    if processed_whole_frame {
                        return Ok( true );
                    } else if self.audio_chunk_counter != audio_chunk_counter {
                        return Ok( false );
                    }
                },
                Err( error ) => {
                    js!( console.error( "Execution error:", @{format!( "{}", error )} ); );
                    self.pause();

                    return Err( error );
                }
            }
        }
    }

    fn draw( &mut self ) {
        let framebuffer = self.state.framebuffer();
        if !self.paused {
            for (pixel_in, pixel_out) in framebuffer.iter().zip( self.framebuffer.iter_mut() ) {
                *pixel_out = self.palette[ pixel_in.full_color_index() as usize ];
            }
        }

        js! {
            var h = @{&self.js_ctx};
            var framebuffer = @{unsafe { UnsafeTypedArray::new( &self.framebuffer ) }};
            if( h.gl ) {
                var data = new Uint8Array( framebuffer.buffer, framebuffer.byteOffset, framebuffer.byteLength );
                h.gl.texSubImage2D( h.gl.TEXTURE_2D, 0, 0, 0, 256, 240, h.gl.RGBA, h.gl.UNSIGNED_BYTE, data );
                h.gl.drawElements( h.gl.TRIANGLES, 6, h.gl.UNSIGNED_SHORT, 0 );
            } else {
                h.buffer.set( framebuffer );
                h.ctx.putImageData( h.img, 0, 0 );
            }
        }
    }

    fn on_key( &mut self, keycode: &str, is_pressed: bool ) -> bool {
        let button = match keycode {
            "Enter" => nes::Button::Start,
            "ShiftRight" => nes::Button::Select,
            "ArrowUp" => nes::Button::Up,
            "ArrowLeft" => nes::Button::Left,
            "ArrowRight" => nes::Button::Right,
            "ArrowDown" => nes::Button::Down,

            // On Edge the arrows have different names
            // for some reason.
            "Up" => nes::Button::Up,
            "Left" => nes::Button::Left,
            "Right" => nes::Button::Right,
            "Down" => nes::Button::Down,

            "KeyZ" => nes::Button::A,
            "KeyX" => nes::Button::B,

            // For those using the Dvorak layout **and** Microsoft Edge.
            //
            // On `keydown` we get ";" as we should, but on `keyup`
            // we get "Unidentified". Seriously Microsoft, how buggy can
            // your browser be?
            "Unidentified" if is_pressed == false => nes::Button::A,

            _ => return false
        };

        nes::Interface::set_button_state( self, nes::ControllerPort::First, button, is_pressed );
        return true;
    }
}

fn emulate_for_a_single_frame( pinky: Rc< RefCell< PinkyWeb > > ) {
    pinky.borrow_mut().busy = true;

    web::set_timeout( enclose!( [pinky] move || {
        let finished_frame = match pinky.borrow_mut().run_a_bit() {
            Ok( result ) => result,
            Err( error ) => {
                handle_error( error );
                return;
            }
        };

        if !finished_frame {
            web::set_timeout( move || { emulate_for_a_single_frame( pinky ); }, 0 );
        } else {
            let mut pinky = pinky.borrow_mut();
            if let Some( count ) = pinky.audio_underrun.take() {
                for _ in 0..count {
                    if let Err( error ) = pinky.run_a_bit() {
                        handle_error( error );
                        return;
                    }
                }
            }

            pinky.busy = false;
        }
    }), 0 );
}

fn main_loop( pinky: Rc< RefCell< PinkyWeb > > ) {
    // If we're running too slowly there is no point
    // in queueing up even more work.
    if !pinky.borrow_mut().busy {
        emulate_for_a_single_frame( pinky.clone() );
    }

    pinky.borrow_mut().draw();
    web::window().request_animation_frame( move |_| {
        main_loop( pinky );
    });
}

#[derive(Deserialize)]
struct RomEntry {
    name: String,
    file: String
}

js_deserializable!( RomEntry );

fn show( id: &str ) {
    web::document().get_element_by_id( id ).unwrap().class_list().remove( "hidden" ).unwrap();
}

fn hide( id: &str ) {
    web::document().get_element_by_id( id ).unwrap().class_list().add( "hidden" ).unwrap();
}

fn fetch_builtin_rom_list< F: FnOnce( Vec< RomEntry > ) + 'static >( callback: F ) {
    let on_rom_list_loaded = Once( move |mut roms: Vec< RomEntry >| {
        roms.sort_by( |a, b| a.name.cmp( &b.name ) );
        callback( roms );
    });

    js! {
        var req = new XMLHttpRequest();
        req.addEventListener( "load" , function() {
            var cb = @{on_rom_list_loaded};
            cb( JSON.parse( req.responseText ) );
            cb.drop();
        });
        req.open( "GET", "roms/index.json" );
        req.send();
    }
}

fn support_builtin_roms( roms: Vec< RomEntry >, pinky: Rc< RefCell< PinkyWeb > > ) {
    let entries = web::document().get_element_by_id( "rom-list" ).unwrap();
    for rom in roms {
        let entry = web::document().create_element( "button" ).unwrap();
        let name = rom.name;
        let file = rom.file;

        entry.set_text_content( &name );
        entries.append_child( &entry );
        entry.add_event_listener( enclose!( [pinky] move |_: ClickEvent| {
            hide( "change-rom-menu" );
            hide( "side-text" );
            show( "loading" );

            let builtin_rom_loaded = Once( enclose!( [pinky] move |array_buffer: ArrayBuffer| {
                let rom_data: Vec< u8 > = array_buffer.into();
                load_rom( &pinky, &rom_data );
            }));
            js! {
                var req = new XMLHttpRequest();
                req.addEventListener( "load" , function() {
                    @{builtin_rom_loaded}( req.response );
                });
                req.open( "GET", "roms/" + @{&file} );
                req.responseType = "arraybuffer";
                req.send();
            }
        }));
    }
}

fn support_custom_roms( pinky: Rc< RefCell< PinkyWeb > > ) {
    let browse_for_roms_button = web::document().get_element_by_id( "browse-for-roms" ).unwrap();
    browse_for_roms_button.add_event_listener( move |event: ChangeEvent| {
        let input: InputElement = event.target().unwrap().try_into().unwrap();
        let files: FileList = js!( return @{input}.files; ).try_into().unwrap();
        let file = match files.iter().next() {
            Some( file ) => file,
            None => return
        };

        hide( "change-rom-menu" );
        hide( "side-text" );
        show( "loading" );

        let reader = FileReader::new();
        reader.add_event_listener( enclose!( [pinky, reader] move |_: ProgressLoadEvent| {
            let rom_data: Vec< u8 > = match reader.result().unwrap() {
                FileReaderResult::ArrayBuffer( buffer ) => buffer,
                _ => unreachable!()
            }.into();

            load_rom( &pinky, &rom_data );
        }));

        reader.read_as_array_buffer( &file ).unwrap();
    });
}

fn support_rom_changing( pinky: Rc< RefCell< PinkyWeb > > ) {
    let change_rom_button = web::document().get_element_by_id( "change-rom-button" ).unwrap();
    change_rom_button.add_event_listener( enclose!( [pinky] move |_: ClickEvent| {
        pinky.borrow_mut().pause();
        hide( "viewport" );
        hide( "change-rom-button" );
        show( "change-rom-menu" );
        show( "rom-menu-close" );
    }));

    let rom_menu_close_button = web::document().get_element_by_id( "rom-menu-close" ).unwrap();
    rom_menu_close_button.add_event_listener( move |_: ClickEvent| {
        pinky.borrow_mut().unpause();
        show( "viewport" );
        show( "change-rom-button" );
        hide( "change-rom-menu" );
        hide( "rom-menu-close" );
    });
}

fn support_input( pinky: Rc< RefCell< PinkyWeb > > ) {
    web::window().add_event_listener( enclose!( [pinky] move |event: KeyDownEvent| {
        let handled = pinky.borrow_mut().on_key( &event.code(), true );
        if handled {
            event.prevent_default();
        }
    }));

    web::window().add_event_listener( enclose!( [pinky] move |event: KeyUpEvent| {
        let handled = pinky.borrow_mut().on_key( &event.code(), false );
        if handled {
            event.prevent_default();
        }
    }));
}

fn load_rom( pinky: &Rc< RefCell< PinkyWeb > >, rom_data: &[u8] ) {
    hide( "loading" );
    hide( "error" );

    let mut pinky = pinky.borrow_mut();
    let pinky = pinky.deref_mut();
    if let Err( err ) = nes::Interface::load_rom( pinky, rom_data ) {
        handle_error( err );
        return;
    }
    pinky.unpause();

    show( "viewport" );
    show( "change-rom-button" );
}

fn handle_error< E: Into< Box< Error > > >( error: E ) {
    let error_message = format!( "{}", error.into() );
    web::document().get_element_by_id( "error-description" ).unwrap().set_text_content( &error_message );

    hide( "viewport" );
    hide( "change-rom-button" );
    hide( "rom-menu-close" );
    show( "change-rom-menu" );
    show( "error" );
}

fn main() {
    stdweb::initialize();

    let canvas = web::document().get_element_by_id( "viewport" ).unwrap();
    let pinky = Rc::new( RefCell::new( PinkyWeb::new( &canvas ) ) );

    support_custom_roms( pinky.clone() );
    support_rom_changing( pinky.clone() );

    fetch_builtin_rom_list( enclose!( [pinky] |roms| {
        support_builtin_roms( roms, pinky );

        hide( "loading" );
        show( "change-rom-menu" );
    }));

    support_input( pinky.clone() );

    web::window().request_animation_frame( move |_| {
        main_loop( pinky );
    });

    stdweb::event_loop();
}
