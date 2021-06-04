#[repr(transparent)]
pub struct Opaque<const N: usize>(pub(crate) [u8; N]);

impl<const N: usize> Default for Opaque<N> {
    fn default() -> Self {
        Opaque([0; N])
    }
}
