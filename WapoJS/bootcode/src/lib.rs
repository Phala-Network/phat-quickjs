#![no_std]
pub const BOOT_CODE_WAPO: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bootcode-wapo.jsc"));
pub const BOOT_CODE_NODEJS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bootcode-nodejs.jsc"));
pub const BOOT_CODE_BROWSER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bootcode-browser.jsc"));
pub const BOOT_CODE: &[u8] = BOOT_CODE_NODEJS;
