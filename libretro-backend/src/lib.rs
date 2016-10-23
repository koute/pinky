extern crate libc;
extern crate libretro_sys;

use std::mem;
use std::ptr;
use std::slice;
use std::ffi::{CStr, CString};
use std::cmp::max;
use std::panic;

pub use libretro_sys::{PixelFormat, Region};

fn to_cstring< T: AsRef< [u8] > + ?Sized >( input: &T ) -> CString {
    let bytes = input.as_ref();
    let mut vec = Vec::with_capacity( bytes.len() + 1 );
    vec.extend_from_slice( bytes );
    CString::new( vec ).unwrap()
}

fn empty_cstring() -> CString {
    to_cstring( "" )
}

pub struct CoreInfo {
    library_name: CString,
    library_version: CString,
    supported_romfile_extensions: CString,
    require_path_when_loading_roms: bool,
    allow_frontend_to_extract_archives: bool
}

impl CoreInfo {
    pub fn new( name: &str, version: &str ) -> CoreInfo {
        CoreInfo {
            library_name: to_cstring( name ),
            library_version: to_cstring( version ),
            supported_romfile_extensions: empty_cstring(),
            require_path_when_loading_roms: false,
            allow_frontend_to_extract_archives: true
        }
    }

    pub fn supports_roms_with_extension( mut self, mut extension: &str ) -> Self {
        if extension.starts_with( "." ) {
            extension = &extension[ 1.. ];
        }

        let mut string = empty_cstring();
        mem::swap( &mut string, &mut self.supported_romfile_extensions );

        let mut vec = string.into_bytes();
        if vec.is_empty() == false {
            vec.push( '|' as u8 );
        }

        vec.extend_from_slice( extension.as_bytes() );

        match extension {
            "gz" | "xz"  |
            "zip" | "rar" | "7z" | "tar" | "tgz" | "txz" | "bz2" |
            "tar.gz" | "tar.bz2"| "tar.xz" => {
                self.allow_frontend_to_extract_archives = false;
            },
            _ => {}
        }

        self.supported_romfile_extensions = CString::new( vec ).unwrap();
        self
    }

    pub fn requires_path_when_loading_roms( mut self ) -> Self {
        self.require_path_when_loading_roms = true;
        self
    }
}

pub struct AudioVideoInfo {
    width: u32,
    height: u32,
    max_width: u32,
    max_height: u32,
    frames_per_second: f64,
    audio_sample_rate: f64,
    aspect_ratio: Option< f32 >,
    pixel_format: PixelFormat,
    game_region: Option< Region >
}

impl AudioVideoInfo {
    pub fn new() -> AudioVideoInfo {
        AudioVideoInfo {
            width: 0,
            height: 0,
            max_width: 0,
            max_height: 0,
            frames_per_second: 0.0,
            aspect_ratio: None,
            pixel_format: PixelFormat::RGB565,
            audio_sample_rate: 0.0,
            game_region: None
        }
    }

    pub fn video( mut self, width: u32, height: u32, frames_per_second: f64, pixel_format: PixelFormat ) -> Self {
        self.width = width;
        self.height = height;
        self.max_width = max( self.max_width, width );
        self.max_height = max( self.max_height, height );
        self.frames_per_second = frames_per_second;
        self.pixel_format = pixel_format;
        self
    }

    pub fn max_video_size( mut self, max_width: u32, max_height: u32 ) -> Self {
        self.max_width = max( self.max_width, max_width );
        self.max_height = max( self.max_height, max_height );
        self
    }

    pub fn aspect_ratio( mut self, aspect_ratio: f32 ) -> Self {
        self.aspect_ratio = Some( aspect_ratio );
        self
    }

    pub fn audio( mut self, sample_rate: f64 ) -> Self {
        self.audio_sample_rate = sample_rate;
        self
    }

    pub fn region( mut self, game_region: Region ) -> Self {
        self.game_region = Some( game_region );
        self
    }

