use std::collections::BTreeMap;

use super::*;

use ::url::{form_urlencoded, Url};
use anyhow::Context;

pub(super) fn parse_url(
    _service: ServiceRef,
    ctx: *mut c::JSContext,
    args: &[c::JSValueConst],
) -> Result<JsValue> {
    if args.len() < 1 {
        return Err("URL: expected at least one argument").anyhow();
    }
    let url: String = DecodeFromJSValue::decode(ctx, args[0])
        .anyhow()
        .context("URL: Failed to decode url")?;
    let url = if args.len() > 1 {
        let base_url: String = DecodeFromJSValue::decode(ctx, args[1])
            .anyhow()
            .context("URL: Invalid base URL")?;
        let base_url: Url = base_url.parse().context("URL: Invalid base URL")?;
        base_url.join(&url).context("URL: Invalid URL")?
    } else {
        Url::parse(&url).context("URL: Invalid URL")?
    };

    let mut attrs = BTreeMap::new();
    fn set_value(o: &mut BTreeMap<String, JsValue>, k: &str, v: &str) {
        o.insert(k.to_string(), JsValue::String(v.to_string()));
    }
    set_value(&mut attrs, "hash", url.fragment().unwrap_or(""));
    set_value(&mut attrs, "host", url.host_str().unwrap_or(""));
    set_value(&mut attrs, "hostname", url.host_str().unwrap_or(""));
    set_value(&mut attrs, "href", url.as_str());
    set_value(&mut attrs, "origin", &url.origin().unicode_serialization());
    set_value(&mut attrs, "password", url.password().unwrap_or(""));
    set_value(&mut attrs, "pathname", &url.path().to_string());
    set_value(
        &mut attrs,
        "port",
        &url.port().map(|p| p.to_string()).unwrap_or("".to_string()),
    );
    set_value(&mut attrs, "protocol", &url.scheme().to_string());
    set_value(&mut attrs, "search", url.query().unwrap_or(""));
    set_value(&mut attrs, "username", url.username());

    Ok(JsValue::Object(attrs))
}

pub(super) fn parse_search_params(
    _service: ServiceRef,
    ctx: *mut c::JSContext,
    args: &[c::JSValueConst],
) -> Result<JsValue> {
    let query = args
        .get(0)
        .ok_or("URLSearchParams: expected at least one argument")
        .anyhow()?;
    let query_str: String = DecodeFromJSValue::decode(ctx, *query)
        .anyhow()
        .context("URLSearchParams: Failed to decode query")?;
    let pairs: Vec<_> = form_urlencoded::parse(query_str.as_bytes())
        .map(|(k, v)| {
            JsValue::Array(vec![
                JsValue::String(k.to_string()),
                JsValue::String(v.to_string()),
            ])
        })
        .collect();
    Ok(JsValue::Array(pairs))
}
