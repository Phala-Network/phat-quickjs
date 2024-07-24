use crate::{Config, Worker};
use anyhow::Result;
use rocket::data::Limits;
use rocket::figment::providers::{Env, Format, Toml};
use rocket::figment::Figment;
use rocket::http::ContentType;
use rocket::response::status::Custom;
use rocket::routes;
use rocket::{get, post, Data, State};
use wapod::prpc_service::handle_prpc;

type UserService = wapod::prpc_service::UserService<Config>;

#[post("/<method>?<json>", data = "<data>")]
async fn prpc_post(
    state: &State<Worker>,
    method: &str,
    data: Data<'_>,
    limits: &Limits,
    content_type: Option<&ContentType>,
    json: bool,
) -> Result<Vec<u8>, Custom<Vec<u8>>> {
    handle_prpc::<UserService, _>(state, method, Some(data), limits, content_type, json).await
}

#[get("/<method>")]
async fn prpc_get(
    state: &State<Worker>,
    method: &str,
    limits: &Limits,
    content_type: Option<&ContentType>,
) -> Result<Vec<u8>, Custom<Vec<u8>>> {
    handle_prpc::<UserService, _>(state, method, None, limits, content_type, true).await
}

pub async fn serve_user(state: Worker, port: u16) -> Result<()> {
    let figment = Figment::from(rocket::Config::default())
        .merge(Toml::file("Wapod.toml").nested())
        .merge(Env::prefixed("WAPOD_USER_").global())
        .select("user")
        .merge(("port", port));
    let _rocket = rocket::custom(figment)
        .manage(state)
        .mount("/prpc", routes![prpc_post, prpc_get])
        .launch()
        .await?;
    Ok(())
}
