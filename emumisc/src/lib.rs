#![no_std]

extern crate unreachable;
extern crate alloc;

#[macro_use]
mod misc;

mod bits;
mod indexing;
mod wrapping;
mod memory;

pub use unreachable::unreachable;
pub use bits::{
    BitExtra,
    HiLoAccess,
    is_b0_set,
    is_b1_set,
    is_b2_set,
    is_b3_set,
    is_b4_set,
    is_b5_set,
    is_b6_set,
    is_b7_set,
    to_bit,
    reverse_bits
};

pub use indexing::{At, PeekPoke, AsIndex};
pub use wrapping::WrappingExtra;
pub use memory::{as_bytes, allocate_slice, copy_memory};

#[macro_export]
macro_rules! impl_deref {
    ($container: ty, $field_type: ty, $field: ident) => (
        impl ::core::ops::Deref for $container {
            type Target = $field_type;

            #[inline(always)]
            fn deref( &self ) -> &Self::Target {
                &self.$field
            }
        }

        impl ::core::ops::DerefMut for $container {
            #[inline(always)]
            fn deref_mut( &mut self ) -> &mut Self::Target {
                &mut self.$field
            }
        }
    )
}

#[macro_export]
macro_rules! impl_as_ref {
    ($container: ty, $field_type: ty, $field: ident) => (
        impl ::core::convert::AsRef< $field_type > for $container {
            #[inline(always)]
            fn as_ref( &self ) -> &$field_type {
                &self.$field
            }
        }

        impl ::core::convert::AsMut< $field_type > for $container {
            #[inline(always)]
            fn as_mut( &mut self ) -> &mut $field_type {
                &mut self.$field
            }
        }
    )
}

#[macro_export]
macro_rules! newtype {
    ($(#[$attr:meta])* struct $new: ident = $old: ty) => (
        $(#[$attr])*
        pub struct $new( pub $old );

        impl ::core::ops::Deref for $new {
            type Target = $old;

            #[inline(always)]
            fn deref( &self ) -> &Self::Target {
                &self.0
            }
        }

        impl ::core::ops::DerefMut for $new {
            #[inline(always)]
            fn deref_mut( &mut self ) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl ::core::convert::AsRef< $old > for $new {
            #[inline(always)]
            fn as_ref( &self ) -> &$old {
                &self.0
            }
        }

        impl ::core::convert::AsMut< $old > for $new {
            #[inline(always)]
            fn as_mut( &mut self ) -> &mut $old {
                &mut self.0
            }
        }
    )
}

