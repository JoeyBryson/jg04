//! The database module of ___-lib.
//!
//! This is the bottom layer of the crate and the source of truth for
//! application state during runtime.
//!
//! Database calls can be made from anywhere through the [`DbClient`] API.
//!
//! ```
//! db_client.add_nw_chat(chat);
//! let chats: Vec<NwChat> = db_client.get_nw_chats();
//! ```
//!
//! [`DbClient`] instances are spawned from the [`DbManager`], which
//! manages database workers and connections.
//!
//! ```
//! let db_manager = DbManager::spawn(db_path_string);
//! let db_client = db_manager.spawn_client();
//! ```
//! 
//! The database layer should remain independent of higher-level
//! application logic, but it does take and return primative types 
//! from other dependencies needed to represent Ui or Networking state strictly through the
//! methods of DbClient.

pub mod client;
pub(crate) mod macros;
pub mod manager;
pub(crate) mod requests;
pub mod sample_data_insertions;
pub(crate) mod workers;