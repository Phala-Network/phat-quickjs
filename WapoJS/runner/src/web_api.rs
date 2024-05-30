use crate::{Config, Worker};
use anyhow::Result;
use rocket::data::Limits;
use rocket::figment::providers::{Env, Format, Toml};
use rocket::figment::Figment;
use rocket::http::ContentType;
use rocket::response::status::Custom;
use rocket::routes;
use rocket::{get, http::Status, post, Data, State};
use std::path::PathBuf;
use wapod::ext::{RequestInfo, StreamResponse};
use wapod::prpc_service::{connect_vm, handle_prpc, HexBytes};

type UserService = wapod::prpc_service::UserService<Config>;

#[post("/app/<id>/<path..>", data = "<body>")]
async fn connect_vm_post<'r>(
    state: &State<Worker>,
    head: RequestInfo,
    id: HexBytes,
    path: PathBuf,
    body: Data<'r>,
) -> Result<StreamResponse, (Status, String)> {
    connect_vm(state, head, id, path, Some(body)).await
}

#[get("/app/<id>/<path..>")]
async fn connect_vm_get<'r>(
    state: &State<Worker>,
    head: RequestInfo,
    id: HexBytes,
    path: PathBuf,
) -> Result<StreamResponse, (Status, String)> {
    connect_vm(state, head, id, path, None).await
}

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
        .mount("/", routes![connect_vm_get, connect_vm_post])
        .mount("/prpc", routes![prpc_post, prpc_get])
        .launch()
        .await?;
    Ok(())
}
