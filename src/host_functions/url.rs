use anyhow::Context;
use js::ToJsValue;
use std::collections::BTreeMap;
use url::{form_urlencoded, Url};

use super::Result;

#[derive(ToJsValue, Debug)]
struct URL {
    host: String,
    hostname: String,
    href: String,
    origin: String,
    password: String,
    pathname: String,
    hash: String,
    port: String,
    protocol: String,
    search: String,
    username: String,
}

#[js::host_call]
fn parse_url(url: String, base_url: Option<String>) -> Result<URL> {
    let url = match base_url {
        Some(base_url) => {
            let base_url: Url = base_url.parse().context("URL: Invalid base URL")?;
            base_url.join(&url).context("URL: Invalid URL")?
        }
        None => url.parse().context("URL: Invalid URL")?,
    };

    Ok(URL {
        hash: url.fragment().unwrap_or("").to_string(),
        host: url.host_str().unwrap_or("").to_string(),
        hostname: url.host_str().unwrap_or("").to_string(),
        href: url.as_str().to_string(),
        origin: url.origin().unicode_serialization(),
        password: url.password().unwrap_or("").to_string(),
        pathname: url.path().to_string(),
        port: url.port().map(|p| p.to_string()).unwrap_or("".to_string()),
        protocol: url.scheme().to_string(),
        search: url.query().unwrap_or("").to_string(),
        username: url.username().to_string(),
    })
}

#[js::host_call]
fn parse_search_params(query_str: String) -> Result<BTreeMap<String, String>> {
    Ok(form_urlencoded::parse(query_str.as_bytes())
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect())
}

pub(crate) fn setup(ns: &js::Value) -> Result<()> {
    ns.define_property_fn("parseURL", parse_url)?;
    ns.define_property_fn("parseURLParams", parse_search_params)?;
    Ok(())
}
