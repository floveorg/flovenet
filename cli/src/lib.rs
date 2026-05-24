use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "flovenet", about = "Flovenet - Red social descentralizada")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Inicia el proceso daemon del nodo
    Daemon {
        /// Puerto libp2p
        #[arg(long, default_value = "0")]
        port: u16,
        /// Puerto HTTP para metrics/API
        #[arg(long, default_value_t = 9090)]
        api_port: u16,
        /// Roles del nodo (compute, storage, validation, ai, social)
        #[arg(long, default_value_t = String::new())]
        roles: String,
        /// Swarm key file (PSK) for private sub-network
        #[arg(long)]
        swarm_key: Option<String>,
    },
    /// Inicia el gateway GraphQL
    ApiGateway {
        /// Puerto HTTP
        #[arg(long, default_value_t = 8080)]
        port: u16,
    },
    /// Comparte recursos locales
    Share {
        #[arg(long)]
        role: Option<String>,
    },
    /// Ejecuta un WASM localmente
    Run {
        /// Nombre de la función entrypoint (_start, run, etc.)
        #[arg(long, default_value = "_start")]
        manifest: String,
        /// CID o path del WASM image
        #[arg(long)]
        image: Option<String>,
    },
    /// Muestra estado del nodo
    Status,
}
