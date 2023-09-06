use core::ffi::{c_int, c_uchar};
use pink::{error, info};
use qjsbind::{Context as JsContext, Result as JsResult};

mod log;

#[no_mangle]
extern "C" fn __pink_fd_write(fd: c_int, buf: *const c_uchar, len: usize) -> usize {
    // TODO: a more robust implementation.
    let bin = unsafe { core::slice::from_raw_parts(buf, len) };
    let mut message = core::str::from_utf8(bin).unwrap_or("<Invalid UTF-8 string>");
    if message.ends_with('\n') {
        let new_len = message.len() - 1;
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
    let t = pink::ext().untrusted_millis_since_unix_epoch() * 1_000_000;
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

pub fn setup_host_functions(ctx: &JsContext) -> JsResult<()> {
    let global_object = ctx.get_global_object();
    let pink = ctx.new_object();
    log::setup(&pink)?;
    global_object.set_property("pink", &pink)
}
