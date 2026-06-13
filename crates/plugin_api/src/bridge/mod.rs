//! This module provides data structures and enumrates that are used
//! as a bridge between the communication of both the plugin and the host.

mod ipc;
mod registration;
mod handlers;
mod shared;
mod library;
mod future;

pub use ipc::*;
pub use registration::*;
pub use handlers::*;
pub use shared::*;
pub use library::*;
pub use future::*;
