// Copyright (C) 2016 Jan Bujak
// Copyright (C) 2010-2016 The RetroArch team
//
// ---------------------------------------------------------------------------------------
// The following license statement only applies to this libretro API header (libretro.h).
// ---------------------------------------------------------------------------------------
//
// Permission is hereby granted, free of charge,
// to any person obtaining a copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software,
// and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
// WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

extern crate libc;

macro_rules! define_enum {
    (
        $( #[$enum_attr:meta] )*
        pub enum $typename:ident {
            $( $( $( #[$variant_attr:meta] )* $variant:ident ),+ = $value:expr ),+,
        }
    ) => {
        $( #[$enum_attr] )*
        pub enum $typename {
            $( $( $( #[$variant_attr] )* $variant ),+ = $value ),+,
        }

        impl $typename {
            pub fn from_uint( value: libc::c_uint ) -> Option< Self > {
                $( $(
                    if value == $typename::$variant as libc::c_uint {
                        return Some( $typename::$variant )
                    }
                )+ )+

                None
            }

            pub fn to_uint( self ) -> libc::c_uint {
                use std::mem::transmute;
                unsafe {
                    // This will generate an error at compile time if the size
                    // of the enum is not equal to the size of libc::c_uint.
                    transmute( self )
                }
            }
        }
    };
}

// Used for checking API/ABI mismatches that can break libretro
// implementations.
// It is not incremented for compatible changes to the API.
pub const API_VERSION: libc::c_uint = 1;

// Libretro's fundamental device abstractions.
//
// Libretro's input system consists of some standardized device types,
// such as a joypad (with/without analog), mouse, keyboard, lightgun
// and a pointer.
//
// The functionality of these devices are fixed, and individual cores
// map their own concept of a controller to libretro's abstractions.
// This makes it possible for frontends to map the abstract types to a
// real input device, and not having to worry about binding input
// correctly to arbitrary controller layouts.

pub const DEVICE_TYPE_SHIFT: libc::c_uint = 8;
pub const DEVICE_MASK: libc::c_uint = ((1 << DEVICE_TYPE_SHIFT) - 1);
// TODO: Insert DEVICE_SUBCLASS here

// Input disabled.
pub const DEVICE_NONE: libc::c_uint = 0;

// The JOYPAD is called RetroPad. It is essentially a Super Nintendo
// controller, but with additional L2/R2/L3/R3 buttons, similar to a
// PS1 DualShock.
pub const DEVICE_JOYPAD: libc::c_uint = 1;

// The mouse is a simple mouse, similar to Super Nintendo's mouse.
// X and Y coordinates are reported relatively to last poll (poll callback).
// It is up to the libretro implementation to keep track of where the mouse
// pointer is supposed to be on the screen.
// The frontend must make sure not to interfere with its own hardware
// mouse pointer.
pub const DEVICE_MOUSE: libc::c_uint = 2;

// KEYBOARD device lets one poll for raw key pressed.
// It is poll based, so input callback will return with the current
// pressed state.
// For event/text based keyboard input, see
// ENVIRONMENT_SET_KEYBOARD_CALLBACK.
pub const DEVICE_KEYBOARD: libc::c_uint = 3;

// Lightgun X/Y coordinates are reported relatively to last poll,
// similar to mouse.
pub const DEVICE_LIGHTGUN: libc::c_uint = 4;

// The ANALOG device is an extension to JOYPAD (RetroPad).
// Similar to DualShock it adds two analog sticks.
// This is treated as a separate device type as it returns values in the
// full analog range of [-0x8000, 0x7fff]. Positive X axis is right.
// Positive Y axis is down.
// Only use ANALOG type when polling for analog values of the axes.
pub const DEVICE_ANALOG: libc::c_uint = 5;

// Abstracts the concept of a pointing mechanism, e.g. touch.
// This allows libretro to query in absolute coordinates where on the
// screen a mouse (or something similar) is being placed.
// For a touch centric device, coordinates reported are the coordinates
// of the press.
//
// Coordinates in X and Y are reported as:
// [-0x7fff, 0x7fff]: -0x7fff corresponds to the far left/top of the screen,
// and 0x7fff corresponds to the far right/bottom of the screen.
// The "screen" is here defined as area that is passed to the frontend and
// later displayed on the monitor.
//
// The frontend is free to scale/resize this screen as it sees fit, however,
// (X, Y) = (-0x7fff, -0x7fff) will correspond to the top-left pixel of the
// game image, etc.
//
// To check if the pointer coordinates are valid (e.g. a touch display
// actually being touched), PRESSED returns 1 or 0.
//
// If using a mouse on a desktop, PRESSED will usually correspond to the
// left mouse button, but this is a frontend decision.
// PRESSED will only return 1 if the pointer is inside the game screen.
//
// For multi-touch, the index variable can be used to successively query
// more presses.
// If index = 0 returns true for _PRESSED, coordinates can be extracted
// with _X, _Y for index = 0. One can then query _PRESSED, _X, _Y with
// index = 1, and so on.
// Eventually _PRESSED will return false for an index. No further presses
// are registered at this point.
pub const DEVICE_POINTER: libc::c_uint = 6;

// Buttons for the RetroPad (JOYPAD).
// The placement of these is equivalent to placements on the
// Super Nintendo controller.
//
// L2/R2/L3/R3 buttons correspond to the PS1 DualShock.
pub const DEVICE_ID_JOYPAD_B: libc::c_uint = 0;
pub const DEVICE_ID_JOYPAD_Y: libc::c_uint = 1;
pub const DEVICE_ID_JOYPAD_SELECT: libc::c_uint = 2;
pub const DEVICE_ID_JOYPAD_START: libc::c_uint = 3;
pub const DEVICE_ID_JOYPAD_UP: libc::c_uint = 4;
pub const DEVICE_ID_JOYPAD_DOWN: libc::c_uint = 5;
pub const DEVICE_ID_JOYPAD_LEFT: libc::c_uint = 6;
pub const DEVICE_ID_JOYPAD_RIGHT: libc::c_uint = 7;
pub const DEVICE_ID_JOYPAD_A: libc::c_uint = 8;
pub const DEVICE_ID_JOYPAD_X: libc::c_uint = 9;
pub const DEVICE_ID_JOYPAD_L: libc::c_uint = 10;
pub const DEVICE_ID_JOYPAD_R: libc::c_uint = 11;
pub const DEVICE_ID_JOYPAD_L2: libc::c_uint = 12;
pub const DEVICE_ID_JOYPAD_R2: libc::c_uint = 13;
pub const DEVICE_ID_JOYPAD_L3: libc::c_uint = 14;
pub const DEVICE_ID_JOYPAD_R3: libc::c_uint = 15;

// Index / ID values for ANALOG device.
pub const DEVICE_INDEX_ANALOG_LEFT: libc::c_uint = 0;
pub const DEVICE_INDEX_ANALOG_RIGHT: libc::c_uint = 1;
pub const DEVICE_ID_ANALOG_X: libc::c_uint = 0;
pub const DEVICE_ID_ANALOG_Y: libc::c_uint = 1;

// ID values for MOUSE.
pub const DEVICE_ID_MOUSE_X: libc::c_uint = 0;
pub const DEVICE_ID_MOUSE_Y: libc::c_uint = 1;
pub const DEVICE_ID_MOUSE_LEFT: libc::c_uint = 2;
pub const DEVICE_ID_MOUSE_RIGHT: libc::c_uint = 3;
pub const DEVICE_ID_MOUSE_WHEELUP: libc::c_uint = 4;
pub const DEVICE_ID_MOUSE_WHEELDOWN: libc::c_uint = 5;
pub const DEVICE_ID_MOUSE_MIDDLE: libc::c_uint = 6;
pub const DEVICE_ID_MOUSE_HORIZ_WHEELUP: libc::c_uint = 7;
pub const DEVICE_ID_MOUSE_HORIZ_WHEELDOWN: libc::c_uint = 8;

// ID values for LIGHTGUN types.
pub const DEVICE_ID_LIGHTGUN_X: libc::c_uint = 0;
pub const DEVICE_ID_LIGHTGUN_Y: libc::c_uint = 1;
pub const DEVICE_ID_LIGHTGUN_TRIGGER: libc::c_uint = 2;
pub const DEVICE_ID_LIGHTGUN_CURSOR: libc::c_uint = 3;
pub const DEVICE_ID_LIGHTGUN_TURBO: libc::c_uint = 4;
pub const DEVICE_ID_LIGHTGUN_PAUSE: libc::c_uint = 5;
pub const DEVICE_ID_LIGHTGUN_START: libc::c_uint = 6;

// ID values for POINTER.
pub const DEVICE_ID_POINTER_X: libc::c_uint = 0;
pub const DEVICE_ID_POINTER_Y: libc::c_uint = 1;
pub const DEVICE_ID_POINTER_PRESSED: libc::c_uint = 2;

define_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    #[repr(C)]
    pub enum Region {
        NTSC = 0,
        PAL = 1,
    }
}

define_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    #[repr(C)]
    pub enum Language {
        English = 0,
        Japanese = 1,
        French = 2,
        Spanish = 3,
        German = 4,
        Italian = 5,
        Dutch = 6,
        Portuguese = 7,
        Russian = 8,
        Korean = 9,
        ChineseTraditional = 10,
        ChineseSimplified = 11,
        Esperanto = 12,
        Polish = 13,
    }
}

// Passed to retro_get_memory_data/size().
// If the memory type doesn't apply to the
// implementation NULL/0 can be returned.
pub const MEMORY_MASK: libc::c_uint = 0xff;

// Regular save RAM. This RAM is usually found on a game cartridge,
// backed up by a battery.
//
// If save game data is too complex for a single memory buffer,
// the SAVE_DIRECTORY (preferably) or SYSTEM_DIRECTORY environment
// callback can be used.
pub const MEMORY_SAVE_RAM: libc::c_uint = 0;

// Some games have a built-in clock to keep track of time.
// This memory is usually just a couple of bytes to keep track of time.
pub const MEMORY_RTC: libc::c_uint = 1;

// System ram lets a frontend peek into a game systems main RAM.
pub const MEMORY_SYSTEM_RAM: libc::c_uint = 2;

// Video ram lets a frontend peek into a game systems video RAM (VRAM).
pub const MEMORY_VIDEO_RAM: libc::c_uint = 3;

define_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    #[repr(C)]
    pub enum Key {
        Unknown = 0,

        Backspace = 8,
        Tab = 9,
        Clear = 12,
        Return = 13,
        Pause = 19,
        Escape = 27,
        Space = 32,
        ExclamationMark = 33,
        DoubleQuotes = 34,
        Hash = 35,
        Dollar = 36,
        Ampersand = 38,
        Quote = 39,
        LeftParen = 40,
        RightParen = 41,
        Asterisk = 42,
        Plus = 43,
        Comma = 44,
        Minus = 45,
        Period = 46,
        Slash = 47,
        Number_0 = 48,
        Number_1 = 49,
        Number_2 = 50,
        Number_3 = 51,
        Number_4 = 52,
        Number_5 = 53,
        Number_6 = 54,
        Number_7 = 55,
        Number_8 = 56,
        Number_9 = 57,
        Colon = 58,
        Semicolon = 59,
        Less = 60,
        Equals = 61,
        Greater = 62,
        QuestionMark = 63,
        At = 64,
        LeftBracket = 91,
        Backslash = 92,
        RightBracket = 93,
        Caret = 94,
        Underscore = 95,
        Backquote = 96,
        A = 97,
        B = 98,
        C = 99,
        D = 100,
        E = 101,
        F = 102,
        G = 103,
        H = 104,
        I = 105,
        J = 106,
        K = 107,
        L = 108,
        M = 109,
        N = 110,
        O = 111,
        P = 112,
        Q = 113,
        R = 114,
        S = 115,
        T = 116,
        U = 117,
        V = 118,
        W = 119,
        X = 120,
        Y = 121,
        Z = 122,
        Delete = 127,

        Kp0 = 256,
        Kp1 = 257,
        Kp2 = 258,
        Kp3 = 259,
        Kp4 = 260,
        Kp5 = 261,
        Kp6 = 262,
        Kp7 = 263,
        Kp8 = 264,
        Kp9 = 265,
        KpPeriod = 266,
        KpDivide = 267,
        KpMultiply = 268,
        KpMinus = 269,
        KpPlus = 270,
        KpEnter = 271,
        KpEquals = 272,

        Up = 273,
        Down = 274,
        Right = 275,
        Left = 276,
        Insert = 277,
        Home = 278,
        End = 279,
        PageUp = 280,
        PageDown = 281,

        F1 = 282,
        F2 = 283,
        F3 = 284,
        F4 = 285,
        F5 = 286,
        F6 = 287,
        F7 = 288,
        F8 = 289,
        F9 = 290,
        F10 = 291,
        F11 = 292,
        F12 = 293,
        F13 = 294,
        F14 = 295,
        F15 = 296,

        Numlock = 300,
        Capslock = 301,
        Scrollock = 302,
        RShift = 303,
        LShift = 304,
        RCtrl = 305,
        LCtrl = 306,
        RAlt = 307,
        LAlt = 308,
        RMeta = 309,
        LMeta = 310,
        LSuper = 311,
        RSuper = 312,
        Mode = 313,
        Compose = 314,

        Help = 315,
        Print = 316,
        Sysrq = 317,
        Break = 318,
        Menu = 319,
        Power = 320,
        Euro = 321,
        Undo = 322,
    }
}

define_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    #[repr(C)]
    pub enum Mod {
        None = 0x0000,

        Shift = 0x01,
        Ctrl = 0x02,
        Alt = 0x04,
        Meta = 0x08,

        Numlock = 0x10,
        Capslock = 0x20,
        Scrollock = 0x40,
    }
}

// If set, this call is not part of the public libretro API yet. It can
// change or be removed at any time.
pub const ENVIRONMENT_EXPERIMENTAL: libc::c_uint = 0x10000;

// Environment callback to be used internally in frontend.
pub const ENVIRONMENT_PRIVATE: libc::c_uint = 0x20000;

// Environment commands.

// const unsigned * --
// Sets screen rotation of graphics.
// Is only implemented if rotation can be accelerated by hardware.
// Valid values are 0, 1, 2, 3, which rotates screen by 0, 90, 180,
// 270 degrees counter-clockwise respectively.
pub const ENVIRONMENT_SET_ROTATION: libc::c_uint = 1;

//  bool * --
// Boolean value whether or not the implementation should use overscan,
// or crop away overscan.
pub const ENVIRONMENT_GET_OVERSCAN: libc::c_uint = 2;

// bool * --
// Boolean value whether or not frontend supports frame duping,
// passing NULL to video frame callback.
pub const ENVIRONMENT_GET_CAN_DUPE: libc::c_uint = 3;

// Environ 4, 5 are no longer supported (GET_VARIABLE / SET_VARIABLES),
// and reserved to avoid possible ABI clash.

// const struct Message * --
// Sets a message to be displayed in implementation-specific manner
// for a certain amount of 'frames'.
// Should not be used for trivial messages, which should simply be
// logged via ENVIRONMENT_GET_LOG_INTERFACE (or as a
// fallback, stderr).
pub const ENVIRONMENT_SET_MESSAGE: libc::c_uint = 6;

// N/A (NULL) --
// Requests the frontend to shutdown.
// Should only be used if game has a specific
// way to shutdown the game from a menu item or similar.
pub const ENVIRONMENT_SHUTDOWN: libc::c_uint = 7;

// const unsigned * --
// Gives a hint to the frontend how demanding this implementation
// is on a system. E.g. reporting a level of 2 means
// this implementation should run decently on all frontends
// of level 2 and up.
//
// It can be used by the frontend to potentially warn
// about too demanding implementations.
//
// The levels are "floating".
//
// This function can be called on a per-game basis,
// as certain games an implementation can play might be
// particularly demanding.
//
// If called, it should be called in retro_load_game().
pub const ENVIRONMENT_SET_PERFORMANCE_LEVEL: libc::c_uint = 8;

// const char ** --
// Returns the "system" directory of the frontend.
// This directory can be used to store system specific
// content such as BIOSes, configuration data, etc.
// The returned value can be NULL.
// If so, no such directory is defined,
// and it's up to the implementation to find a suitable directory.
//
// NOTE: Some cores used this folder also for "save" data such as
// memory cards, etc, for lack of a better place to put it.
// This is now discouraged, and if possible, cores should try to
// use the new GET_SAVE_DIRECTORY.
pub const ENVIRONMENT_GET_SYSTEM_DIRECTORY: libc::c_uint = 9;

// const enum PixelFormat * --
// Sets the internal pixel format used by the implementation.
// The default pixel format is RGB1555.
// This pixel format however, is deprecated (see enum PixelFormat).
// If the call returns false, the frontend does not support this pixel
// format.
//
// This function should be called inside retro_load_game() or
// retro_get_system_av_info().
pub const ENVIRONMENT_SET_PIXEL_FORMAT: libc::c_uint = 10;

// const struct InputDescriptor * --
// Sets an array of retro_input_descriptors.
// It is up to the frontend to present this in a usable way.
// The array is terminated by retro_input_descriptor::description
// being set to NULL.
//
// This function can be called at any time, but it is recommended
// to call it as early as possible.
pub const ENVIRONMENT_SET_INPUT_DESCRIPTORS: libc::c_uint = 11;

// const struct KeyboardCallback * --
// Sets a callback function used to notify core about keyboard events.
pub const ENVIRONMENT_SET_KEYBOARD_CALLBACK: libc::c_uint = 12;

// const struct DiskControlCallback * --
// Sets an interface which frontend can use to eject and insert
// disk images.
//
// This is used for games which consist of multiple images and
// must be manually swapped out by the user (e.g. PSX).
pub const ENVIRONMENT_SET_DISK_CONTROL_INTERFACE: libc::c_uint = 13;

// struct HwRenderCallback * --
// Sets an interface to let a libretro core render with
// hardware acceleration.
//
// Should be called in retro_load_game().
//
// If successful, libretro cores will be able to render to a
// frontend-provided framebuffer.
//
// The size of this framebuffer will be at least as large as
// max_width/max_height provided in get_av_info().
//
// If HW rendering is used, pass only HW_FRAME_BUFFER_VALID or
// NULL to retro_video_refresh_t.
pub const ENVIRONMENT_SET_HW_RENDER: libc::c_uint = 14;

// struct Variable * --
// Interface to acquire user-defined information from environment
// that cannot feasibly be supported in a multi-system way.
// 'key' should be set to a key which has already been set by
// SET_VARIABLES.
//
// 'data' will be set to a value or NULL.
pub const ENVIRONMENT_GET_VARIABLE: libc::c_uint = 15;

// const struct Variable * --
// Allows an implementation to signal the environment
// which variables it might want to check for later using
// GET_VARIABLE.
// This allows the frontend to present these variables to
// a user dynamically.
// This should be called as early as possible (ideally in
// retro_set_environment).
//
// 'data' points to an array of retro_variable structs
// terminated by a { NULL, NULL } element.
// retro_variable::key should be namespaced to not collide
// with other implementations' keys. E.g. A core called
// 'foo' should use keys named as 'foo_option'.
// retro_variable::value should contain a human readable
// description of the key as well as a '|' delimited list
// of expected values.
//
// The number of possible options should be very limited,
// i.e. it should be feasible to cycle through options
// without a keyboard.
//
// First entry should be treated as a default.
//
// Example entry:
// { "foo_option", "Speed hack coprocessor X; false|true" }
//
// Text before first ';' is description. This ';' must be
// followed by a space, and followed by a list of possible
// values split up with '|'.
//
// Only strings are operated on. The possible values will
// generally be displayed and stored as-is by the frontend.
pub const ENVIRONMENT_SET_VARIABLES: libc::c_uint = 16;

// bool * --
// Result is set to true if some variables are updated by
// frontend since last call to ENVIRONMENT_GET_VARIABLE.
// Variables should be queried with GET_VARIABLE.
pub const ENVIRONMENT_GET_VARIABLE_UPDATE: libc::c_uint = 17;

// const bool * --
// If true, the libretro implementation supports calls to
// retro_load_game() with NULL as argument.
//
// Used by cores which can run without particular game data.
// This should be called within retro_set_environment() only.
pub const ENVIRONMENT_SET_SUPPORT_NO_GAME: libc::c_uint = 18;

// const char ** --
// Retrieves the absolute path from where this libretro
// implementation was loaded.
// NULL is returned if the libretro was loaded statically
// (i.e. linked statically to frontend), or if the path cannot be
// determined.
//
// Mostly useful in cooperation with SET_SUPPORT_NO_GAME as assets can
// be loaded without ugly hacks.
pub const ENVIRONMENT_GET_LIBRETRO_PATH: libc::c_uint = 19;

// Environment 20 was an obsolete version of SET_AUDIO_CALLBACK.
// It was not used by any known core at the time,
// and was removed from the API.

// const struct AudioCallback * --
// Sets an interface which is used to notify a libretro core about audio
// being available for writing.
// The callback can be called from any thread, so a core using this must
// have a thread safe audio implementation.
// It is intended for games where audio and video are completely
// asynchronous and audio can be generated on the fly.
// This interface is not recommended for use with emulators which have
// highly synchronous audio.
//
// The callback only notifies about writability; the libretro core still
// has to call the normal audio callbacks
// to write audio. The audio callbacks must be called from within the
// notification callback.
// The amount of audio data to write is up to the implementation.
// Generally, the audio callback will be called continously in a loop.
//
// Due to thread safety guarantees and lack of sync between audio and
// video, a frontend  can selectively disallow this interface based on
// internal configuration. A core using this interface must also
// implement the "normal" audio interface.
//
// A libretro core using SET_AUDIO_CALLBACK should also make use of
// SET_FRAME_TIME_CALLBACK.
pub const ENVIRONMENT_SET_AUDIO_CALLBACK: libc::c_uint = 22;

// const struct FrameTimeCallback * --
// Lets the core know how much time has passed since last
// invocation of retro_run().
// The frontend can tamper with the timing to fake fast-forward,
// slow-motion, frame stepping, etc.
// In this case the delta time will use the reference value
// in frame_time_callback..
pub const ENVIRONMENT_SET_FRAME_TIME_CALLBACK: libc::c_uint = 21;

// struct RumbleInterface * --
// Gets an interface which is used by a libretro core to set
// state of rumble motors in controllers.
// A strong and weak motor is supported, and they can be
// controlled indepedently.
pub const ENVIRONMENT_GET_RUMBLE_INTERFACE: libc::c_uint = 23;

// uint64_t * --
// Gets a bitmask telling which device type are expected to be
// handled properly in a call to retro_input_state_t.
// Devices which are not handled or recognized always return
// 0 in retro_input_state_t.
// Example bitmask: caps = (1 << DEVICE_JOYPAD) | (1 << DEVICE_ANALOG).
// Should only be called in retro_run().
pub const ENVIRONMENT_GET_INPUT_DEVICE_CAPABILITIES: libc::c_uint = 24;

// struct SensorInterface * --
// Gets access to the sensor interface.
// The purpose of this interface is to allow
// setting state related to sensors such as polling rate,
// enabling/disable it entirely, etc.
// Reading sensor state is done via the normal
// input_state_callback API.
pub const ENVIRONMENT_GET_SENSOR_INTERFACE: libc::c_uint = (25 | ENVIRONMENT_EXPERIMENTAL);

// struct CameraCallback * --
// Gets an interface to a video camera driver.
// A libretro core can use this interface to get access to a
// video camera.
// New video frames are delivered in a callback in same
// thread as retro_run().
//
// GET_CAMERA_INTERFACE should be called in retro_load_game().
//
// Depending on the camera implementation used, camera frames
// will be delivered as a raw framebuffer,
// or as an OpenGL texture directly.
//
// The core has to tell the frontend here which types of
// buffers can be handled properly.
// An OpenGL texture can only be handled when using a
// libretro GL core (SET_HW_RENDER).
// It is recommended to use a libretro GL core when
// using camera interface.
//
// The camera is not started automatically. The retrieved start/stop
// functions must be used to explicitly
// start and stop the camera driver.
//
pub const ENVIRONMENT_GET_CAMERA_INTERFACE: libc::c_uint = (26 | ENVIRONMENT_EXPERIMENTAL);

// struct LogCallback * --
// Gets an interface for logging. This is useful for
// logging in a cross-platform way
// as certain platforms cannot use use stderr for logging.
// It also allows the frontend to
// show logging information in a more suitable way.
// If this interface is not used, libretro cores should
// log to stderr as desired.
pub const ENVIRONMENT_GET_LOG_INTERFACE: libc::c_uint = 27;

// struct PerfCallback * --
// Gets an interface for performance counters. This is useful
// for performance logging in a cross-platform way and for detecting
// architecture-specific features, such as SIMD support.
pub const ENVIRONMENT_GET_PERF_INTERFACE: libc::c_uint = 28;

// struct LocationCallback * --
// Gets access to the location interface.
// The purpose of this interface is to be able to retrieve
// location-based information from the host device,
// such as current latitude / longitude.
pub const ENVIRONMENT_GET_LOCATION_INTERFACE: libc::c_uint = 29;

// const char ** --
// Returns the "core assets" directory of the frontend.
// This directory can be used to store specific assets that the
// core relies upon, such as art assets,
// input data, etc etc.
// The returned value can be NULL.
// If so, no such directory is defined,
// and it's up to the implementation to find a suitable directory.
pub const ENVIRONMENT_GET_CORE_ASSETS_DIRECTORY: libc::c_uint = 30;

// const char ** --
// Returns the "save" directory of the frontend.
// This directory can be used to store SRAM, memory cards,
// high scores, etc, if the libretro core
// cannot use the regular memory interface (retro_get_memory_data()).
//
// NOTE: libretro cores used to check GET_SYSTEM_DIRECTORY for
// similar things before.
// They should still check GET_SYSTEM_DIRECTORY if they want to
// be backwards compatible.
// The path here can be NULL. It should only be non-NULL if the
// frontend user has set a specific save path.
pub const ENVIRONMENT_GET_SAVE_DIRECTORY: libc::c_uint = 31;

// const struct SystemAvInfo * --
// Sets a new av_info structure. This can only be called from
// within retro_run().
// This should *only* be used if the core is completely altering the
// internal resolutions, aspect ratios, timings, sampling rate, etc.
// Calling this can require a full reinitialization of video/audio
// drivers in the frontend, so it is important to call it very sparingly,
// and usually only with the users explicit consent.
//
// An eventual driver reinitialize will happen so that video and
// audio callbacks happening after this call within the same retro_run()
// call will target the newly initialized driver.
//
// This callback makes it possible to support configurable resolutions
// in games, which can be useful to avoid setting the "worst case" in max_width/max_height.
//
// *** HIGHLY RECOMMENDED*** Do not call this callback every time
// resolution changes in an emulator core if it's
// expected to be a temporary change, for the reasons of possible
// driver reinitialization.
// This call is not a free pass for not trying to provide
// correct values in retro_get_system_av_info(). If you need to change
// things like aspect ratio or nominal width/height,
// use ENVIRONMENT_SET_GEOMETRY, which is a softer variant
// of SET_SYSTEM_AV_INFO.
//
// If this returns false, the frontend does not acknowledge a
// changed av_info struct.
pub const ENVIRONMENT_SET_SYSTEM_AV_INFO: libc::c_uint = 32;

// const struct GetProcAddressInterface * --
// Allows a libretro core to announce support for the
// get_proc_address() interface.
// This interface allows for a standard way to extend libretro where
// use of environment calls are too indirect,
// e.g. for cases where the frontend wants to call directly into the core.
//
// If a core wants to expose this interface, SET_PROC_ADDRESS_CALLBACK
// ** MUST** be called from within retro_set_environment().
pub const ENVIRONMENT_SET_PROC_ADDRESS_CALLBACK: libc::c_uint = 33;

// const struct SubsystemInfo * --
// This environment call introduces the concept of libretro "subsystems".
// A subsystem is a variant of a libretro core which supports
// different kinds of games.
// The purpose of this is to support e.g. emulators which might
// have special needs, e.g. Super Nintendo's Super GameBoy, Sufami Turbo.
// It can also be used to pick among subsystems in an explicit way
// if the libretro implementation is a multi-system emulator itself.
//
// Loading a game via a subsystem is done with retro_load_game_special(),
// and this environment call allows a libretro core to expose which
// subsystems are supported for use with retro_load_game_special().
// A core passes an array of retro_game_special_info which is terminated
// with a zeroed out retro_game_special_info struct.
//
// If a core wants to use this functionality, SET_SUBSYSTEM_INFO
// ** MUST** be called from within retro_set_environment().
pub const ENVIRONMENT_SET_SUBSYSTEM_INFO: libc::c_uint = 34;

// const struct ControllerInfo * --
// This environment call lets a libretro core tell the frontend
// which controller types are recognized in calls to
// retro_set_controller_port_device().
//
// Some emulators such as Super Nintendo
// support multiple lightgun types which must be specifically
// selected from.
// It is therefore sometimes necessary for a frontend to be able
// to tell the core about a special kind of input device which is
// not covered by the libretro input API.
//
// In order for a frontend to understand the workings of an input device,
// it must be a specialized type
// of the generic device types already defined in the libretro API.
//
// Which devices are supported can vary per input port.
// The core must pass an array of const struct ControllerInfo which
// is terminated with a blanked out struct. Each element of the struct
// corresponds to an ascending port index to
// retro_set_controller_port_device().
// Even if special device types are set in the libretro core,
// libretro should only poll input based on the base input device types.
pub const ENVIRONMENT_SET_CONTROLLER_INFO: libc::c_uint = 35;

// const struct MemoryMap * --
// This environment call lets a libretro core tell the frontend
// about the memory maps this core emulates.
// This can be used to implement, for example, cheats in a core-agnostic way.
//
// Should only be used by emulators; it doesn't make much sense for
// anything else.
// It is recommended to expose all relevant pointers through
// retro_get_memory_* as well.
//
// Can be called from retro_init and retro_load_game.
pub const ENVIRONMENT_SET_MEMORY_MAPS: libc::c_uint = (36 | ENVIRONMENT_EXPERIMENTAL);

// const struct GameGeometry * --
// This environment call is similar to SET_SYSTEM_AV_INFO for changing
// video parameters, but provides a guarantee that drivers will not be
// reinitialized.
// This can only be called from within retro_run().
//
// The purpose of this call is to allow a core to alter nominal
// width/heights as well as aspect ratios on-the-fly, which can be
// useful for some emulators to change in run-time.
//
// max_width/max_height arguments are ignored and cannot be changed
// with this call as this could potentially require a reinitialization or a
// non-constant time operation.
// If max_width/max_height are to be changed, SET_SYSTEM_AV_INFO is required.
//
// A frontend must guarantee that this environment call completes in
// constant time.
pub const ENVIRONMENT_SET_GEOMETRY: libc::c_uint = 37;

// const char **
// Returns the specified username of the frontend, if specified by the user.
// This username can be used as a nickname for a core that has online facilities
// or any other mode where personalization of the user is desirable.
// The returned value can be NULL.
// If this environ callback is used by a core that requires a valid username,
// a default username should be specified by the core.
pub const ENVIRONMENT_GET_USERNAME: libc::c_uint = 38;

// unsigned * --
// Returns the specified language of the frontend, if specified by the user.
// It can be used by the core for localization purposes.
pub const ENVIRONMENT_GET_LANGUAGE: libc::c_uint = 39;

// struct Framebuffer * --
// Returns a preallocated framebuffer which the core can use for rendering
// the frame into when not using SET_HW_RENDER.
// The framebuffer returned from this call must not be used
// after the current call to retro_run() returns.
//
// The goal of this call is to allow zero-copy behavior where a core
// can render directly into video memory, avoiding extra bandwidth cost by copying
// memory from core to video memory.
//
// If this call succeeds and the core renders into it,
// the framebuffer pointer and pitch can be passed to retro_video_refresh_t.
// If the buffer from GET_CURRENT_SOFTWARE_FRAMEBUFFER is to be used,
// the core must pass the exact
// same pointer as returned by GET_CURRENT_SOFTWARE_FRAMEBUFFER;
// i.e. passing a pointer which is offset from the
// buffer is undefined. The width, height and pitch parameters
// must also match exactly to the values obtained from GET_CURRENT_SOFTWARE_FRAMEBUFFER.
//
// It is possible for a frontend to return a different pixel format
// than the one used in SET_PIXEL_FORMAT. This can happen if the frontend
// needs to perform conversion.
//
// It is still valid for a core to render to a different buffer
// even if GET_CURRENT_SOFTWARE_FRAMEBUFFER succeeds.
//
// A frontend must make sure that the pointer obtained from this function is
// writeable (and readable).
pub const ENVIRONMENT_GET_CURRENT_SOFTWARE_FRAMEBUFFER: libc::c_uint = (40 | ENVIRONMENT_EXPERIMENTAL);

define_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    #[repr(C)]
    pub enum HwRenderInterfaceType {
        Vulkan = 0,
    }
}

// Base struct. All retro_hw_render_interface_* types
// contain at least these fields.
#[derive(Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct HwRenderInterface {
    // HwRenderInterfaceType
    pub interface_type: libc::c_uint,
    pub interface_version: libc::c_uint,
}

// const struct HwRenderInterface ** --
// Returns an API specific rendering interface for accessing API specific data.
// Not all HW rendering APIs support or need this.
// The contents of the returned pointer is specific to the rendering API
// being used. See the various headers like libretro_vulkan.h, etc.
//
// GET_HW_RENDER_INTERFACE cannot be called before context_reset has been called.
// Similarly, after context_destroyed callback returns,
// the contents of the HW_RENDER_INTERFACE are invalidated.
pub const ENVIRONMENT_GET_HW_RENDER_INTERFACE: libc::c_uint = (41 | ENVIRONMENT_EXPERIMENTAL);

pub const MEMDESC_CONST: libc::c_uint = (1 << 0); // The frontend will never change this memory area once retro_load_game has returned.
pub const MEMDESC_BIGENDIAN: libc::c_uint = (1 << 1); // The memory area contains big endian data. Default is little endian.
pub const MEMDESC_ALIGN_2: libc::c_uint = (1 << 16); // All memory access in this area is aligned to their own size, or 2, whichever is smaller.
pub const MEMDESC_ALIGN_4: libc::c_uint = (2 << 16);
pub const MEMDESC_ALIGN_8: libc::c_uint = (3 << 16);
pub const MEMDESC_MINSIZE_2: libc::c_uint = (1 << 24); // All memory in this region is accessed at least 2 bytes at the time.
pub const MEMDESC_MINSIZE_4: libc::c_uint = (2 << 24);
pub const MEMDESC_MINSIZE_8: libc::c_uint = (3 << 24);

#[derive(Clone, Debug)]
#[repr(C)]
pub struct MemoryDescriptor {
    pub flags: u64,

    // Pointer to the start of the relevant ROM or RAM chip.
    // It's strongly recommended to use 'offset' if possible, rather than
    // doing math on the pointer.
    //
    // If the same byte is mapped my multiple descriptors, their descriptors
    // must have the same pointer.
    // If 'start' does not point to the first byte in the pointer, put the
    // difference in 'offset' instead.
    //
    // May be NULL if there's nothing usable here (e.g. hardware registers and
    // open bus). No flags should be set if the pointer is NULL.
    // It's recommended to minimize the number of descriptors if possible,
    // but not mandatory.
    pub ptr: *mut libc::c_void,
    pub offset: libc::size_t,

    // This is the location in the emulated address space
    // where the mapping starts.
    pub start: libc::size_t,

    // Which bits must be same as in 'start' for this mapping to apply.
    // The first memory descriptor to claim a certain byte is the one
    // that applies.
    // A bit which is set in 'start' must also be set in this.
    // Can be zero, in which case each byte is assumed mapped exactly once.
    // In this case, 'len' must be a power of two.
    pub select: libc::size_t,

    // If this is nonzero, the set bits are assumed not connected to the
    // memory chip's address pins.
    pub disconnect: libc::size_t,

    // This one tells the size of the current memory area.
    // If, after start+disconnect are applied, the address is higher than
    // this, the highest bit of the address is cleared.
    //
    // If the address is still too high, the next highest bit is cleared.
    // Can be zero, in which case it's assumed to be infinite (as limited
    // by 'select' and 'disconnect').
    pub len: libc::size_t,

    // To go from emulated address to physical address, the following
    // order applies:
    // Subtract 'start', pick off 'disconnect', apply 'len', add 'offset'.
    //
    // The address space name must consist of only a-zA-Z0-9_-,
    // should be as short as feasible (maximum length is 8 plus the NUL),
    // and may not be any other address space plus one or more 0-9A-F
    // at the end.
    // However, multiple memory descriptors for the same address space is
    // allowed, and the address space name can be empty. NULL is treated
    // as empty.
    //
    // Address space names are case sensitive, but avoid lowercase if possible.
    // The same pointer may exist in multiple address spaces.
    //
    // Examples:
    // blank+blank - valid (multiple things may be mapped in the same namespace)
    // 'Sp'+'Sp' - valid (multiple things may be mapped in the same namespace)
    // 'A'+'B' - valid (neither is a prefix of each other)
    // 'S'+blank - valid ('S' is not in 0-9A-F)
    // 'a'+blank - valid ('a' is not in 0-9A-F)
    // 'a'+'A' - valid (neither is a prefix of each other)
    // 'AR'+blank - valid ('R' is not in 0-9A-F)
    // 'ARB'+blank - valid (the B can't be part of the address either, because
    // there is no namespace 'AR')
    // blank+'B' - not valid, because it's ambigous which address space B1234
    // would refer to.
    // The length can't be used for that purpose; the frontend may want
    // to append arbitrary data to an address, without a separator.
    pub addrspace: *const libc::c_char,
}

// The frontend may use the largest value of 'start'+'select' in a
// certain namespace to infer the size of the address space.
//
// If the address space is larger than that, a mapping with .ptr=NULL
// should be at the end of the array, with .select set to all ones for
// as long as the address space is big.
//
// Sample descriptors (minus .ptr, and MEMFLAG_ on the flags):
// SNES WRAM:
// .start=0x7E0000, .len=0x20000
// (Note that this must be mapped before the ROM in most cases; some of the
// ROM mappers
// try to claim $7E0000, or at least $7E8000.)
// SNES SPC700 RAM:
// .addrspace="S", .len=0x10000
// SNES WRAM mirrors:
// .flags=MIRROR, .start=0x000000, .select=0xC0E000, .len=0x2000
// .flags=MIRROR, .start=0x800000, .select=0xC0E000, .len=0x2000
// SNES WRAM mirrors, alternate equivalent descriptor:
// .flags=MIRROR, .select=0x40E000, .disconnect=~0x1FFF
// (Various similar constructions can be created by combining parts of
// the above two.)
// SNES LoROM (512KB, mirrored a couple of times):
// .flags=CONST, .start=0x008000, .select=0x408000, .disconnect=0x8000, .len=512*1024
// .flags=CONST, .start=0x400000, .select=0x400000, .disconnect=0x8000, .len=512*1024
// SNES HiROM (4MB):
// .flags=CONST,                 .start=0x400000, .select=0x400000, .len=4*1024*1024
// .flags=CONST, .offset=0x8000, .start=0x008000, .select=0x408000, .len=4*1024*1024
// SNES ExHiROM (8MB):
// .flags=CONST, .offset=0,                  .start=0xC00000, .select=0xC00000, .len=4*1024*1024
// .flags=CONST, .offset=4*1024*1024,        .start=0x400000, .select=0xC00000, .len=4*1024*1024
// .flags=CONST, .offset=0x8000,             .start=0x808000, .select=0xC08000, .len=4*1024*1024
// .flags=CONST, .offset=4*1024*1024+0x8000, .start=0x008000, .select=0xC08000, .len=4*1024*1024
// Clarify the size of the address space:
// .ptr=NULL, .select=0xFFFFFF
// .len can be implied by .select in many of them, but was included for clarity.

#[derive(Clone, Debug)]
#[repr(C)]
pub struct MemoryMap {
    pub descriptors: *const MemoryDescriptor,
    pub num_descriptors: libc::c_uint,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ControllerDescription {
    // Human-readable description of the controller. Even if using a generic
    // input device type, this can be set to the particular device type the
    // core uses.
    pub desc: *const libc::c_char,

    // Device type passed to retro_set_controller_port_device(). If the device
    // type is a sub-class of a generic input device type, use the
    // DEVICE_SUBCLASS macro to create an ID.
    //
    // E.g. DEVICE_SUBCLASS(DEVICE_JOYPAD, 1).
    pub id: libc::c_uint,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ControllerInfo {
    pub types: *const ControllerDescription,
    pub num_types: libc::c_uint,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct SubsystemMemoryInfo {
    // The extension associated with a memory type, e.g. "psram".
    pub extension: *const libc::c_char,

    // The memory type for retro_get_memory(). This should be at
    // least 0x100 to avoid conflict with standardized
    // libretro memory types.
    pub kind: libc::c_uint,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct SubsystemRomInfo {
    // Describes what the content is (SGB BIOS, GB ROM, etc).
    pub desc: *const libc::c_char,

    // Same definition as retro_get_system_info().
    pub valid_extensions: *const libc::c_char,

    // Same definition as retro_get_system_info().
    pub need_fullpath: bool,

    // Same definition as retro_get_system_info().
    pub block_extract: bool,

    // This is set if the content is required to load a game.
    // If this is set to false, a zeroed-out retro_game_info can be passed.
    pub required: bool,

    // Content can have multiple associated persistent
    // memory types (retro_get_memory()).
    pub memory: *const SubsystemMemoryInfo,
    pub num_memory: libc::c_uint,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct SubsystemInfo {
    // Human-readable string of the subsystem type, e.g. "Super GameBoy"
    pub desc: *const libc::c_char,

    // A computer friendly short string identifier for the subsystem type.
    // This name must be [a-z].
    // E.g. if desc is "Super GameBoy", this can be "sgb".
    // This identifier can be used for command-line interfaces, etc.
    //
    pub ident: *const libc::c_char,

    // Infos for each content file. The first entry is assumed to be the
    // "most significant" content for frontend purposes.
    // E.g. with Super GameBoy, the first content should be the GameBoy ROM,
    // as it is the most "significant" content to a user.
    // If a frontend creates new file paths based on the content used
    // (e.g. savestates), it should use the path for the first ROM to do so.
    pub roms: *const SubsystemRomInfo,

    // Number of content files associated with a subsystem.
    pub num_roms: libc::c_uint,

    // The type passed to retro_load_game_special().
    pub id: libc::c_uint,
}

pub type ProcAddressFn = unsafe extern "C" fn();

// libretro API extension functions:
// (None here so far).
//
// Get a symbol from a libretro core.
// Cores should only return symbols which are actual
// extensions to the libretro API.
//
// Frontends should not use this to obtain symbols to standard
// libretro entry points (static linking or dlsym).
//
// The symbol name must be equal to the function name,
// e.g. if void retro_foo(void); exists, the symbol must be called "retro_foo".
// The returned function pointer must be cast to the corresponding type.
//
pub type GetProcAddressFn = unsafe extern "C" fn(sym: *const libc::c_char) -> ProcAddressFn;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct GetProcAddressInterface {
    pub get_proc_address: GetProcAddressFn,
}

define_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    #[repr(C)]
    pub enum LogLevel {
        Debug = 0,
        Info = 1,
        Warn = 2,
        Error = 3,
    }
}

// Logging function. Takes log level argument as well.
pub type LogPrintfFn = unsafe extern "C" fn(level: LogLevel, fmt: *const libc::c_char);

#[derive(Clone, Debug)]
#[repr(C)]
pub struct LogCallback {
    pub log: LogPrintfFn,
}

// Performance related functions

// ID values for SIMD CPU features
pub const SIMD_SSE: libc::c_uint = (1 << 0);
pub const SIMD_SSE2: libc::c_uint = (1 << 1);
pub const SIMD_VMX: libc::c_uint = (1 << 2);
pub const SIMD_VMX128: libc::c_uint = (1 << 3);
pub const SIMD_AVX: libc::c_uint = (1 << 4);
pub const SIMD_NEON: libc::c_uint = (1 << 5);
pub const SIMD_SSE3: libc::c_uint = (1 << 6);
pub const SIMD_SSSE3: libc::c_uint = (1 << 7);
pub const SIMD_MMX: libc::c_uint = (1 << 8);
pub const SIMD_MMXEXT: libc::c_uint = (1 << 9);
pub const SIMD_SSE4: libc::c_uint = (1 << 10);
pub const SIMD_SSE42: libc::c_uint = (1 << 11);
pub const SIMD_AVX2: libc::c_uint = (1 << 12);
pub const SIMD_VFPU: libc::c_uint = (1 << 13);
pub const SIMD_PS: libc::c_uint = (1 << 14);
pub const SIMD_AES: libc::c_uint = (1 << 15);
pub const SIMD_VFPV3: libc::c_uint = (1 << 16);
pub const SIMD_VFPV4: libc::c_uint = (1 << 17);
pub const SIMD_POPCNT: libc::c_uint = (1 << 18);
pub const SIMD_MOVBE: libc::c_uint = (1 << 19);

pub type PerfTick = u64;
pub type Time = i64;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct PerfCounter {
    pub ident: *const libc::c_char,
    pub start: PerfTick,
    pub total: PerfTick,
    pub call_cnt: PerfTick,

    pub registered: bool,
}

// Returns current time in microseconds.
// Tries to use the most accurate timer available.
pub type PerfGetTimeUsecFn = unsafe extern "C" fn() -> Time;

// A simple counter. Usually nanoseconds, but can also be CPU cycles.
// Can be used directly if desired (when creating a more sophisticated
// performance counter system).
pub type PerfGetCounterFn = unsafe extern "C" fn() -> PerfTick;

// Returns a bit-mask of detected CPU features (SIMD_*).
pub type GetCpuFeaturesFn = unsafe extern "C" fn() -> u64;

// Asks frontend to log and/or display the state of performance counters.
// Performance counters can always be poked into manually as well.
pub type PerfLogFn = unsafe extern "C" fn();

// Register a performance counter.
// ident field must be set with a discrete value and other values in
// retro_perf_counter must be 0.
// Registering can be called multiple times. To avoid calling to
// frontend redundantly, you can check registered field first.
pub type PerfRegisterFn = unsafe extern "C" fn(counter: *mut PerfCounter);

// Starts a registered counter.
pub type PerfStartFn = unsafe extern "C" fn(counter: *mut PerfCounter);

// Stops a registered counter.
pub type PerfStopFn = unsafe extern "C" fn(counter: *mut PerfCounter);

// For convenience it can be useful to wrap register, start and stop in macros.
// E.g.:
// #ifdef LOG_PERFORMANCE
// #define PERFORMANCE_INIT(perf_cb, name) static struct PerfCounter name = {#name}; if (!name.registered) perf_cb.perf_register(&(name))
// #define PERFORMANCE_START(perf_cb, name) perf_cb.perf_start(&(name))
// #define PERFORMANCE_STOP(perf_cb, name) perf_cb.perf_stop(&(name))
// #else
// ... Blank macros ...
// #endif
//
// These can then be used mid-functions around code snippets.
//
// extern struct PerfCallback perf_cb;  * Somewhere in the core.
//
// void do_some_heavy_work(void)
// {
//    PERFORMANCE_INIT(cb, work_1;
//    PERFORMANCE_START(cb, work_1);
//    heavy_work_1();
//    PERFORMANCE_STOP(cb, work_1);
//
//    PERFORMANCE_INIT(cb, work_2);
//    PERFORMANCE_START(cb, work_2);
//    heavy_work_2();
//    PERFORMANCE_STOP(cb, work_2);
// }
//
// void retro_deinit(void)
// {
//    perf_cb.perf_log();  * Log all perf counters here for example.
// }
//

#[derive(Clone, Debug)]
#[repr(C)]
pub struct PerfCallback {
    pub get_time_usec: PerfGetTimeUsecFn,
    pub get_cpu_features: GetCpuFeaturesFn,

    pub get_perf_counter: PerfGetCounterFn,
    pub perf_register: PerfRegisterFn,
    pub perf_start: PerfStartFn,
    pub perf_stop: PerfStopFn,
    pub perf_log: PerfLogFn,
}

define_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    #[repr(C)]
    pub enum SensorAction {
        AccelerometerEnable = 0,
        AccelerometerDisable = 1,
    }
}

// ID values for SENSOR types.
pub const SENSOR_ACCELEROMETER_X: libc::c_uint = 0;
pub const SENSOR_ACCELEROMETER_Y: libc::c_uint = 1;
pub const SENSOR_ACCELEROMETER_Z: libc::c_uint = 2;

pub type SetSensorStateFn = unsafe extern "C" fn(port: libc::c_uint, action: SensorAction, rate: libc::c_uint) -> bool;

pub type SensorGetInputFn = unsafe extern "C" fn(port: libc::c_uint, id: libc::c_uint) -> f32;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct SensorInterface {
    pub set_sensor_state: SetSensorStateFn,
    pub get_sensor_input: SensorGetInputFn,
}

define_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    #[repr(C)]
    pub enum CameraBuffer {
        OpenGLTexture = 0,
        RawFramebuffer = 1,
    }
}

// Starts the camera driver. Can only be called in retro_run().
pub type CameraStartFn = unsafe extern "C" fn() -> bool;

// Stops the camera driver. Can only be called in retro_run().
pub type CameraStopFn = unsafe extern "C" fn();

// Callback which signals when the camera driver is initialized
// and/or deinitialized.
// retro_camera_start_t can be called in initialized callback.
pub type CameraLifetimeStatusFn = unsafe extern "C" fn();

// A callback for raw framebuffer data. buffer points to an XRGB8888 buffer.
// Width, height and pitch are similar to retro_video_refresh_t.
// First pixel is top-left origin.
pub type CameraFrameRawFramebufferFn =
    unsafe extern "C" fn(buffer: *const u32, width: libc::c_uint, height: libc::c_uint, pitch: libc::size_t);

// A callback for when OpenGL textures are used.
//
// texture_id is a texture owned by camera driver.
// Its state or content should be considered immutable, except for things like
// texture filtering and clamping.
//
// texture_target is the texture target for the GL texture.
// These can include e.g. GL_TEXTURE_2D, GL_TEXTURE_RECTANGLE, and possibly
// more depending on extensions.
//
// affine points to a packed 3x3 column-major matrix used to apply an affine
// transform to texture coordinates. (affine_matrix * vec3(coord_x, coord_y, 1.0))
// After transform, normalized texture coord (0, 0) should be bottom-left
// and (1, 1) should be top-right (or (width, height) for RECTANGLE).
//
// GL-specific typedefs are avoided here to avoid relying on gl.h in
// the API definition.
pub type CameraFrameOpenglTextureFn =
    unsafe extern "C" fn(texture_id: libc::c_uint, texture_target: libc::c_uint, affine: *const f32);

#[derive(Clone, Debug)]
#[repr(C)]
pub struct CameraCallback {
    // Set by libretro core.
    // Example bitmask: caps = (1 << CAMERA_BUFFER_OPENGL_TEXTURE) | (1 << CAMERA_BUFFER_RAW_FRAMEBUFFER).
    pub caps: u64,

    // Desired resolution for camera. Is only used as a hint.
    pub width: libc::c_uint,
    pub height: libc::c_uint,

    // Set by frontend.
    pub start: CameraStartFn,
    pub stop: CameraStopFn,

    // Set by libretro core if raw framebuffer callbacks will be used.
    pub frame_raw_framebuffer: CameraFrameRawFramebufferFn,

    // Set by libretro core if OpenGL texture callbacks will be used.
    pub frame_opengl_texture: CameraFrameOpenglTextureFn,

    // Set by libretro core. Called after camera driver is initialized and
    // ready to be started.
    //
    // Can be NULL, in which this callback is not called.
    pub initialized: CameraLifetimeStatusFn,

    // Set by libretro core. Called right before camera driver is
    // deinitialized.
    //
    // Can be NULL, in which this callback is not called.
    pub deinitialized: CameraLifetimeStatusFn,
}

// Sets the interval of time and/or distance at which to update/poll
// location-based data.
//
// To ensure compatibility with all location-based implementations,
// values for both interval_ms and interval_distance should be provided.
//
// interval_ms is the interval expressed in milliseconds.
// interval_distance is the distance interval expressed in meters.
pub type LocationSetIntervalFn = unsafe extern "C" fn(interval_ms: libc::c_uint, interval_distance: libc::c_uint);

// Start location services. The device will start listening for changes to the
// current location at regular intervals (which are defined with
// retro_location_set_interval_t).
pub type LocationStartFn = unsafe extern "C" fn() -> bool;

// Stop location services. The device will stop listening for changes
// to the current location.
pub type LocationStopFn = unsafe extern "C" fn();

// Get the position of the current location. Will set parameters to
// 0 if no new  location update has happened since the last time.
pub type LocationGetPositionFn =
    unsafe extern "C" fn(lat: *mut f64, lon: *mut f64, horiz_accuracy: *mut f64, vert_accuracy: *mut f64) -> bool;

// Callback which signals when the location driver is initialized
// and/or deinitialized.
//
// retro_location_start_t can be called in initialized callback.
pub type LocationLifetimeStatusFn = unsafe extern "C" fn();

#[derive(Clone, Debug)]
#[repr(C)]
pub struct LocationCallback {
    pub start: LocationStartFn,
    pub stop: LocationStopFn,
    pub get_position: LocationGetPositionFn,
    pub set_interval: LocationSetIntervalFn,

    pub initialized: LocationLifetimeStatusFn,
    pub deinitialized: LocationLifetimeStatusFn,
}

define_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    #[repr(C)]
    pub enum RumbleEffect {
        Strong = 0,
        Weak = 1,
    }
}

// Sets rumble state for joypad plugged in port 'port'.
// Rumble effects are controlled independently,
// and setting e.g. strong rumble does not override weak rumble.
// Strength has a range of [0, 0xffff].
//
// Returns true if rumble state request was honored.
// Calling this before first retro_run() is likely to return false.
pub type SetRumbleStateFn = unsafe extern "C" fn(port: libc::c_uint, effect: RumbleEffect, strength: u16) -> bool;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct RumbleInterface {
    pub set_rumble_state: SetRumbleStateFn,
}

// Notifies libretro that audio data should be written.
pub type AudioCallbackFn = unsafe extern "C" fn();

// True: Audio driver in frontend is active, and callback is
// expected to be called regularily.
// False: Audio driver in frontend is paused or inactive.
// Audio callback will not be called until set_state has been
// called with true.
//
// Initial state is false (inactive).
pub type AudioSetStateCallbackFn = unsafe extern "C" fn(enabled: bool);

#[derive(Clone, Debug)]
#[repr(C)]
pub struct AudioCallback {
    pub callback: AudioCallbackFn,
    pub set_state: AudioSetStateCallbackFn,
}

// Notifies a libretro core of time spent since last invocation
// of retro_run() in microseconds.
//
// It will be called right before retro_run() every frame.
// The frontend can tamper with timing to support cases like
// fast-forward, slow-motion and framestepping.
//
// In those scenarios the reference frame time value will be used.
pub type Usec = i64;
pub type FrameTimeCallbackFn = unsafe extern "C" fn(usec: Usec);

#[derive(Clone, Debug)]
#[repr(C)]
pub struct FrameTimeCallback {
    pub callback: FrameTimeCallbackFn,

    // Represents the time of one frame. It is computed as
    // 1000000 / fps, but the implementation will resolve the
    // rounding to ensure that framestepping, etc is exact.
    pub reference: Usec,
}

// Pass this to retro_video_refresh_t if rendering to hardware.
// Passing NULL to retro_video_refresh_t is still a frame dupe as normal.
pub const HW_FRAME_BUFFER_VALID: *const libc::c_void = -1 as libc::intptr_t as usize as *const libc::c_void;

// Invalidates the current HW context.
// Any GL state is lost, and must not be deinitialized explicitly.
// If explicit deinitialization is desired by the libretro core,
// it should implement context_destroy callback.
// If called, all GPU resources must be reinitialized.
// Usually called when frontend reinits video driver.
// Also called first time video driver is initialized,
// allowing libretro core to initialize resources.
pub type HwContextResetFn = unsafe extern "C" fn();

// Gets current framebuffer which is to be rendered to.
// Could change every frame potentially.
pub type HwGetCurrentFramebufferFn = unsafe extern "C" fn() -> libc::uintptr_t;

// Get a symbol from HW context.
pub type HwGetProcAddressFn = unsafe extern "C" fn(sym: *const libc::c_char) -> ProcAddressFn;

define_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    #[repr(C)]
    pub enum HwContextType {
        None = 0,

        // OpenGL 2.x. Driver can choose to use latest compatibility context.
        OpenGL = 1,
        // OpenGL ES 2.0.
        OpenGLES2 = 2,
        // Modern desktop core GL context. Use version_major/
        // version_minor fields to set GL version.
        OpenGLCore = 3,
        // OpenGL ES 3.0
        OpenGLES3 = 4,
        // OpenGL ES 3.1+. Set version_major/version_minor. For GLES2 and GLES3,
        // use the corresponding enums directly.
        OpenGLESVersion = 5,

        // Vulkan, see ENVIRONMENT_GET_HW_RENDER_INTERFACE.
        Vulkan = 6,
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct HwRenderCallback {
    // Which API to use. Set by libretro core. HwContextType
    pub context_type: libc::c_uint,

    // Called when a context has been created or when it has been reset.
    // An OpenGL context is only valid after context_reset() has been called.
    //
    // When context_reset is called, OpenGL resources in the libretro
    // implementation are guaranteed to be invalid.
    //
    // It is possible that context_reset is called multiple times during an
    // application lifecycle.
    // If context_reset is called without any notification (context_destroy),
    // the OpenGL context was lost and resources should just be recreated
    // without any attempt to "free" old resources.
    pub context_reset: HwContextResetFn,

    // Set by frontend.
    // TODO: This is rather obsolete. The frontend should not
    // be providing preallocated framebuffers.
    pub get_current_framebuffer: HwGetCurrentFramebufferFn,

    // Set by frontend.
    pub get_proc_address: HwGetProcAddressFn,

    // Set if render buffers should have depth component attached.
    // TODO: Obsolete.
    pub depth: bool,

    // Set if stencil buffers should be attached.
    // TODO: Obsolete.
    pub stencil: bool,

    // If depth and stencil are true, a packed 24/8 buffer will be added.
    // Only attaching stencil is invalid and will be ignored. */
    //
    // Use conventional bottom-left origin convention. If false,
    // standard libretro top-left origin semantics are used.
    // TODO: Move to GL specific interface.
    pub bottom_left_origin: bool,

    // Major version number for core GL context or GLES 3.1+.
    pub version_major: libc::c_uint,

    // Minor version number for core GL context or GLES 3.1+.
    pub version_minor: libc::c_uint,

    // If this is true, the frontend will go very far to avoid
    // resetting context in scenarios like toggling fullscreen, etc.
    //
    // The reset callback might still be called in extreme situations
    // such as if the context is lost beyond recovery.
    //
    // For optimal stability, set this to false, and allow context to be
    // reset at any time.
    pub cache_context: bool,

    // A callback to be called before the context is destroyed in a
    // controlled way by the frontend.
    //
    // OpenGL resources can be deinitialized cleanly at this step.
    // context_destroy can be set to NULL, in which resources will
    // just be destroyed without any notification.
    //
    // Even when context_destroy is non-NULL, it is possible that
    // context_reset is called without any destroy notification.
    // This happens if context is lost by external factors (such as
    // notified by GL_ARB_robustness).
    //
    // In this case, the context is assumed to be already dead,
    // and the libretro implementation must not try to free any OpenGL
    // resources in the subsequent context_reset.
    pub context_destroy: HwContextResetFn,

    // Creates a debug context.
    pub debug_context: bool,
}

// Callback type passed in ENVIRONMENT_SET_KEYBOARD_CALLBACK.
// Called by the frontend in response to keyboard events.
// down is set if the key is being pressed, or false if it is being released.
// keycode is the RETROK value of the char.
// character is the text character of the pressed key. (UTF-32).
// key_modifiers is a set of RETROKMOD values or'ed together.
//
// The pressed/keycode state can be indepedent of the character.
// It is also possible that multiple characters are generated from a
// single keypress.
// Keycode events should be treated separately from character events.
// However, when possible, the frontend should try to synchronize these.
// If only a character is posted, keycode should be RETROK_UNKNOWN.
//
// Similarily if only a keycode event is generated with no corresponding
// character, character should be 0.
pub type KeyboardEventFn =
    unsafe extern "C" fn(down: bool, keycode: libc::c_uint, character: u32, key_modifiers: u16);

#[derive(Clone, Debug)]
#[repr(C)]
pub struct KeyboardCallback {
    pub callback: KeyboardEventFn,
}

// Callbacks for ENVIRONMENT_SET_DISK_CONTROL_INTERFACE.
// Should be set for implementations which can swap out multiple disk
// images in runtime.
//
// If the implementation can do this automatically, it should strive to do so.
// However, there are cases where the user must manually do so.
//
// Overview: To swap a disk image, eject the disk image with
// set_eject_state(true).
// Set the disk index with set_image_index(index). Insert the disk again
// with set_eject_state(false).

// If ejected is true, "ejects" the virtual disk tray.
// When ejected, the disk image index can be set.
pub type SetEjectStateFn = unsafe extern "C" fn(ejected: bool) -> bool;

// Gets current eject state. The initial state is 'not ejected'.
pub type GetEjectStateFn = unsafe extern "C" fn() -> bool;

// Gets current disk index. First disk is index 0.
// If return value is >= get_num_images(), no disk is currently inserted.
pub type GetImageIndexFn = unsafe extern "C" fn() -> libc::c_uint;

// Sets image index. Can only be called when disk is ejected.
// The implementation supports setting "no disk" by using an
// index >= get_num_images().
pub type SetImageIndexFn = unsafe extern "C" fn(index: libc::c_uint) -> bool;

// Gets total number of images which are available to use.
pub type GetNumImagesFn = unsafe extern "C" fn() -> libc::c_uint;

// Replaces the disk image associated with index.
// Arguments to pass in info have same requirements as retro_load_game().
// Virtual disk tray must be ejected when calling this.
//
// Replacing a disk image with info = NULL will remove the disk image
// from the internal list.
// As a result, calls to get_image_index() can change.
//
// E.g. replace_image_index(1, NULL), and previous get_image_index()
// returned 4 before.
// Index 1 will be removed, and the new index is 3.
pub type ReplaceImageIndexFn = unsafe extern "C" fn(index: libc::c_uint, info: *const GameInfo) -> bool;

// Adds a new valid index (get_num_images()) to the internal disk list.
// This will increment subsequent return values from get_num_images() by 1.
// This image index cannot be used until a disk image has been set
// with replace_image_index.
pub type AddImageIndexFn = unsafe extern "C" fn() -> bool;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct DiskControlCallback {
    pub set_eject_state: SetEjectStateFn,
    pub get_eject_state: GetEjectStateFn,

    pub get_image_index: GetImageIndexFn,
    pub set_image_index: SetImageIndexFn,
    pub get_num_images: GetNumImagesFn,

    pub replace_image_index: ReplaceImageIndexFn,
    pub add_image_index: AddImageIndexFn,
}

define_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    #[repr(C)]
    pub enum PixelFormat {
        // ARGB1555, native endian.
        // Alpha bit has to be set to 0.
        // This pixel format is default for compatibility concerns only.
        // If a 15/16-bit pixel format is desired, consider using RGB565.
        ARGB1555 = 0,

        // ARGB8888, native endian.
        // Alpha bits are ignored.
        ARGB8888 = 1,

        // RGB565, native endian.
        // This pixel format is the recommended format to use if a 15/16-bit
        // format is desired as it is the pixel format that is typically
        // available on a wide range of low-power devices.
        //
        // It is also natively supported in APIs like OpenGL ES.
        RGB565 = 2,
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Message {
    // Message to be displayed.
    pub msg: *const libc::c_char,

    // Duration in frames of message.
    pub frames: libc::c_uint,
}

// Describes how the libretro implementation maps a libretro input bind
// to its internal input system through a human readable string.
// This string can be used to better let a user configure input.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct InputDescriptor {
    // Associates given parameters with a description.
    pub port: libc::c_uint,
    pub device: libc::c_uint,
    pub index: libc::c_uint,
    pub id: libc::c_uint,

    // Human readable description for parameters.
    // The pointer must remain valid until
    // retro_unload_game() is called.
    pub description: *const libc::c_char,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct SystemInfo {
    // All pointers are owned by libretro implementation, and pointers must
    // remain valid until retro_deinit() is called. */
    //
    // Descriptive name of library. Should not contain any version numbers, etc.
    pub library_name: *const libc::c_char,

    // Descriptive version of core.
    pub library_version: *const libc::c_char,

    // A string listing probably content extensions the core will be able to load,
    // separated with pipe. I.e. "bin|rom|iso". Typically used for a GUI to filter out
    // extensions.
    pub valid_extensions: *const libc::c_char,

    // If true, retro_load_game() is guaranteed to provide a valid pathname
    // in retro_game_info::path.
    // ::data and ::size are both invalid.
    //
    // If false, ::data and ::size are guaranteed to be valid, but ::path
    // might not be valid.
    //
    // This is typically set to true for libretro implementations that must
    // load from file.
    // Implementations should strive for setting this to false, as it allows
    // the frontend to perform patching, etc.
    pub need_fullpath: bool,

    // If true, the frontend is not allowed to extract any archives before
    // loading the real content.
    // Necessary for certain libretro implementations that load games
    // from zipped archives.
    pub block_extract: bool,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct GameGeometry {
    // Nominal video width of game.
    pub base_width: libc::c_uint,

    // Nominal video height of game.
    pub base_height: libc::c_uint,

    // Maximum possible width of game.
    pub max_width: libc::c_uint,

    // Maximum possible height of game.
    pub max_height: libc::c_uint,

    // Nominal aspect ratio of game. If aspect_ratio is <= 0.0, an aspect ratio of
    // base_width / base_height is assumed. A frontend could override this setting, if
    // desired.
    pub aspect_ratio: f32,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct SystemTiming {
    // FPS of video content.
    pub fps: f64,

    // Sampling rate of audio.
    pub sample_rate: f64,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct SystemAvInfo {
    pub geometry: GameGeometry,
    pub timing: SystemTiming,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Variable {
    // Variable to query in ENVIRONMENT_GET_VARIABLE.
    // If NULL, obtains the complete environment string if more
    // complex parsing is necessary.
    //
    // The environment string is formatted as key-value pairs
    // delimited by semicolons as so:
    //   "key1=value1;key2=value2;..."
    //
    pub key: *const libc::c_char,

    // Value to be obtained. If key does not exist, it is set to NULL.
    pub value: *const libc::c_char,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct GameInfo {
    // Path to game, UTF-8 encoded. Usually used as a reference. May be NULL if rom
    // was loaded from stdin or similar. retro_system_info::need_fullpath guaranteed
    // that this path is valid.
    pub path: *const libc::c_char,

    // Memory buffer of loaded game. Will be NULL if need_fullpath was set.
    pub data: *const libc::c_void,

    // Size of memory buffer.
    pub size: libc::size_t,

    // String of implementation specific meta-data.
    pub meta: *const libc::c_char,
}

// The core will write to the buffer provided by retro_framebuffer::data.
pub const MEMORY_ACCESS_WRITE: libc::c_uint = (1 << 0);

// The core will read from retro_framebuffer::data.
pub const MEMORY_ACCESS_READ: libc::c_uint = (1 << 1);

// The memory in data is cached.
// If not cached, random writes and/or reading from the buffer is expected to be very slow.
pub const MEMORY_TYPE_CACHED: libc::c_uint = (1 << 0);

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Framebuffer {
    // The framebuffer which the core can render into. Set by frontend in
    // GET_CURRENT_SOFTWARE_FRAMEBUFFER. The initial contents of data are unspecified.
    pub data: *mut libc::c_void,

    // The framebuffer width used by the core. Set by core.
    pub width: libc::c_uint,

    // The framebuffer height used by the core. Set by core.
    pub height: libc::c_uint,

    // The number of bytes between the beginning of a scanline, and beginning of the
    // next scanline. Set by frontend in GET_CURRENT_SOFTWARE_FRAMEBUFFER.
    pub pitch: libc::size_t,

    // The pixel format the core must use to render into data. This format could
    // differ from the format used in SET_PIXEL_FORMAT. Set by frontend in
    // GET_CURRENT_SOFTWARE_FRAMEBUFFER. A value from enum PixelFormat.
    pub format: libc::c_uint,

    // How the core will access the memory in the framebuffer. MEMORY_ACCESS_*
    // flags. Set by core.
    pub access_flags: libc::c_uint,

    // Flags telling core how the memory has been mapped. MEMORY_TYPE_* flags.
    // Set by frontend in GET_CURRENT_SOFTWARE_FRAMEBUFFER.
    pub memory_flags: libc::c_uint,
}

// Callbacks

// Environment callback. Gives implementations a way of performing
// uncommon tasks. Extensible.
pub type EnvironmentFn = unsafe extern "C" fn(cmd: libc::c_uint, data: *mut libc::c_void) -> bool;

// Render a frame. Pixel format is 15-bit 0RGB1555 native endian
// unless changed (see ENVIRONMENT_SET_PIXEL_FORMAT).
//
// Width and height specify dimensions of buffer.
// Pitch specifices length in bytes between two lines in buffer.
//
// For performance reasons, it is highly recommended to have a frame
// that is packed in memory, i.e. pitch == width * byte_per_pixel.
// Certain graphic APIs, such as OpenGL ES, do not like textures
// that are not packed in memory.
pub type VideoRefreshFn =
    unsafe extern "C" fn(data: *const libc::c_void, width: libc::c_uint, height: libc::c_uint, pitch: libc::size_t);

// Renders a single audio frame. Should only be used if implementation
// generates a single sample at a time.
// Format is signed 16-bit native endian.
pub type AudioSampleFn = unsafe extern "C" fn(left: i16, right: i16);

// Renders multiple audio frames in one go.
//
// One frame is defined as a sample of left and right channels, interleaved.
// I.e. int16_t buf[4] = { l, r, l, r }; would be 2 frames.
// Only one of the audio callbacks must ever be used.
pub type AudioSampleBatchFn = unsafe extern "C" fn(data: *const i16, frames: libc::size_t) -> libc::size_t;

// Polls input.
pub type InputPollFn = unsafe extern "C" fn();

// Queries for input for player 'port'. device will be masked with
// DEVICE_MASK.
//
// Specialization of devices such as DEVICE_JOYPAD_MULTITAP that
// have been set with retro_set_controller_port_device()
// will still use the higher level DEVICE_JOYPAD to request input.
pub type InputStateFn =
    unsafe extern "C" fn(port: libc::c_uint, device: libc::c_uint, index: libc::c_uint, id: libc::c_uint) -> i16;

#[derive(Clone, Debug)]
pub struct CoreAPI {
    // Sets callbacks. retro_set_environment() is guaranteed to be called
    // before retro_init().
    //
    // The rest of the set_* functions are guaranteed to have been called
    // before the first call to retro_run() is made.
    retro_set_environment: unsafe extern "C" fn(callback: EnvironmentFn),
    retro_set_video_refresh: unsafe extern "C" fn(callback: VideoRefreshFn),
    retro_set_audio_sample: unsafe extern "C" fn(callback: AudioSampleFn),
    retro_set_audio_sample_batch: unsafe extern "C" fn(callback: AudioSampleBatchFn),
    retro_set_input_poll: unsafe extern "C" fn(callback: InputPollFn),
    retro_set_input_state: unsafe extern "C" fn(callback: InputStateFn),

    // Library global initialization/deinitialization.
    retro_init: unsafe extern "C" fn(),
    retro_deinit: unsafe extern "C" fn(),

    // Must return API_VERSION. Used to validate ABI compatibility
    // when the API is revised.
    retro_api_version: unsafe extern "C" fn() -> libc::c_uint,

    // Gets statically known system info. Pointers provided in * info
    // must be statically allocated.
    // Can be called at any time, even before retro_init().
    retro_get_system_info: unsafe extern "C" fn(info: *mut SystemInfo),

    // Gets information about system audio/video timings and geometry.
    // Can be called only after retro_load_game() has successfully completed.
    // NOTE: The implementation of this function might not initialize every
    // variable if needed.
    // E.g. geom.aspect_ratio might not be initialized if core doesn't
    // desire a particular aspect ratio.
    retro_get_system_av_info: unsafe extern "C" fn(info: *mut SystemAvInfo),

    // Sets device to be used for player 'port'.
    // By default, DEVICE_JOYPAD is assumed to be plugged into all
    // available ports.
    // Setting a particular device type is not a guarantee that libretro cores
    // will only poll input based on that particular device type. It is only a
    // hint to the libretro core when a core cannot automatically detect the
    // appropriate input device type on its own. It is also relevant when a
    // core can change its behavior depending on device type.
    retro_set_controller_port_device: unsafe extern "C" fn(port: libc::c_uint, device: libc::c_uint),

    // Resets the current game.
    retro_reset: unsafe extern "C" fn(),

    // Runs the game for one video frame.
    // During retro_run(), input_poll callback must be called at least once.
    //
    // If a frame is not rendered for reasons where a game "dropped" a frame,
    // this still counts as a frame, and retro_run() should explicitly dupe
    // a frame if GET_CAN_DUPE returns true.
    // In this case, the video callback can take a NULL argument for data.
    retro_run: unsafe extern "C" fn(),

    // Returns the amount of data the implementation requires to serialize
    // internal state (save states).
    //
    // Between calls to retro_load_game() and retro_unload_game(), the
    // returned size is never allowed to be larger than a previous returned
    // value, to ensure that the frontend can allocate a save state buffer once.
    retro_serialize_size: unsafe extern "C" fn() -> libc::size_t,

    // Serializes internal state. If failed, or size is lower than
    // retro_serialize_size(), it should return false, true otherwise.
    retro_serialize: unsafe extern "C" fn(data: *mut libc::c_void, size: libc::size_t),

    retro_unserialize: unsafe extern "C" fn(data: *const libc::c_void, size: libc::size_t) -> bool,
    retro_cheat_reset: unsafe extern "C" fn(),
    retro_cheat_set: unsafe extern "C" fn(index: libc::c_uint, enabled: bool, code: *const libc::c_char),

    // Loads a game.
    retro_load_game: unsafe extern "C" fn(game: *const GameInfo) -> bool,

    // Loads a "special" kind of game. Should not be used,
    // except in extreme cases.
    retro_load_game_special: unsafe extern "C" fn(game_type: libc::c_uint, info: *const GameInfo, num_info: libc::size_t) -> bool,

    // Unloads a currently loaded game.
    retro_unload_game: unsafe extern "C" fn(),

    // Gets region of game.
    retro_get_region: unsafe extern "C" fn() -> libc::c_uint,

    // Gets region of memory.
    retro_get_memory_data: unsafe extern "C" fn(id: libc::c_uint) -> *mut libc::c_void,
    retro_get_memory_size: unsafe extern "C" fn(id: libc::c_uint) -> libc::size_t,
}
