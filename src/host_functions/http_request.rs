use super::*;

pub(super) fn http_request(
    service: ServiceRef,
    ctx: *mut c::JSContext,
    args: &[c::JSValueConst],
) -> Result<JsValue> {
    let Some(config) = args.get(0) else {
        anyhow::bail!("Invoking http_request without arguments");
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
