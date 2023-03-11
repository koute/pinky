use core::ptr;
use core::slice;
use core::mem;
use alloc::boxed::Box;
use alloc::vec::Vec;

#[inline]
pub fn as_bytes< T: Copy >( array: &[T] ) -> &[u8] {
    unsafe {
        slice::from_raw_parts( mem::transmute( array.as_ptr() ), mem::size_of::<T>() * array.len() )
    }
}

#[inline]
pub fn allocate_slice< T: Default + Copy + Sized >( size: usize ) -> Box< [T] > {
    let mut vector = Vec::with_capacity( size );
    unsafe {
        vector.set_len( size );
    }

    for p in vector.iter_mut() {
        *p = T::default();
    }

    vector.into_boxed_slice()
}

// Since copy_memory got deprecated and copy_from_slice is still unstable.
#[inline]
pub fn copy_memory( src: &[u8], dst: &mut [u8] ) {
    let length = src.len();
    assert!( dst.len() >= length );

    unsafe {
        ptr::copy_nonoverlapping( src.as_ptr(), dst.as_mut_ptr(), length );
    }
}
