pub trait AsIndex: Sized {
    fn as_index( self ) -> usize;
}

impl AsIndex for u8 {
    #[inline(always)]
    fn as_index( self ) -> usize {
        self as usize
    }
}

impl AsIndex for u16 {
    #[inline(always)]
    fn as_index( self ) -> usize {
        self as usize
    }
}

impl AsIndex for u32 {
    #[inline(always)]
    fn as_index( self ) -> usize {
        self as usize
    }
}

impl AsIndex for usize {
    #[inline(always)]
    fn as_index( self ) -> usize {
        self
    }
}

impl AsIndex for i8 {
    #[inline(always)]
    fn as_index( self ) -> usize {
        debug_assert!( self >= 0 );
        self as usize
    }
}

impl AsIndex for i16 {
    #[inline(always)]
    fn as_index( self ) -> usize {
        debug_assert!( self >= 0 );
        self as usize
    }
}

impl AsIndex for i32 {
    #[inline(always)]
    fn as_index( self ) -> usize {
        debug_assert!( self >= 0 );
        self as usize
    }
}

pub trait At<T>: AsRef<[T]> + AsMut<[T]> {
    #[inline(always)]
    #[cfg(not(debug_assertions))]
    fn at< I: AsIndex >( &self, index: I ) -> &T {
        unsafe { self.as_ref().get_unchecked( index.as_index() ) }
    }

    #[inline(always)]
    #[cfg(debug_assertions)]
    fn at< I: AsIndex >( &self, index: I ) -> &T {
        &self.as_ref()[ index.as_index() ]
    }

    #[inline(always)]
    #[cfg(not(debug_assertions))]
    fn at_mut< I: AsIndex >( &mut self, index: I ) -> &mut T {
        unsafe { self.as_mut().get_unchecked_mut( index.as_index() ) }
    }

    #[inline(always)]
    #[cfg(debug_assertions)]
    fn at_mut< I: AsIndex >( &mut self, index: I ) -> &mut T {
        &mut self.as_mut()[ index.as_index() ]
    }
}

pub trait PeekPoke<T>: AsRef<[T]> + AsMut<[T]> where T: Copy {
    #[inline(always)]
    #[cfg(not(debug_assertions))]
    fn poke< I: AsIndex >( &mut self, index: I, value: T ) {
        unsafe { *self.as_mut().get_unchecked_mut( index.as_index() ) = value }
    }

    #[inline(always)]
    #[cfg(debug_assertions)]
    fn poke< I: AsIndex >( &mut self, index: I, value: T ) {
        self.as_mut()[ index.as_index() ] = value;
    }

    #[inline(always)]
    #[cfg(not(debug_assertions))]
    fn peek< I: AsIndex >( &self, index: I ) -> T {
        unsafe { *self.as_ref().get_unchecked( index.as_index() ) }
    }

    #[inline(always)]
    #[cfg(debug_assertions)]
    fn peek< I: AsIndex >( &self, index: I ) -> T {
        self.as_ref()[ index.as_index() ]
    }
}

impl<T> At<T> for [T] {}
impl< T: Copy > PeekPoke<T> for [T] {}
