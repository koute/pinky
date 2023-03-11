use core::mem;

#[inline(always)] pub fn is_b0_set( value: u8 ) -> bool { (value & (1 << 0)) != 0 }
#[inline(always)] pub fn is_b1_set( value: u8 ) -> bool { (value & (1 << 1)) != 0 }
#[inline(always)] pub fn is_b2_set( value: u8 ) -> bool { (value & (1 << 2)) != 0 }
#[inline(always)] pub fn is_b3_set( value: u8 ) -> bool { (value & (1 << 3)) != 0 }
#[inline(always)] pub fn is_b4_set( value: u8 ) -> bool { (value & (1 << 4)) != 0 }
#[inline(always)] pub fn is_b5_set( value: u8 ) -> bool { (value & (1 << 5)) != 0 }
#[inline(always)] pub fn is_b6_set( value: u8 ) -> bool { (value & (1 << 6)) != 0 }
#[inline(always)] pub fn is_b7_set( value: u8 ) -> bool { (value & (1 << 7)) != 0 }

#[inline(always)]
pub fn to_bit( bit: u8, value: bool ) -> u8 {
    debug_assert!( bit < 8 );
    let converted: u8 = unsafe { mem::transmute( value ) };
    debug_assert!( converted == 0 || converted == 1 );
    converted << bit
}

#[inline]
pub fn reverse_bits( value: u8 ) -> u8 {
    ((((value as u32).wrapping_mul( 0x0802u32 ) & 0x22110u32) |
      ((value as u32).wrapping_mul( 0x8020u32 ) & 0x88440u32)).wrapping_mul( 0x10101u32 ) >> 16) as u8
}

pub trait BitExtra {
    fn replace_bits( &mut self, mask: Self, replacement: Self );
    fn copy_bits_from( &mut self, mask: Self, replacement: Self );
    fn get_bits( self, mask: Self ) -> Self;
    fn mask_to_shift( mask: Self ) -> u8;
}

macro_rules! impl_bit_extra {
    ($typ:ty) => (
        impl BitExtra for $typ {
            #[inline(always)]
            fn mask_to_shift( mask: Self ) -> u8 {
                use core::mem::size_of;
                debug_assert!( mask != 0 );

                for n_bit in 0..(size_of::< Self >() * 8) as u8 {
                    if mask & (1 << n_bit) != 0 {
                        return n_bit;
                    }
                }

                unsafe {
                    fast_unreachable!();
                }
            }

            #[inline(always)]
            fn replace_bits( &mut self, mask: Self, replacement: Self ) {
                let shift = BitExtra::mask_to_shift( mask );
                *self = (*self & !mask) | ((replacement << shift) & mask);
            }

            #[inline(always)]
            fn copy_bits_from( &mut self, mask: Self, replacement: Self ) {
                *self = (*self & !mask) | (replacement & mask);
            }

            #[inline(always)]
            fn get_bits( self, mask: Self ) -> Self {
                let shift = BitExtra::mask_to_shift( mask );
                (self & mask) >> shift
            }
        }
    )
}

impl_bit_extra!(u8);
impl_bit_extra!(u16);
impl_bit_extra!(u32);
impl_bit_extra!(u64);

pub trait HiLoAccess: Sized {
    type Half;

    #[inline(always)]
    fn set_lo( &mut self, value: Self::Half ) {
        debug_assert_eq!( mem::size_of::< Self >(), mem::size_of::< Self::Half >() * 2 );
        unsafe { *(self as *mut Self as *mut Self::Half) = value };
    }

    #[inline(always)]
    fn set_hi( &mut self, value: Self::Half ) {
        debug_assert_eq!( mem::size_of::< Self >(), mem::size_of::< Self::Half >() * 2 );
        unsafe { *(self as *mut Self as *mut Self::Half).offset( mem::size_of::< u8 >() as isize ) = value };
    }
}

impl HiLoAccess for u16 {
    type Half = u8;
}

