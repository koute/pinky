pub trait WrappingExtra {
    fn wrapping_inc( &mut self );
    fn wrapping_dec( &mut self );
}

macro_rules! impl_wrapping_extra {
    ($typ:ty) => (
        impl WrappingExtra for $typ {
            #[inline(always)]
            fn wrapping_inc( &mut self ) {
                *self = self.wrapping_add( 1 );
            }

            #[inline(always)]
            fn wrapping_dec( &mut self ) {
                *self = self.wrapping_sub( 1 );
            }
        }
    )
}

impl_wrapping_extra!(u8);
impl_wrapping_extra!(u16);
impl_wrapping_extra!(u32);