    fn infer_game_region( &self ) -> Region {
        self.game_region.unwrap_or_else( || {
            if self.frames_per_second > 59.0 {
                Region::NTSC
            } else {
                Region::PAL
            }
        })
    }
}

pub struct GameData {
    path: Option< String >,

    // The 'static lifetime here is a lie, but it's safe anyway
    // since the user doesn't get direct access to this reference,
    // and he has to give us a GameData object back in on_unload_game,
    // and since we're the only source of those he has to give us
    // the one that he got in on_load_game.
    data: Option< &'static [u8] >
}

impl GameData {
    pub fn path( &self ) -> Option< &str > {
        self.path.as_ref().map( |path| &path[..] )
    }

    pub fn data( &self ) -> Option< &[u8] > {
        self.data.map( |data| data as &[u8] )
    }

    pub fn is_empty( &self ) -> bool {
        self.path().is_none() && self.data().is_none()
    }
}

pub enum LoadGameResult {
    Success( AudioVideoInfo ),
    Failed( GameData )
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum JoypadButton {
    A,
    B,
    X,
    Y,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
    L1,
    L2,
    L3,
    R1,
    R2,
    R3
}

pub trait Backend {
    fn on_initialize( &mut self ) -> CoreInfo;
    fn on_load_game( &mut self, game_data: GameData ) -> LoadGameResult;
    fn on_unload_game( &mut self ) -> GameData;
    fn on_destroy( &mut self );
    fn on_run( &mut self, handle: &mut RuntimeHandle );
    fn on_reset( &mut self );
}

struct Retro {
    environment_callback: Option< libretro_sys::EnvironmentFn >,
    video_refresh_callback: Option< libretro_sys::VideoRefreshFn >,
    audio_sample_callback: Option< libretro_sys::AudioSampleFn >,
    audio_sample_batch_callback: Option< libretro_sys::AudioSampleBatchFn >,
    input_poll_callback: Option< libretro_sys::InputPollFn >,
    input_state_callback: Option< libretro_sys::InputStateFn >,

    backend: Option< Box< Backend > >,

    is_game_loaded: bool,
    core_info: CoreInfo,
    av_info: AudioVideoInfo,
    total_audio_samples_uploaded: usize
}

impl Retro {
    fn new() -> Retro {
        Retro {
            environment_callback: None,
            video_refresh_callback: None,
            audio_sample_callback: None,
            audio_sample_batch_callback: None,
            input_poll_callback: None,
            input_state_callback: None,

            backend: None,

            is_game_loaded: false,
            core_info: CoreInfo::new( "", "" ),
            av_info: AudioVideoInfo::new(),
            total_audio_samples_uploaded: 0
        }
    }

    fn backend_mut( &mut self ) -> &mut Backend {
        &mut **self.backend.as_mut().unwrap()
    }

    #[must_use]
    unsafe fn call_environment< T >( &mut self, command: libc::c_uint, pointer: &T ) -> Result< (), () > {
        let ok = self.environment_callback.unwrap()( command, mem::transmute( pointer ) );
        if ok {
            Ok(())
        } else {
            Err(())
        }
    }

    fn on_initialize( &mut self ) {
        self.core_info = self.backend_mut().on_initialize();
    }

    fn on_get_system_info( &mut self, info: &mut libretro_sys::SystemInfo ) {
        info.library_name = self.core_info.library_name.as_ptr();
        info.library_version = self.core_info.library_version.as_ptr();
        info.valid_extensions = self.core_info.supported_romfile_extensions.as_ptr();
        info.need_fullpath = self.core_info.require_path_when_loading_roms;
        info.block_extract = self.core_info.allow_frontend_to_extract_archives == false;
    }

