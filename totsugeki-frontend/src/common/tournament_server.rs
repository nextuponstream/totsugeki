//! Tournament server properties
use yew::{html::IntoPropValue, Properties};

/// Tournament server as seen by a component
#[derive(Clone, PartialEq)]
pub struct TournamentServer {
    addr: String,
}

impl TournamentServer {
    pub fn get_backend_addr(&self) -> String {
        self.addr.clone()
    }
}

/// Properties needed to interact with tournament server
#[derive(PartialEq, Properties)]
pub struct Props {
    pub props: TournamentServer,
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
            props: TournamentServer { addr },
        }
    }
}

impl IntoPropValue<TournamentServer> for Props {
    fn into_prop_value(self) -> TournamentServer {
        TournamentServer {
            addr: self.props.addr,
        }
    }
}
