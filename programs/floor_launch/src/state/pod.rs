//!
//!
//!
//!
//!

use anchor_lang::prelude::*;

#[zero_copy]
#[derive(Debug, Default, PartialEq, Eq)]
pub struct U128 {
///
    pub limbs: [u64; 2],
}

impl U128 {
    pub const fn new(v: u128) -> Self {
        Self { limbs: [v as u64, (v >> 64) as u64] }
    }
    pub const fn get(&self) -> u128 {
        (self.limbs[0] as u128) | ((self.limbs[1] as u128) << 64)
    }
    pub fn set(&mut self, v: u128) {
        *self = Self::new(v);
    }
}

#[zero_copy]
#[derive(Debug, Default, PartialEq, Eq)]
pub struct I128 {
///
    pub limbs: [u64; 2],
}

impl I128 {
    pub const fn new(v: i128) -> Self {
        let u = v as u128;
        Self { limbs: [u as u64, (u >> 64) as u64] }
    }
    pub const fn get(&self) -> i128 {
        ((self.limbs[0] as u128) | ((self.limbs[1] as u128) << 64)) as i128
    }
    pub fn set(&mut self, v: i128) {
        *self = Self::new(v);
    }
}

const _: () = assert!(core::mem::size_of::<U128>() == 16 && core::mem::align_of::<U128>() == 8);
const _: () = assert!(core::mem::size_of::<I128>() == 16 && core::mem::align_of::<I128>() == 8);