    fn on_get_system_av_info( &mut self, info: &mut libretro_sys::SystemAvInfo ) {
        info.geometry.base_width = self.av_info.width as libc::c_uint;
        info.geometry.base_height = self.av_info.height as libc::c_uint;
        info.geometry.max_width = self.av_info.max_width as libc::c_uint;
        info.geometry.max_height = self.av_info.max_height as libc::c_uint;
        info.geometry.aspect_ratio = self.av_info.aspect_ratio.unwrap_or( 0.0 );
        info.timing.fps = self.av_info.frames_per_second;
        info.timing.sample_rate = self.av_info.audio_sample_rate;
    }

    fn on_load_game( &mut self, game_info: Option< &libretro_sys::GameInfo > ) -> bool {
        assert_eq!( self.is_game_loaded, false );

        let game_data = match game_info {
            Some( game_info ) => {
                let path = if game_info.path == ptr::null() {
                    None
                } else {
                    unsafe {
                        CStr::from_ptr( game_info.path ).to_str().ok().map( |path| path.to_owned() )
                    }
                };

                let data = if game_info.data == ptr::null() && game_info.size == 0 {
                    None
                } else {
                    unsafe {
                        Some( slice::from_raw_parts( game_info.data as *const u8, game_info.size ) )
                    }
                };

                GameData {
                    path: path,
                    data: data
                }
            },
            None => {
                GameData {
                    path: None,
                    data: None
                }
            }
        };

        let result = self.backend_mut().on_load_game( game_data );
        match result {
            LoadGameResult::Success( av_info ) => {
                self.av_info = av_info;
                unsafe {
                    let pixel_format = self.av_info.pixel_format;
                    self.call_environment( libretro_sys::ENVIRONMENT_SET_PIXEL_FORMAT, &pixel_format ).unwrap();
                }

                self.is_game_loaded = true;
                true
            },
            LoadGameResult::Failed( _ ) => false
        }
    }

    fn on_run( &mut self ) {
        let mut handle = RuntimeHandle {
            video_refresh_callback: self.video_refresh_callback.unwrap(),
            input_state_callback: self.input_state_callback.unwrap(),
            audio_sample_batch_callback: self.audio_sample_batch_callback.unwrap(),
            upload_video_frame_already_called: false,
            audio_samples_uploaded: 0,

            video_width: self.av_info.width,
            video_height: self.av_info.height,
            video_frame_bytes_per_pixel: match self.av_info.pixel_format {
                PixelFormat::ARGB1555 | PixelFormat::RGB565 => 2,
                PixelFormat::ARGB8888 => 4
            }
        };

        unsafe {
            self.input_poll_callback.unwrap()();
        }

        self.backend_mut().on_run( &mut handle );

        self.total_audio_samples_uploaded += handle.audio_samples_uploaded;
        let required_audio_sample_count_per_frame = (self.av_info.audio_sample_rate / self.av_info.frames_per_second) * 2.0;
        assert!(
            self.total_audio_samples_uploaded as f64 >= required_audio_sample_count_per_frame,
            format!( "You need to upload at least {} audio samples each frame!", required_audio_sample_count_per_frame )
        );

        self.total_audio_samples_uploaded -= required_audio_sample_count_per_frame as usize;
    }

    fn on_unload_game( &mut self ) {
        if self.is_game_loaded == false {
            return;
        }

        let _ = self.backend_mut().on_unload_game();
    }

    fn on_reset( &mut self ) {
        self.backend_mut().on_reset();
    }
}

pub struct RuntimeHandle {
    video_refresh_callback: libretro_sys::VideoRefreshFn,
    input_state_callback: libretro_sys::InputStateFn,
    audio_sample_batch_callback: libretro_sys::AudioSampleBatchFn,
    upload_video_frame_already_called: bool,
    audio_samples_uploaded: usize,

    video_width: u32,
    video_height: u32,
    video_frame_bytes_per_pixel: u32
}

impl RuntimeHandle {
    pub fn upload_video_frame( &mut self, data: &[u8] ) {
        assert!( self.upload_video_frame_already_called == false, "You can only call upload_video_frame() once per frame!" );

        self.upload_video_frame_already_called = true;
        let bytes = data.as_ptr() as *const libc::c_void;
        let width = self.video_width as libc::c_uint;
        let height = self.video_height as libc::c_uint;
        let bytes_per_pixel = (self.video_width * self.video_frame_bytes_per_pixel) as usize;
        unsafe {
            (self.video_refresh_callback)( bytes, width, height, bytes_per_pixel );
        }
    }

