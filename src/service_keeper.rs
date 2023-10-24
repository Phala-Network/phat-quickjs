use anyhow::{anyhow, bail, Result};
use log::error;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::BTreeMap};

use crate::runtime::AccountId;
use crate::service::{Service, ServiceRef};

#[derive(Debug, Serialize, Deserialize)]
enum Message {
    Run {
        name: String,
        source: String,
        #[serde(default)]
        reset: bool,
    },
    Reset {
        name: String,
    },
}

thread_local! {
    static KEEPER: RefCell<ServiceKeeper>  = RefCell::new(ServiceKeeper::new());
}

pub struct ServiceKeeper {
    services: BTreeMap<String, ServiceRef>,
}

impl ServiceKeeper {
    pub const fn new() -> Self {
        Self {
            services: BTreeMap::new(),
        }
    }
}

/// Operations on the singleton `ServiceKeeper`.
impl ServiceKeeper {
    pub fn reset(name: &str) {
        KEEPER.with(|keeper| {
            keeper.borrow_mut().remove_service(name);
        });
    }

    /// Exceute a script in the service named `name`.
    ///
    /// If the service does not exist, it will be created. If the service already exists, the state
    /// of the service will keep until the service is reset.
    pub fn exec_script(name: &str, source: &str) {
        let service = KEEPER.with(|keeper| keeper.borrow_mut().get_service_or_default(name));
        match service.exec_script(source) {
            Ok(_) => {}
            Err(err) => {
                error!("Executing script [{name}] returned error: {err}");
            }
        }
    }

    pub fn handle_query(_from: Option<AccountId>, query: &[u8]) -> Vec<u8> {
        if String::from_utf8_lossy(query) == "ping" {
            return "pong".into();
        }
        Vec::new()
    }

    pub fn handle_message(message: Vec<u8>) {
        let message = match serde_json::from_slice::<Message>(&message) {
            Ok(message) => message,
            Err(err) => {
                error!("Failed to parse incoming message: {err}");
                return;
            }
        };
        match message {
            Message::Run {
                name,
                source,
                reset,
            } => {
                if reset {
                    Self::reset(&name);
                }
                Self::exec_script(&name, &source);
            }
            Message::Reset { name } => Self::reset(&name),
        }
    }

    pub fn handle_connection(connection: crate::runtime::HttpRequest) -> Result<()> {
        let url: url::Url = connection.head.url.parse()?;
        let name = url
            .path_segments()
            .ok_or(anyhow!("Failed to get path segments"))?
            .next()
            .ok_or(anyhow!("Failed to get service name from path"))?;
        let Some(service) = KEEPER.with(|keeper| keeper.borrow_mut().get_service(name)) else {
            connection
                .response_tx
                .send(crate::runtime::HttpResponseHead {
                    status: 404,
                    headers: vec![("Content-Length".into(), "0".into())],
                })
                .map_err(|err| anyhow!("Failed to send response: {err:?}"))?;
            bail!("Service [{name}] not found");
        };
        crate::host_functions::try_accept_http_request(service, connection)?;
        Ok(())
    }
}

impl ServiceKeeper {
    fn get_service(&self, name: &str) -> Option<ServiceRef> {
        self.services.get(name).cloned()
    }

    fn get_service_or_default(&mut self, name: &str) -> ServiceRef {
        if let Some(service) = self.get_service(name) {
            return service;
        }
        let service = Service::new_ref();
        self.services.insert(name.into(), service.clone());
        service
    }

    fn remove_service(&mut self, name: &str) {
        self.services.remove(name);
    }
}
