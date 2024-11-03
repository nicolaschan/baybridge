pub mod client;
pub mod configuration;
pub mod connectors;
pub mod crdt;
pub mod crypto;
pub mod models;
pub mod server;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
