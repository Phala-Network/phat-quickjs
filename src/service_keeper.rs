use log::error;
use std::{cell::RefCell, collections::BTreeMap};

use crate::runtime::AccountId;
use crate::service::{Service, ServiceRef};

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

    pub fn handle_query(_from: Option<AccountId>, _query: &[u8]) -> Vec<u8> {
        Self::exec_script("", "");
        todo!()
    }

    pub fn handle_message(_message: Vec<u8>) {
        Self::reset("");
        todo!()
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
