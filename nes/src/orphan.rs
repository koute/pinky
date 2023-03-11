use core::mem;
use core::ops::{Deref, DerefMut};

// A shim like this is necessary to implement an orphaned instance in Rust.
// It's very important to keep the structure opaque to keep it safe to use.
pub struct Orphan< T >( T );

impl< T > Orphan< T > {
    #[inline]
    pub fn cast( value: &T ) -> &Orphan< T > {
        unsafe {
            mem::transmute( value )
        }
    }

    #[inline]
    pub fn cast_mut( value: &mut T ) -> &mut Orphan< T > {
        unsafe {
            mem::transmute( value )
        }
    }
}

impl< T > Deref for Orphan< T > {
    type Target = T;

    #[inline]
    fn deref( &self ) -> &Self::Target {
        &self.0
    }
}

impl< T > DerefMut for Orphan< T > {
    #[inline]
    fn deref_mut( &mut self ) -> &mut Self::Target {
        &mut self.0
    }
}

impl< T > AsRef< T > for Orphan< T > {
    #[inline]
    fn as_ref( &self ) -> &T {
        &self.0
    }
}

impl< T > AsMut< T > for Orphan< T > {
    #[inline]
    fn as_mut( &mut self ) -> &mut T {
        &mut self.0
    }
}
