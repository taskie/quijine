use std::{
    mem::{self, MaybeUninit},
    ptr, slice,
};

pub fn ref_sized_to_vec<T>(v: &T) -> Vec<u8> {
    ref_sized_to_slice(v).to_vec()
}

pub fn ref_sized_to_slice<T>(v: &T) -> &[u8] {
    let len = mem::size_of::<T>();
    let p = v as *const T as *const u8;
    unsafe { slice::from_raw_parts(p, len) }
}

/// # Safety
/// `v` must represent `T`, and that is memcpy-safe.
#[deny(unsafe_op_in_unsafe_fn)]
pub unsafe fn sized_from_bytes<T>(v: &[u8]) -> T {
    assert_eq!(mem::size_of::<T>(), v.len());
    let mut mu = MaybeUninit::<T>::uninit();
    let p = mu.as_mut_ptr() as *mut u8;
    unsafe {
        ptr::copy_nonoverlapping(v.as_ptr(), p, v.len());
        mu.assume_init()
    }
}

/// # Safety
/// `v` must represent `T`.
#[deny(unsafe_op_in_unsafe_fn)]
pub unsafe fn ref_sized_from_bytes<T>(v: &[u8]) -> &T {
    assert_eq!(mem::size_of::<T>(), v.len());
    let p = v.as_ptr() as *const T;
    unsafe { &*p }
}

#[cfg(test)]
mod tests {
    use crate::internal::*;
    use std::u8;

    #[derive(Debug, PartialEq, Eq)]
    #[repr(packed)]
    struct S1(u8, u8, u8, u8);

    #[test]
    fn test() {
        let s1 = S1(2, 3, 5, 7);
        let r = ref_sized_to_slice(&s1);
        assert_eq!(&[2, 3, 5, 7], r);
        let v = ref_sized_to_vec(&s1);
        assert_eq!(&[2, 3, 5, 7], v.as_slice());
        let rs1 = unsafe { ref_sized_from_bytes::<S1>(r) };
        assert_eq!(s1, *rs1);
        let s1_2 = unsafe { sized_from_bytes::<S1>(r) };
        assert_eq!(s1, s1_2);
    }
}
