//! Tournament server properties
use yew::{html::IntoPropValue, Properties};

/// Totsugeki api as seen by a component
#[derive(Clone, PartialEq, Eq)]
pub struct Api {
    /// url of the API
    addr: String,
}

impl Api {
    /// Returns the API url
    pub fn get_backend_addr(&self) -> String {
        self.addr.clone()
    }
}

/// Properties needed to interact with tournament server
#[derive(Eq, PartialEq, Properties)]
pub struct Props {
    /// API
    pub props: Api,
}

impl Default for Props {
    fn default() -> Self {
        let addr = std::env!("API_ADDR");
        let port = std::env!("API_PORT");
        let addr = if port.is_empty() {
            addr.to_owned()
        } else {
            format!("{addr}:{port}")
        };
        Self {
            props: Api { addr },
        }
    }
}

impl IntoPropValue<Api> for Props {
    fn into_prop_value(self) -> Api {
        Api {
            addr: self.props.addr,
        }
    }
}
