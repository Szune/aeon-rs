use std::ops::{BitAnd, BitOrAssign};

#[inline]
pub fn has<T>(value: T, flag: T) -> bool
where
    T: BitAnd<Output = T> +
    Copy +
    PartialEq +
    Eq,
{
    value & flag == flag
}

#[inline]
pub fn add<T>(value: &mut T, flag: T)
where
    T: BitOrAssign + Copy
{
    *value |= flag;
}
