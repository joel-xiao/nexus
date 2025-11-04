pub mod registry;
pub mod factory;
pub mod generic;
pub mod wrapper;
pub mod implementations;

pub use registry::{Adapter, AdapterRegistry};
pub use factory::AdapterFactory;
pub use generic::{GenericAdapter, RequestConfig, AuthType};
pub use wrapper::WrappedAdapter;
