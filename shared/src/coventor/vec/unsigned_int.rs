use std::convert::TryInto;

pub trait FromLeBytes: Sized {
    fn from_le_bytes(bytes: &[u8]) -> Option<Self>;
}

macro_rules! impl_from_le_bytes {
    ($t:ty, $size:expr) => {
        impl FromLeBytes for $t {
            fn from_le_bytes(bytes: &[u8]) -> Option<Self> {
                if bytes.len() < $size {
                    return None;
                }
                let array: [u8; $size] = bytes[0..$size].try_into().ok()?;
                Some(<$t>::from_le_bytes(array))
            }
        }
    };
}

impl_from_le_bytes!(u16, 2);
impl_from_le_bytes!(u32, 4);
impl_from_le_bytes!(u64, 8);
impl_from_le_bytes!(u128, 16);
impl_from_le_bytes!(usize, std::mem::size_of::<usize>());

pub fn vec_to_unsigned_int<T: FromLeBytes>(vec: &Vec<u8>) -> Option<T> {
    T::from_le_bytes(&vec)
}
