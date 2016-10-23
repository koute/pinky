use sdl2;
use sdl2::pixels::PixelFormatEnum;

use emumisc;
use emumisc::{PeekPoke, as_bytes};

pub struct ImageBuffer {
    buffer: Vec< u32 >,
    width: u32,
    height: u32
}

pub struct Renderer {
    pub sdl_renderer: sdl2::render::Renderer< 'static >
}

impl_deref!( Renderer, sdl2::render::Renderer< 'static >, sdl_renderer );
impl_as_ref!( Renderer, sdl2::render::Renderer< 'static >, sdl_renderer );

impl Renderer {
    pub fn new( sdl_renderer: sdl2::render::Renderer< 'static > ) -> Renderer {
        Renderer {
            sdl_renderer: sdl_renderer
        }
    }

    pub fn blit( &mut self, texture: &Texture ) {
        self.copy( texture.as_ref(), None, None ).unwrap();
    }
}

newtype!( struct Texture = sdl2::render::Texture );

#[allow(dead_code)]
impl ImageBuffer {
    pub fn empty() -> ImageBuffer {
        ImageBuffer {
            width: 0,
            height: 0,
            buffer: Vec::new()
        }
    }

    pub fn new( width: u32, height: u32 ) -> ImageBuffer {
        let size = (width * height) as usize;
        let mut buffer = Vec::with_capacity( size );
        buffer.resize( size, 0 );

        ImageBuffer {
            width: width,
            height: height,
            buffer: buffer
        }
    }

    pub fn clear( &mut self ) {
        for index in 0..self.buffer.len() {
            self.buffer.poke( index, 0 );
        }
    }

    pub fn width( &self ) -> u32 {
        self.width
    }

    pub fn height( &self ) -> u32 {
        self.height
    }

    pub fn replace< F >( &mut self, width: u32, height: u32, callback: F ) where F: FnOnce( &mut [u32] ) {
        let size = (width * height) as usize;
        self.buffer.clear();
        self.buffer.reserve( size );
        unsafe {
            self.buffer.set_len( size );
        }
        callback( &mut self.buffer[..] );
    }

    pub fn as_bytes( &self ) -> &[u8] {
        emumisc::as_bytes( &self.buffer[..] )
    }

    pub fn stride_in_bytes( &self ) -> usize {
        self.width as usize * 4
    }
}

impl AsMut< [u32] > for ImageBuffer {
    fn as_mut( &mut self ) -> &mut [u32] {
        &mut self.buffer[..]
    }
}

#[allow(dead_code)]
impl Texture {
    pub fn new_streaming< T: AsMut< sdl2::render::Renderer< 'static >>>( renderer: &mut T, width: u32, height: u32 ) -> Texture {
        Texture( renderer.as_mut().create_texture_streaming( PixelFormatEnum::ABGR8888, width, height ).unwrap() )
    }

    pub fn new_static< T: AsMut< sdl2::render::Renderer< 'static >>>( renderer: &mut T, width: u32, height: u32 ) -> Texture {
        Texture( renderer.as_mut().create_texture_static( PixelFormatEnum::ABGR8888, width, height ).unwrap() )
    }

    pub fn update( &mut self, buffer: &ImageBuffer ) {
        assert_eq!( self.query().width, buffer.width() );
        assert_eq!( self.query().height, buffer.height() );
        self.0.update( None, as_bytes( &buffer.buffer[..] ), buffer.width() as usize * 4 ).unwrap();
    }
}