    pub fn upload_audio_frame( &mut self, data: &[i16] ) {
        assert!( data.len() % 2 == 0, "Audio data must be in stereo!" );

        self.audio_samples_uploaded += data.len();
        unsafe {
            (self.audio_sample_batch_callback)( data.as_ptr(), data.len() / 2 );
        }
    }

    pub fn is_joypad_button_pressed( &mut self, port: u32, button: JoypadButton ) -> bool {
        let device_id = match button {
            JoypadButton::A => libretro_sys::DEVICE_ID_JOYPAD_A,
            JoypadButton::B => libretro_sys::DEVICE_ID_JOYPAD_B,
            JoypadButton::X => libretro_sys::DEVICE_ID_JOYPAD_X,
            JoypadButton::Y => libretro_sys::DEVICE_ID_JOYPAD_Y,
            JoypadButton::Start => libretro_sys::DEVICE_ID_JOYPAD_START,
            JoypadButton::Select => libretro_sys::DEVICE_ID_JOYPAD_SELECT,
            JoypadButton::Left => libretro_sys::DEVICE_ID_JOYPAD_LEFT,
            JoypadButton::Right => libretro_sys::DEVICE_ID_JOYPAD_RIGHT,
            JoypadButton::Up => libretro_sys::DEVICE_ID_JOYPAD_UP,
            JoypadButton::Down => libretro_sys::DEVICE_ID_JOYPAD_DOWN,
            JoypadButton::L1 => libretro_sys::DEVICE_ID_JOYPAD_L,
            JoypadButton::L2 => libretro_sys::DEVICE_ID_JOYPAD_L2,
            JoypadButton::L3 => libretro_sys::DEVICE_ID_JOYPAD_L3,
            JoypadButton::R1 => libretro_sys::DEVICE_ID_JOYPAD_R,
            JoypadButton::R2 => libretro_sys::DEVICE_ID_JOYPAD_R2,
            JoypadButton::R3 => libretro_sys::DEVICE_ID_JOYPAD_R3
        };

        unsafe {
            let value = (self.input_state_callback)( port, libretro_sys::DEVICE_JOYPAD, 0, device_id );
            return value == 1;
        }
    }
}

static mut INSTANCE: *mut Retro = 0 as *mut Retro;

fn ensure_instance_is_initialized() {
    unsafe {
        if INSTANCE != ptr::null_mut() {
            return;
        }
        INSTANCE = Box::into_raw( Box::new( Retro::new() ));
    }
}

fn instance() -> &'static mut Retro {
    unsafe {
        ensure_instance_is_initialized();
        &mut *INSTANCE
    }
}

#[doc(hidden)]
pub unsafe fn initialize( backend: Box< Backend > ) {
    instance().backend = Some( backend );
    instance().on_initialize();
}

#[macro_export]
macro_rules! libretro_backend {
    ($backend: expr) => (
        #[no_mangle]
        pub extern "C" fn retro_init() {
            let backend = $backend;
            unsafe {
                $crate::initialize( backend );
            }
        }
    )
}

macro_rules! set_callback {
    ($output: expr, $input: expr) => (
        unsafe {
            if $input == mem::transmute( 0 as usize ) {
                $output = None;
            } else {
                $output = Some( $input );
            }
        }
    )
}

macro_rules! abort_on_panic {
    ($code:expr) => ({
        let result = panic::catch_unwind(|| {
            $code
        });

        match result {
            Err( _ ) => {
                use std::process;
                process::exit( 1 );
            },
            Ok( value ) => value
        }
    })
}

#[no_mangle]
pub extern "C" fn retro_set_environment( callback: libretro_sys::EnvironmentFn ) {
    set_callback!( instance().environment_callback, callback );
}

