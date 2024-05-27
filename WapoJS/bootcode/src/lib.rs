#![no_std]
pub const BOOT_CODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bootcode.jsc"));
