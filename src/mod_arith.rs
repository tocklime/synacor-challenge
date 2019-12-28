use std::ops::{Rem, Shr};
use std::fmt::Debug;
use num::{Num, CheckedMul, CheckedAdd, CheckedSub};
use num::traits::WrappingMul;

pub fn mod_pow<T>(mut base: T, mut exp: T, modulus: T) -> T
    where T: Num + Copy + Shr<Output = T> + From<u8> + PartialOrd + WrappingMul
{
    if modulus == T::one() {
        return T::zero();
    }
    let mut result = T::one();
    base = base % modulus;
    while exp > T::zero() {
        if exp % 2.into() == T::one() {
            result = mod_mul(result , base , modulus);
        }
        exp = exp >> T::one();
        base = mod_mul(base , base ,modulus);
    }
    result
}
pub fn mod_mul<T>(a : T, b: T, m : T) -> T
    where T : WrappingMul + Rem<Output = T> + Copy
{
    a.wrapping_mul(&b) % m
}
pub fn mod_add<T>(a:&T, b: &T, m: T) -> T
    where T : CheckedAdd + Rem<Output = T> + Debug
{
    match a.checked_add(&b){
        None => panic!("mod_add overflowed with {:?}+{:?}%{:?}",a,b,m),
        Some(ab) => ab % m
    }
}
pub fn mod_sub<T>(a: &T, b: &T, m: T) -> T
    where T : CheckedSub + Rem<Output = T> + Debug
{

    match a.checked_sub(b){
        None => panic!("mod_sub underflowed with {:?}-{:?}%{:?}",a,b,m),
        Some(ab) => ab % m
    }
}