#[macro_export]
macro_rules! set_lo {
    ($this: expr, $value: expr) => ({
        let value = $value;
        $this.set_lo( value );
    })
}

#[macro_export]
macro_rules! set_hi {
    ($this: expr, $value: expr) => ({
        let value = $value;
        $this.set_hi( value );
    })
}

#[macro_export]
macro_rules! bit {
    ($nth: expr, $name: ident) => (
        #[inline]
        fn $name( &self ) -> bool {
            (self.0 & (1 << $nth)) != 0
        }
    )
}

#[macro_export]
macro_rules! bit_setter {
    ($nth: expr, $name: ident) => (
        #[inline]
        fn $name( &mut self, value: bool ) {
            self.0 = (self.0 & !(1 << $nth)) | ((value as u8) << $nth);
        }
    )
}

#[test]
fn test_replace_bits() {
    let mut a: u8 = 0b11111111;
    a.replace_bits( 0b10000001, 0b00000000 );
    assert_eq!( a, 0b01111110 );

    a.replace_bits( 0b10000001, 0b10000001 );
    assert_eq!( a, 0b11111111 );

    a.replace_bits( 0b11000000, 0b00000000 );
    assert_eq!( a, 0b00111111 );

    a.replace_bits( 0b11000000, 0b00000011 );
    assert_eq!( a, 0b11111111 );
}

#[test]
fn test_copy_bits_from() {
    let mut a: u16 = 0b00010100_00000000;
    a.copy_bits_from( 0b01111011_11100000, 0b00000100_00000101 );
    assert_eq!( a, 0b00000100_00000000 );
}

#[test]
fn test_get_bits() {
    assert_eq!( 0b11110000u8.get_bits( 0b11110000 ), 0b1111 );
    assert_eq!( 0b11110000u8.get_bits( 0b00111100 ), 0b1100 );
    assert_eq!( 0b11110000u8.get_bits( 0b00001111 ), 0b0000 );
}

#[test]
fn test_set_lo_hi() {
    let mut value = 0u16;
    value.set_lo( 0xFF );
    assert_eq!( value, 0x00FF );

    value = 0;
    value.set_hi( 0xFF );
    assert_eq!( value, 0xFF00 );
}