#[no_mangle]
pub extern "C" fn retro_set_video_refresh( callback: libretro_sys::VideoRefreshFn ) {
    set_callback!( instance().video_refresh_callback, callback );
}

#[no_mangle]
pub extern "C" fn retro_set_audio_sample( callback: libretro_sys::AudioSampleFn ) {
    set_callback!( instance().audio_sample_callback, callback );
}

#[no_mangle]
pub extern "C" fn retro_set_audio_sample_batch( callback: libretro_sys::AudioSampleBatchFn ) {
    set_callback!( instance().audio_sample_batch_callback, callback );
}

#[no_mangle]
pub extern "C" fn retro_set_input_poll( callback: libretro_sys::InputPollFn ) {
    set_callback!( instance().input_poll_callback, callback );
}

#[no_mangle]
pub extern "C" fn retro_set_input_state( callback: libretro_sys::InputStateFn ) {
    set_callback!( instance().input_state_callback, callback );
}

#[no_mangle]
pub extern "C" fn retro_deinit() {
    let _ = panic::catch_unwind(|| {
        instance().backend_mut().on_destroy();
    });

    unsafe {
        let instance = Box::from_raw( INSTANCE );
        INSTANCE = ptr::null_mut();
        mem::drop( instance );
    }
}

#[no_mangle]
pub extern "C" fn retro_api_version() -> libc::c_uint {
    return libretro_sys::API_VERSION;
}

#[no_mangle]
pub extern "C" fn retro_get_system_info( info: *mut libretro_sys::SystemInfo ) {
    assert!( info != ptr::null_mut() );

    abort_on_panic!({
        unsafe {
            instance().on_get_system_info( &mut *info );
        }
    });
}

#[no_mangle]
pub extern "C" fn retro_get_system_av_info( info: *mut libretro_sys::SystemAvInfo ) {
    assert!( info != ptr::null_mut() );

    abort_on_panic!({
        unsafe {
            instance().on_get_system_av_info( &mut *info );
        }
    });
}

#[no_mangle]
pub extern "C" fn retro_set_controller_port_device( _: libc::c_uint, _: libc::c_uint ) {
}

#[no_mangle]
pub extern "C" fn retro_reset() {
    abort_on_panic!({
        instance().on_reset();
    });
}

#[no_mangle]
pub extern "C" fn retro_run() {
    abort_on_panic!({
        instance().on_run();
    });
}

#[no_mangle]
pub extern "C" fn retro_serialize_size() -> libc::size_t {
    0
}

#[no_mangle]
pub extern "C" fn retro_serialize( _: *mut libc::c_void, _: libc::size_t ) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn retro_unserialize( _: *const libc::c_void, _: libc::size_t ) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn retro_cheat_reset() {
}

#[no_mangle]
pub extern "C" fn retro_cheat_set( _: libc::c_uint, _: bool, _: *const libc::c_char ) {
}

#[no_mangle]
pub extern "C" fn retro_load_game( game: *const libretro_sys::GameInfo ) -> bool {
    if game == ptr::null() {
        return false;
    }

    abort_on_panic!({
        unsafe {
            if game == ptr::null() {
                instance().on_load_game( None )
            } else {
                instance().on_load_game( Some( &*game ) )
            }
        }
    })
}

#[no_mangle]
pub extern "C" fn retro_load_game_special( _: libc::c_uint, _: *const libretro_sys::GameInfo, _: libc::size_t ) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn retro_unload_game() {
    abort_on_panic!({
        instance().on_unload_game()
    });
}

#[no_mangle]
pub extern "C" fn retro_get_region() -> libc::c_uint {
    abort_on_panic!({
        instance().av_info.infer_game_region().to_uint()
    })
}

#[no_mangle]
pub extern "C" fn retro_get_memory_data( _: libc::c_uint ) -> *mut libc::c_void {
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn retro_get_memory_size( _: libc::c_uint ) -> libc::size_t {
    0
}
