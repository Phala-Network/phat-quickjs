extern crate alloc;

pub use service::Service;

mod host_functions;
mod service;

pub mod js_eval;
mod traits;

#[cfg(feature = "native")]
pub use native_runtime as runtime;
#[cfg(feature = "wapo")]
pub use wapo_runtime as runtime;

pub mod wapo_runtime;

#[cfg(feature = "native")]
pub mod native_runtime;
