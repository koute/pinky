pub type F32 = f32;
pub type F64 = f64;

#[macro_export]
macro_rules! f32 {
    ($value:expr) => { $value }
}

#[macro_export]
macro_rules! f64 {
    ($value:expr) => { $value }
}

pub fn u8_to_f32( value: u8 ) -> F32 {
    value as f32
}

pub fn f32_to_u32( value: F32 ) -> u32 {
    value as u32
}
