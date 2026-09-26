use std::sync::Arc;

use anyhow::Result;
use iroh::RelayMap;
use iroh::RelayMode;
use iroh::SecretKey;
use iroh::endpoint::Builder;
use iroh::endpoint::presets::Preset;
use iroh::test_utils::DnsPkarrServer;
use iroh::tls::CaTlsConfig;
use tempfile::TempDir;
use tokio::runtime::Runtime;

use rust_api::database::client::DbClient;
use rust_api::database::manager::DbManager;
use rust_api::network::{NwContact, NwCore, NwProfile};

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

    /// Spawns a node with a fresh temporary database and a freshly generated identity.
    pub fn spawn_node(&self, name: impl Into<String>) -> Result<TestNode> {
        let (db_manager, temp_dir) = spawn_test_db_manager()?;
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
            _temp_dir: temp_dir,
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
    // Keeps the temporary database alive for the lifetime of the node.
    _temp_dir: TempDir,
}

/// Builds a [`DbManager`] backed by a private temporary SQLite database.
///
/// Each call creates an independent temporary directory, so multiple
/// [`TestNode`]s can coexist without interfering with one another.
fn spawn_test_db_manager() -> Result<(DbManager, TempDir)> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("app.db");

    let db_manager = DbManager::new(db_path)?;

    Ok((db_manager, temp_dir))
}