#[test]
fn test_reverse_bits() {
    assert_eq!( reverse_bits( 0b00000000 ), 0b00000000 );
    assert_eq!( reverse_bits( 0b00000001 ), 0b10000000 );
    assert_eq!( reverse_bits( 0b00000010 ), 0b01000000 );
    assert_eq!( reverse_bits( 0b00000011 ), 0b11000000 );
    assert_eq!( reverse_bits( 0b00000100 ), 0b00100000 );
    assert_eq!( reverse_bits( 0b00000101 ), 0b10100000 );
    assert_eq!( reverse_bits( 0b00000110 ), 0b01100000 );
    assert_eq!( reverse_bits( 0b00000111 ), 0b11100000 );
    assert_eq!( reverse_bits( 0b00001000 ), 0b00010000 );
    assert_eq!( reverse_bits( 0b00001001 ), 0b10010000 );
    assert_eq!( reverse_bits( 0b00001010 ), 0b01010000 );
    assert_eq!( reverse_bits( 0b00001011 ), 0b11010000 );
    assert_eq!( reverse_bits( 0b00001100 ), 0b00110000 );
    assert_eq!( reverse_bits( 0b00001101 ), 0b10110000 );
    assert_eq!( reverse_bits( 0b00001110 ), 0b01110000 );
    assert_eq!( reverse_bits( 0b00001111 ), 0b11110000 );
    assert_eq!( reverse_bits( 0b00010000 ), 0b00001000 );
    assert_eq!( reverse_bits( 0b00010001 ), 0b10001000 );
    assert_eq!( reverse_bits( 0b00010010 ), 0b01001000 );
    assert_eq!( reverse_bits( 0b00010011 ), 0b11001000 );
    assert_eq!( reverse_bits( 0b00010100 ), 0b00101000 );
    assert_eq!( reverse_bits( 0b00010101 ), 0b10101000 );
    assert_eq!( reverse_bits( 0b00010110 ), 0b01101000 );
    assert_eq!( reverse_bits( 0b00010111 ), 0b11101000 );
    assert_eq!( reverse_bits( 0b00011000 ), 0b00011000 );
    assert_eq!( reverse_bits( 0b00011001 ), 0b10011000 );
    assert_eq!( reverse_bits( 0b00011010 ), 0b01011000 );
    assert_eq!( reverse_bits( 0b00011011 ), 0b11011000 );
    assert_eq!( reverse_bits( 0b00011100 ), 0b00111000 );
    assert_eq!( reverse_bits( 0b00011101 ), 0b10111000 );
    assert_eq!( reverse_bits( 0b00011110 ), 0b01111000 );
    assert_eq!( reverse_bits( 0b00011111 ), 0b11111000 );
    assert_eq!( reverse_bits( 0b00100000 ), 0b00000100 );
    assert_eq!( reverse_bits( 0b00100001 ), 0b10000100 );
    assert_eq!( reverse_bits( 0b00100010 ), 0b01000100 );
    assert_eq!( reverse_bits( 0b00100011 ), 0b11000100 );
    assert_eq!( reverse_bits( 0b00100100 ), 0b00100100 );
    assert_eq!( reverse_bits( 0b00100101 ), 0b10100100 );
    assert_eq!( reverse_bits( 0b00100110 ), 0b01100100 );
    assert_eq!( reverse_bits( 0b00100111 ), 0b11100100 );
    assert_eq!( reverse_bits( 0b00101000 ), 0b00010100 );
    assert_eq!( reverse_bits( 0b00101001 ), 0b10010100 );
    assert_eq!( reverse_bits( 0b00101010 ), 0b01010100 );
    assert_eq!( reverse_bits( 0b00101011 ), 0b11010100 );
    assert_eq!( reverse_bits( 0b00101100 ), 0b00110100 );
    assert_eq!( reverse_bits( 0b00101101 ), 0b10110100 );
    assert_eq!( reverse_bits( 0b00101110 ), 0b01110100 );
    assert_eq!( reverse_bits( 0b00101111 ), 0b11110100 );
    assert_eq!( reverse_bits( 0b00110000 ), 0b00001100 );
    assert_eq!( reverse_bits( 0b00110001 ), 0b10001100 );
    assert_eq!( reverse_bits( 0b00110010 ), 0b01001100 );
    assert_eq!( reverse_bits( 0b00110011 ), 0b11001100 );
    assert_eq!( reverse_bits( 0b00110100 ), 0b00101100 );
    assert_eq!( reverse_bits( 0b00110101 ), 0b10101100 );
    assert_eq!( reverse_bits( 0b00110110 ), 0b01101100 );
    assert_eq!( reverse_bits( 0b00110111 ), 0b11101100 );
    assert_eq!( reverse_bits( 0b00111000 ), 0b00011100 );
    assert_eq!( reverse_bits( 0b00111001 ), 0b10011100 );
    assert_eq!( reverse_bits( 0b00111010 ), 0b01011100 );
    assert_eq!( reverse_bits( 0b00111011 ), 0b11011100 );
    assert_eq!( reverse_bits( 0b00111100 ), 0b00111100 );
    assert_eq!( reverse_bits( 0b00111101 ), 0b10111100 );
    assert_eq!( reverse_bits( 0b00111110 ), 0b01111100 );
    assert_eq!( reverse_bits( 0b00111111 ), 0b11111100 );
    assert_eq!( reverse_bits( 0b01000000 ), 0b00000010 );
    assert_eq!( reverse_bits( 0b01000001 ), 0b10000010 );
    assert_eq!( reverse_bits( 0b01000010 ), 0b01000010 );
    assert_eq!( reverse_bits( 0b01000011 ), 0b11000010 );
    assert_eq!( reverse_bits( 0b01000100 ), 0b00100010 );
    assert_eq!( reverse_bits( 0b01000101 ), 0b10100010 );
    assert_eq!( reverse_bits( 0b01000110 ), 0b01100010 );
    assert_eq!( reverse_bits( 0b01000111 ), 0b11100010 );
    assert_eq!( reverse_bits( 0b01001000 ), 0b00010010 );
    assert_eq!( reverse_bits( 0b01001001 ), 0b10010010 );
    assert_eq!( reverse_bits( 0b01001010 ), 0b01010010 );
    assert_eq!( reverse_bits( 0b01001011 ), 0b11010010 );
    assert_eq!( reverse_bits( 0b01001100 ), 0b00110010 );
    assert_eq!( reverse_bits( 0b01001101 ), 0b10110010 );
    assert_eq!( reverse_bits( 0b01001110 ), 0b01110010 );
    assert_eq!( reverse_bits( 0b01001111 ), 0b11110010 );
    assert_eq!( reverse_bits( 0b01010000 ), 0b00001010 );
    assert_eq!( reverse_bits( 0b01010001 ), 0b10001010 );
    assert_eq!( reverse_bits( 0b01010010 ), 0b01001010 );
    assert_eq!( reverse_bits( 0b01010011 ), 0b11001010 );
    assert_eq!( reverse_bits( 0b01010100 ), 0b00101010 );
    assert_eq!( reverse_bits( 0b01010101 ), 0b10101010 );
    assert_eq!( reverse_bits( 0b01010110 ), 0b01101010 );
    assert_eq!( reverse_bits( 0b01010111 ), 0b11101010 );
    assert_eq!( reverse_bits( 0b01011000 ), 0b00011010 );
    assert_eq!( reverse_bits( 0b01011001 ), 0b10011010 );
    assert_eq!( reverse_bits( 0b01011010 ), 0b01011010 );
    assert_eq!( reverse_bits( 0b01011011 ), 0b11011010 );
    assert_eq!( reverse_bits( 0b01011100 ), 0b00111010 );
    assert_eq!( reverse_bits( 0b01011101 ), 0b10111010 );
    assert_eq!( reverse_bits( 0b01011110 ), 0b01111010 );
    assert_eq!( reverse_bits( 0b01011111 ), 0b11111010 );
    assert_eq!( reverse_bits( 0b01100000 ), 0b00000110 );
    assert_eq!( reverse_bits( 0b01100001 ), 0b10000110 );
    assert_eq!( reverse_bits( 0b01100010 ), 0b01000110 );
    assert_eq!( reverse_bits( 0b01100011 ), 0b11000110 );
    assert_eq!( reverse_bits( 0b01100100 ), 0b00100110 );
    assert_eq!( reverse_bits( 0b01100101 ), 0b10100110 );
    assert_eq!( reverse_bits( 0b01100110 ), 0b01100110 );
    assert_eq!( reverse_bits( 0b01100111 ), 0b11100110 );
    assert_eq!( reverse_bits( 0b01101000 ), 0b00010110 );
    assert_eq!( reverse_bits( 0b01101001 ), 0b10010110 );
    assert_eq!( reverse_bits( 0b01101010 ), 0b01010110 );
    assert_eq!( reverse_bits( 0b01101011 ), 0b11010110 );
    assert_eq!( reverse_bits( 0b01101100 ), 0b00110110 );
    assert_eq!( reverse_bits( 0b01101101 ), 0b10110110 );
    assert_eq!( reverse_bits( 0b01101110 ), 0b01110110 );
    assert_eq!( reverse_bits( 0b01101111 ), 0b11110110 );
    assert_eq!( reverse_bits( 0b01110000 ), 0b00001110 );
    assert_eq!( reverse_bits( 0b01110001 ), 0b10001110 );
    assert_eq!( reverse_bits( 0b01110010 ), 0b01001110 );
    assert_eq!( reverse_bits( 0b01110011 ), 0b11001110 );
    assert_eq!( reverse_bits( 0b01110100 ), 0b00101110 );
    assert_eq!( reverse_bits( 0b01110101 ), 0b10101110 );
    assert_eq!( reverse_bits( 0b01110110 ), 0b01101110 );
    assert_eq!( reverse_bits( 0b01110111 ), 0b11101110 );
    assert_eq!( reverse_bits( 0b01111000 ), 0b00011110 );
    assert_eq!( reverse_bits( 0b01111001 ), 0b10011110 );
    assert_eq!( reverse_bits( 0b01111010 ), 0b01011110 );
    assert_eq!( reverse_bits( 0b01111011 ), 0b11011110 );
    assert_eq!( reverse_bits( 0b01111100 ), 0b00111110 );
    assert_eq!( reverse_bits( 0b01111101 ), 0b10111110 );
    assert_eq!( reverse_bits( 0b01111110 ), 0b01111110 );
    assert_eq!( reverse_bits( 0b01111111 ), 0b11111110 );
    assert_eq!( reverse_bits( 0b10000000 ), 0b00000001 );
    assert_eq!( reverse_bits( 0b10000001 ), 0b10000001 );
    assert_eq!( reverse_bits( 0b10000010 ), 0b01000001 );
    assert_eq!( reverse_bits( 0b10000011 ), 0b11000001 );
    assert_eq!( reverse_bits( 0b10000100 ), 0b00100001 );
    assert_eq!( reverse_bits( 0b10000101 ), 0b10100001 );
    assert_eq!( reverse_bits( 0b10000110 ), 0b01100001 );
    assert_eq!( reverse_bits( 0b10000111 ), 0b11100001 );
    assert_eq!( reverse_bits( 0b10001000 ), 0b00010001 );
    assert_eq!( reverse_bits( 0b10001001 ), 0b10010001 );
    assert_eq!( reverse_bits( 0b10001010 ), 0b01010001 );
    assert_eq!( reverse_bits( 0b10001011 ), 0b11010001 );
    assert_eq!( reverse_bits( 0b10001100 ), 0b00110001 );
    assert_eq!( reverse_bits( 0b10001101 ), 0b10110001 );
    assert_eq!( reverse_bits( 0b10001110 ), 0b01110001 );
    assert_eq!( reverse_bits( 0b10001111 ), 0b11110001 );
    assert_eq!( reverse_bits( 0b10010000 ), 0b00001001 );
    assert_eq!( reverse_bits( 0b10010001 ), 0b10001001 );
    assert_eq!( reverse_bits( 0b10010010 ), 0b01001001 );
    assert_eq!( reverse_bits( 0b10010011 ), 0b11001001 );
    assert_eq!( reverse_bits( 0b10010100 ), 0b00101001 );
    assert_eq!( reverse_bits( 0b10010101 ), 0b10101001 );
    assert_eq!( reverse_bits( 0b10010110 ), 0b01101001 );
    assert_eq!( reverse_bits( 0b10010111 ), 0b11101001 );
    assert_eq!( reverse_bits( 0b10011000 ), 0b00011001 );
    assert_eq!( reverse_bits( 0b10011001 ), 0b10011001 );
    assert_eq!( reverse_bits( 0b10011010 ), 0b01011001 );
    assert_eq!( reverse_bits( 0b10011011 ), 0b11011001 );
    assert_eq!( reverse_bits( 0b10011100 ), 0b00111001 );
    assert_eq!( reverse_bits( 0b10011101 ), 0b10111001 );
    assert_eq!( reverse_bits( 0b10011110 ), 0b01111001 );
    assert_eq!( reverse_bits( 0b10011111 ), 0b11111001 );
    assert_eq!( reverse_bits( 0b10100000 ), 0b00000101 );
    assert_eq!( reverse_bits( 0b10100001 ), 0b10000101 );
    assert_eq!( reverse_bits( 0b10100010 ), 0b01000101 );
    assert_eq!( reverse_bits( 0b10100011 ), 0b11000101 );
    assert_eq!( reverse_bits( 0b10100100 ), 0b00100101 );
    assert_eq!( reverse_bits( 0b10100101 ), 0b10100101 );
    assert_eq!( reverse_bits( 0b10100110 ), 0b01100101 );
    assert_eq!( reverse_bits( 0b10100111 ), 0b11100101 );
    assert_eq!( reverse_bits( 0b10101000 ), 0b00010101 );
    assert_eq!( reverse_bits( 0b10101001 ), 0b10010101 );
    assert_eq!( reverse_bits( 0b10101010 ), 0b01010101 );
    assert_eq!( reverse_bits( 0b10101011 ), 0b11010101 );
    assert_eq!( reverse_bits( 0b10101100 ), 0b00110101 );
    assert_eq!( reverse_bits( 0b10101101 ), 0b10110101 );
    assert_eq!( reverse_bits( 0b10101110 ), 0b01110101 );
    assert_eq!( reverse_bits( 0b10101111 ), 0b11110101 );
    assert_eq!( reverse_bits( 0b10110000 ), 0b00001101 );
    assert_eq!( reverse_bits( 0b10110001 ), 0b10001101 );
    assert_eq!( reverse_bits( 0b10110010 ), 0b01001101 );
    assert_eq!( reverse_bits( 0b10110011 ), 0b11001101 );
    assert_eq!( reverse_bits( 0b10110100 ), 0b00101101 );
    assert_eq!( reverse_bits( 0b10110101 ), 0b10101101 );
    assert_eq!( reverse_bits( 0b10110110 ), 0b01101101 );
    assert_eq!( reverse_bits( 0b10110111 ), 0b11101101 );
    assert_eq!( reverse_bits( 0b10111000 ), 0b00011101 );
    assert_eq!( reverse_bits( 0b10111001 ), 0b10011101 );
    assert_eq!( reverse_bits( 0b10111010 ), 0b01011101 );
    assert_eq!( reverse_bits( 0b10111011 ), 0b11011101 );
    assert_eq!( reverse_bits( 0b10111100 ), 0b00111101 );
    assert_eq!( reverse_bits( 0b10111101 ), 0b10111101 );
    assert_eq!( reverse_bits( 0b10111110 ), 0b01111101 );
    assert_eq!( reverse_bits( 0b10111111 ), 0b11111101 );
    assert_eq!( reverse_bits( 0b11000000 ), 0b00000011 );
    assert_eq!( reverse_bits( 0b11000001 ), 0b10000011 );
    assert_eq!( reverse_bits( 0b11000010 ), 0b01000011 );
    assert_eq!( reverse_bits( 0b11000011 ), 0b11000011 );
    assert_eq!( reverse_bits( 0b11000100 ), 0b00100011 );
    assert_eq!( reverse_bits( 0b11000101 ), 0b10100011 );
    assert_eq!( reverse_bits( 0b11000110 ), 0b01100011 );
    assert_eq!( reverse_bits( 0b11000111 ), 0b11100011 );
    assert_eq!( reverse_bits( 0b11001000 ), 0b00010011 );
    assert_eq!( reverse_bits( 0b11001001 ), 0b10010011 );
    assert_eq!( reverse_bits( 0b11001010 ), 0b01010011 );
    assert_eq!( reverse_bits( 0b11001011 ), 0b11010011 );
    assert_eq!( reverse_bits( 0b11001100 ), 0b00110011 );
    assert_eq!( reverse_bits( 0b11001101 ), 0b10110011 );
    assert_eq!( reverse_bits( 0b11001110 ), 0b01110011 );
    assert_eq!( reverse_bits( 0b11001111 ), 0b11110011 );
    assert_eq!( reverse_bits( 0b11010000 ), 0b00001011 );
    assert_eq!( reverse_bits( 0b11010001 ), 0b10001011 );
    assert_eq!( reverse_bits( 0b11010010 ), 0b01001011 );
    assert_eq!( reverse_bits( 0b11010011 ), 0b11001011 );
    assert_eq!( reverse_bits( 0b11010100 ), 0b00101011 );
    assert_eq!( reverse_bits( 0b11010101 ), 0b10101011 );
    assert_eq!( reverse_bits( 0b11010110 ), 0b01101011 );
    assert_eq!( reverse_bits( 0b11010111 ), 0b11101011 );
    assert_eq!( reverse_bits( 0b11011000 ), 0b00011011 );
    assert_eq!( reverse_bits( 0b11011001 ), 0b10011011 );
    assert_eq!( reverse_bits( 0b11011010 ), 0b01011011 );
    assert_eq!( reverse_bits( 0b11011011 ), 0b11011011 );
    assert_eq!( reverse_bits( 0b11011100 ), 0b00111011 );
    assert_eq!( reverse_bits( 0b11011101 ), 0b10111011 );
    assert_eq!( reverse_bits( 0b11011110 ), 0b01111011 );
    assert_eq!( reverse_bits( 0b11011111 ), 0b11111011 );
    assert_eq!( reverse_bits( 0b11100000 ), 0b00000111 );
    assert_eq!( reverse_bits( 0b11100001 ), 0b10000111 );
    assert_eq!( reverse_bits( 0b11100010 ), 0b01000111 );
    assert_eq!( reverse_bits( 0b11100011 ), 0b11000111 );
    assert_eq!( reverse_bits( 0b11100100 ), 0b00100111 );
    assert_eq!( reverse_bits( 0b11100101 ), 0b10100111 );
    assert_eq!( reverse_bits( 0b11100110 ), 0b01100111 );
    assert_eq!( reverse_bits( 0b11100111 ), 0b11100111 );
    assert_eq!( reverse_bits( 0b11101000 ), 0b00010111 );
    assert_eq!( reverse_bits( 0b11101001 ), 0b10010111 );
    assert_eq!( reverse_bits( 0b11101010 ), 0b01010111 );
    assert_eq!( reverse_bits( 0b11101011 ), 0b11010111 );
    assert_eq!( reverse_bits( 0b11101100 ), 0b00110111 );
    assert_eq!( reverse_bits( 0b11101101 ), 0b10110111 );
    assert_eq!( reverse_bits( 0b11101110 ), 0b01110111 );
    assert_eq!( reverse_bits( 0b11101111 ), 0b11110111 );
    assert_eq!( reverse_bits( 0b11110000 ), 0b00001111 );
    assert_eq!( reverse_bits( 0b11110001 ), 0b10001111 );
    assert_eq!( reverse_bits( 0b11110010 ), 0b01001111 );
    assert_eq!( reverse_bits( 0b11110011 ), 0b11001111 );
    assert_eq!( reverse_bits( 0b11110100 ), 0b00101111 );
    assert_eq!( reverse_bits( 0b11110101 ), 0b10101111 );
    assert_eq!( reverse_bits( 0b11110110 ), 0b01101111 );
    assert_eq!( reverse_bits( 0b11110111 ), 0b11101111 );
    assert_eq!( reverse_bits( 0b11111000 ), 0b00011111 );
    assert_eq!( reverse_bits( 0b11111001 ), 0b10011111 );
    assert_eq!( reverse_bits( 0b11111010 ), 0b01011111 );
    assert_eq!( reverse_bits( 0b11111011 ), 0b11011111 );
    assert_eq!( reverse_bits( 0b11111100 ), 0b00111111 );
    assert_eq!( reverse_bits( 0b11111101 ), 0b10111111 );
    assert_eq!( reverse_bits( 0b11111110 ), 0b01111111 );
    assert_eq!( reverse_bits( 0b11111111 ), 0b11111111 );
}
