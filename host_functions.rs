use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use ink::env::hash::CryptoHash;
use pink::chain_extension::{HttpRequest, HttpResponse};

use core::ffi::{c_int, c_uchar};
use pink_extension::{error, info};
use qjs_sys::c;
use qjs_sys::convert::{
    js_array_for_each as for_each, js_object_get_field as get_field,
    js_object_get_field_or_default as get_field_or_default,
    js_object_get_option_field as get_option_field, js_val_from_bytes, serialize_value,
    DecodeFromJSValue, HashableBytes, JsValue,
};

use pink_extension as pink;

use crate::{code_hash, contract_call};

trait IntoJsValue {
    fn into_js_value(self, ctx: *mut c::JSContext) -> c::JSValue;
}

impl<T: AsRef<str>> IntoJsValue for Result<c::JSValue, T> {
    fn into_js_value(self, ctx: *mut c::JSContext) -> c::JSValue {
        match self {
            Ok(v) => v,
            Err(err) => {
                qjs_sys::throw_type_error(ctx, err.as_ref());
                c::JS_EXCEPTION
            }
        }
    }
}

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
    let t = pink_extension::ext().untrusted_millis_since_unix_epoch() * 1_000_000;
    unsafe {
        *retptr0 = t;
    }
    0
}

#[no_mangle]
fn __pink_host_call(id: u32, ctx: *mut c::JSContext, args: &[c::JSValueConst]) -> c::JSValue {
    match id {
        0 => host_invoke_contract(ctx, args).into_js_value(ctx),
        1 => host_invoke_contract_delegate(ctx, args).into_js_value(ctx),
        2 => host_http_request(ctx, args).into_js_value(ctx),
        3 => host_batch_http_request(ctx, args).into_js_value(ctx),
        4 => host_derive_secret(ctx, args).into_js_value(ctx),
        5 => host_hash(ctx, args).into_js_value(ctx),
        _ => {
            error!("JS: host call with unknown id: {id}");
            qjs_sys::throw_type_error(ctx, &alloc::format!("Invalid host call id: {id}"));
            c::JS_EXCEPTION
        }
    }
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

fn host_invoke_contract(
    ctx: *mut c::JSContext,
    args: &[c::JSValueConst],
) -> Result<c::JSValue, String> {
    let Some(config) = args.get(0) else {
        return Err("Invoking contract without arguments".into());
    };

    let callee: [u8; 32] = get_field(ctx, *config, "callee")?;
    let gas_limit: u64 = get_field_or_default(ctx, *config, "gasLimit")?;
    let transferred_value: u128 = get_field_or_default(ctx, *config, "value")?;
    let selector: u32 = get_field(ctx, *config, "selector")?;
    let input: Vec<u8> = get_field(ctx, *config, "input")?;
    let allow_reentry: bool = get_field(ctx, *config, "allowReentry").unwrap_or_default();

    let output = contract_call::invoke_contract(
        callee.into(),
        gas_limit,
        transferred_value,
        selector,
        &input,
        allow_reentry,
    )
    .map_err(|err| alloc::format!("{:?}", err))?;
    Ok(js_val_from_bytes(ctx, &output))
}

fn host_invoke_contract_delegate(
    ctx: *mut c::JSContext,
    args: &[c::JSValueConst],
) -> Result<c::JSValue, String> {
    let Some(config) = args.get(0) else {
        return Err("Invoking contract delegate without arguments".into());
    };

    let delegate: [u8; 32] = get_field(ctx, *config, "codeHash")?;
    let selector: u32 = get_field(ctx, *config, "selector")?;
    let input: Vec<u8> = get_field(ctx, *config, "input")?;
    let allow_reentry: bool = get_field(ctx, *config, "allowReentry").unwrap_or_default();

    let output =
        contract_call::invoke_contract_delegate(delegate.into(), selector, &input, allow_reentry)
            .map_err(|err| alloc::format!("{:?}", err))?;

    Ok(js_val_from_bytes(ctx, &output))
}

fn host_http_request(
    ctx: *mut c::JSContext,
    args: &[c::JSValueConst],
) -> Result<c::JSValue, String> {
    let Some(config) = args.get(0) else {
        return Err("Invoking http_request without arguments".into());
    };

    let url: String = get_field(ctx, *config, "url")?;
    let method: String = get_option_field(ctx, *config, "method")?.unwrap_or_else(|| "GET".into());
    let headers: BTreeMap<String, String> = get_field_or_default(ctx, *config, "headers")?;
    let body: Vec<u8> = get_field_or_default(ctx, *config, "body")?;
    let return_text_body: bool = get_field_or_default(ctx, *config, "returnTextBody")?;

    let HttpResponse {
        status_code,
        reason_phrase,
        headers,
        body,
    } = pink::http_req!(&method, &url, body, headers.into_iter().collect());
    let status_code = JsValue::Int(status_code as _);
    let reason_phrase = JsValue::String(reason_phrase);
    let headers: BTreeMap<String, JsValue> = headers
        .into_iter()
        .map(|(k, v)| (k, JsValue::String(v)))
        .collect();
    let headers = JsValue::Object(headers);
    let body = if return_text_body {
        JsValue::String(String::from_utf8_lossy(&body).into())
    } else {
        JsValue::Bytes(body)
    };
    let mut response_object: BTreeMap<String, JsValue> = Default::default();
    response_object.insert("statusCode".into(), status_code);
    response_object.insert("reasonPhrase".into(), reason_phrase);
    response_object.insert("headers".into(), headers);
    response_object.insert("body".into(), body);

    Ok(serialize_value(ctx, JsValue::Object(response_object))?)
}

fn host_batch_http_request(
    ctx: *mut c::JSContext,
    args: &[c::JSValueConst],
) -> Result<c::JSValue, String> {
    let Some(config) = args.get(0) else {
        return Err("Invoking batch_http_request without arguments".into());
    };
    let timeout_ms: u64 = match args.get(1) {
        Some(timeout) => DecodeFromJSValue::decode(ctx, *timeout)?,
        None => 10,
    };
    let mut requests = Vec::new();
    let mut return_text_bodies = Vec::new();

    for_each(ctx, *config, |element| {
        let url: String = get_field(ctx, element, "url")?;
        let method: String =
            get_option_field(ctx, element, "method")?.unwrap_or_else(|| "GET".into());
        let headers: BTreeMap<String, String> = get_field_or_default(ctx, element, "headers")?;
        let body: Vec<u8> = get_field_or_default(ctx, element, "body")?;
        let return_text_body: bool = get_field_or_default(ctx, element, "returnTextBody")?;
        requests.push(HttpRequest {
            url,
            method,
            headers: headers.into_iter().collect(),
            body,
        });
        return_text_bodies.push(return_text_body);
        Ok(())
    })?;

    let responses = pink::ext()
        .batch_http_request(requests, timeout_ms)
        .map_err(|err| alloc::format!("Failed to call batch_http_request: {err:?}"))?;

    let mut response_objects = Vec::new();
    if responses.len() != return_text_bodies.len() {
        return Err("Mismatch between number of responses and returnTextBody flags".into());
    }

    for (response, return_text_body) in responses.into_iter().zip(return_text_bodies.into_iter()) {
        let mut response_object: BTreeMap<String, JsValue> = Default::default();
        match response {
            Ok(response) => {
                let HttpResponse {
                    status_code,
                    reason_phrase,
                    headers,
                    body,
                } = response;
                let status_code = JsValue::Int(status_code as _);
                let reason_phrase = JsValue::String(reason_phrase);
                let headers: BTreeMap<String, JsValue> = headers
                    .into_iter()
                    .map(|(k, v)| (k, JsValue::String(v)))
                    .collect();
                let headers = JsValue::Object(headers);
                let body = if return_text_body {
                    JsValue::String(String::from_utf8_lossy(&body).into())
                } else {
                    JsValue::Bytes(body)
                };
                response_object.insert("statusCode".into(), status_code);
                response_object.insert("reasonPhrase".into(), reason_phrase);
                response_object.insert("headers".into(), headers);
                response_object.insert("body".into(), body);
            }
            Err(err) => {
                response_object
                    .insert("error".into(), JsValue::String(alloc::format!("{:?}", err)));
            }
        }
        response_objects.push(JsValue::Object(response_object));
    }
    Ok(serialize_value(ctx, JsValue::Array(response_objects))?)
}

fn host_derive_secret(
    ctx: *mut c::JSContext,
    args: &[c::JSValueConst],
) -> Result<c::JSValue, String> {
    let HashableBytes(salt) = DecodeFromJSValue::decode(ctx, *args.get(0).ok_or("Missing salt")?)?;
    let salt: Vec<u8> = b"javascript."
        .iter()
        .chain(&code_hash())
        .chain(&salt)
        .copied()
        .collect();
    let secret = pink::ext().derive_sr25519_key(salt.into());
    Ok(serialize_value(ctx, JsValue::Bytes(secret))?)
}

fn host_hash(ctx: *mut c::JSContext, args: &[c::JSValueConst]) -> Result<c::JSValue, String> {
    let algorithm = String::decode(ctx, *args.get(0).ok_or("Missing algorithm")?)?;
    let HashableBytes(message) =
        DecodeFromJSValue::decode(ctx, *args.get(1).ok_or("Missing message")?)?;
    let hash = match algorithm.as_str() {
        "blake2b128" => {
            let mut output = Default::default();
            ink::env::hash::Blake2x128::hash(&message, &mut output);
            output.to_vec()
        }
        "blake2b256" => {
            let mut output = Default::default();
            ink::env::hash::Blake2x256::hash(&message, &mut output);
            output.to_vec()
        }
        "keccak256" => {
            let mut output = Default::default();
            ink::env::hash::Keccak256::hash(&message, &mut output);
            output.to_vec()
        }
        "sha256" => {
            let mut output = Default::default();
            ink::env::hash::Sha2x256::hash(&message, &mut output);
            output.to_vec()
        }
        _ => return Err(alloc::format!("Unsupported hash algorithm: {}", algorithm)),
    };
    Ok(serialize_value(ctx, JsValue::Bytes(hash))?)
}
