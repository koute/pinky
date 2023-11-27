pub use softfloat::F32;

#[cfg(feature = "softfloat")]
pub use softfloat::F64;

#[inline]
pub const fn u8_to_f32( value: u8 ) -> F32 {
    F32::from_u32( value as u32 )
}

#[inline]
pub const fn f32_to_u32( value: F32 ) -> u32 {
    value.to_u32()
}
