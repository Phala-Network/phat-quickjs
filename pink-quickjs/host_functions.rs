use core::ffi::{c_int, c_uchar};
use pink::{error, info};
use qjsbind as js;

pub use derive_key::set_codes;

mod contract_call;
mod hash;
mod http_client;
mod log;
mod derive_key;

#[no_mangle]
extern "C" fn __pink_fd_write(fd: c_int, buf: *const c_uchar, len: usize) -> usize {
    // TODO: a more robust implementation.
    let bin = unsafe { core::slice::from_raw_parts(buf, len) };
    let mut message = core::str::from_utf8(bin).unwrap_or("<Invalid UTF-8 string>");
    if message.ends_with('\n') {
        let new_len = message.len().saturating_sub(1);
        message = unsafe { message.get_unchecked(0..new_len) };
    }
    match fd {
        1 => info!("JS: {}", message),
        2 => error!("JS: {}", message),
        _ => {}
    }
    len
}

#[no_mangle]
extern "C" fn __pink_clock_time_get(_id: u32, _precision: u64, retptr0: *mut u64) -> u16 {
    let t = pink::ext()
        .untrusted_millis_since_unix_epoch()
        .saturating_mul(1_000_000);
    unsafe {
        *retptr0 = t;
    }
    0
}

#[no_mangle]
extern "C" fn __pink_getrandom(pbuf: *mut u8, nbytes: u8) {
    let bytes = pink::ext().getrandom(nbytes);
    if bytes.len() != nbytes as usize {
        panic!("Failed to get random bytes");
    }
    let buf = unsafe { core::slice::from_raw_parts_mut(pbuf, bytes.len()) };
    buf.copy_from_slice(&bytes);
}

pub fn setup_host_functions(ctx: &js::Context) -> js::Result<()> {
    let global_object = ctx.get_global_object();
    let pink = ctx.new_object("Pink");
    log::setup(&pink)?;
    contract_call::setup(&pink)?;
    hash::setup(&pink)?;
    http_client::setup(&pink)?;
    derive_key::setup(&pink)?;
    setup_encoding_functions(&pink, ctx)?;
    global_object.set_property("pink", &pink)?;
    global_object.set_property("Pink", &pink)?;
    Ok(())
}

fn setup_encoding_functions(pink: &js::Value, ctx: &js::Context) -> js::Result<()> {
    use qjs_extensions as ext;
    pink.define_property_fn("utf8Encode", ext::utf8::encode)?;
    pink.define_property_fn("utf8Decode", ext::utf8::decode)?;
    pink.define_property_fn("utf8EncodeInto", ext::utf8::encode_into)?;
    pink.define_property_fn("base64Encode", ext::base64::encode)?;
    pink.define_property_fn("base64Decode", ext::base64::decode)?;
    pink.define_property_fn("hexEncode", ext::hex::encode)?;
    pink.define_property_fn("hexDecode", ext::hex::decode)?;
    let scale = ctx.new_object("SCALE");
    ext::scale2::setup(&scale, ctx)?;
    ext::repr::setup(pink)?;
    pink.set_property("SCALE", &scale)?;
    Ok(())
}
