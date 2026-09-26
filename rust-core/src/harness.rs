//! Rust-only test harness for exercising networking behaviour end-to-end
//! without any Kotlin/UI wiring. A [`TestNetwork`] runs a local iroh relay
//! and DNS/pkarr discovery server (no real internet access required), and
//! each [`TestNode`] spawned from it pairs an in-memory database with a
//! running [`NwCore`], mirroring what `AppCore` wires up on the Kotlin side.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;
use iroh::RelayMap;
use iroh::RelayMode;
use iroh::SecretKey;
use iroh::endpoint::Builder;
use iroh::endpoint::presets::Preset;
use iroh::test_utils::DnsPkarrServer;
use iroh::tls::CaTlsConfig;
use rusqlite::Connection;
use tokio::runtime::Runtime;

use crate::database::client::DbClient;
use crate::database::manager::{DbManager, DbMode, start_conn};
use crate::network::{NwContact, NwCore, NwProfile};

/// A local relay + DNS/pkarr discovery pair shared by every [`TestNode`]
/// spawned from it, so nodes can find and reach each other without any real
/// internet access.
pub struct TestNetwork {
    relay_map: RelayMap,
    dns_pkarr: DnsPkarrServer,
    // Keeps the relay server running for the lifetime of the network.
    _relay_server: iroh_relay::server::Server,
    runtime: Runtime,
}

impl TestNetwork {
    pub fn spawn() -> Result<Self> {
        let runtime = Runtime::new()?;

        let (relay_map, _relay_url, relay_server) = runtime
            .block_on(iroh::test_utils::run_relay_server())
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        let dns_pkarr = runtime.block_on(DnsPkarrServer::run())?;

        Ok(Self {
            relay_map,
            dns_pkarr,
            _relay_server: relay_server,
            runtime,
        })
    }

    /// Spawns a node with a fresh in-memory database and a freshly generated identity.
    pub fn spawn_node(&self, name: impl Into<String>) -> Result<TestNode> {
        let (db_manager, memory_guard) = spawn_in_memory_db_manager()?;
        let db_client = db_manager.spawn_client();

        let secret_key = SecretKey::generate();
        let profile = NwProfile {
            contact: NwContact {
                name: name.into(),
                endpoint_id: secret_key.public(),
            },
            secret_key,
        };
        db_client.set_profile(profile)?;

        let preset = LocalNetworkPreset {
            dns_pkarr: &self.dns_pkarr,
            relay_map: self.relay_map.clone(),
        };
        let nw_core = NwCore::spawn_for_test(db_client.clone(), preset)?;

        Ok(TestNode {
            db_manager,
            db_client,
            nw_core,
            _memory_guard: memory_guard,
        })
    }
}

struct LocalNetworkPreset<'a> {
    dns_pkarr: &'a DnsPkarrServer,
    relay_map: RelayMap,
}

impl Preset for LocalNetworkPreset<'_> {
    fn apply(self, builder: Builder) -> Builder {
        builder
            .preset(self.dns_pkarr.preset())
            .relay_mode(RelayMode::Custom(self.relay_map))
            .ca_tls_config(CaTlsConfig::insecure_skip_verify())
    }
}

pub struct TestNode {
    pub db_manager: DbManager,
    pub db_client: Arc<DbClient>,
    pub nw_core: Arc<NwCore>,
    // Keeps the shared-cache in-memory database alive; SQLite discards it
    // once every connection to it closes.
    _memory_guard: Connection,
}

/// Builds a `DbManager` backed by a private, in-memory SQLite database.
///
/// Each call creates an independent database identified by a process-unique
/// name, so multiple `TestNode`s can coexist without interfering with one
/// another. The returned `Connection` must be kept alive for as long as the
/// `DbManager` is used.
fn spawn_in_memory_db_manager() -> Result<(DbManager, Connection)> {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let db_path = PathBuf::from(format!("file:memdb_{id}?mode=memory&cache=shared"));

    let memory_guard = start_conn(&db_path, DbMode::ReadWrite)?;
    memory_guard.execute_batch(include_str!("database/sql/schema.sql"))?;

    let db_manager = DbManager::new(db_path)?;

    Ok((db_manager, memory_guard))
}


