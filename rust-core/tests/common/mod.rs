use anyhow::Result;
use iroh::Endpoint;
use iroh::RelayMap;
use iroh::RelayMode;
use iroh::SecretKey;
use iroh::endpoint::Builder;
use iroh::endpoint::presets::Preset;
use iroh::protocol::Router;
use iroh::test_utils::DnsPkarrServer;
use iroh::tls::CaTlsConfig;
use iroh_gossip::net::Gossip;
use tempfile::TempDir;
use tokio::runtime::Runtime;

use rust_api::database::client::DbClient;
use rust_api::database::manager::DbManager;
use rust_api::network::{NwContact, NwCore, NwProfile};

pub struct TestNetwork {
    relay_map: RelayMap,
    dns_pkarr: DnsPkarrServer,
    _relay_server: iroh_relay::server::Server,

    _runtime: Runtime,
}

impl TestNetwork {
    pub fn runtime(&self) -> &Runtime {
        &self._runtime
    }

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
            _runtime: runtime,
        })
    }

    /// Spawns a node with a fresh temporary database and a freshly generated identity.
    pub fn spawn_node(&self, name: impl Into<String>) -> Result<TestNode> {
        let name = name.into();
        let secret_key = SecretKey::generate();
        let profile = NwProfile {
            contact: NwContact {
                name: name.clone(),
                endpoint_id: secret_key.public(),
            },
            secret_key,
        };

        let id = profile.contact.endpoint_id.to_string();
        let peer_span = tracing::info_span!("peer", name = %name.to_lowercase(), id = %id[..10]);
        let (db_manager, db_client, nw_core, temp_dir) = peer_span.in_scope(|| {
            let (db_manager, temp_dir) = spawn_test_db_manager()?;
            let db_client = db_manager.spawn_client_raw();
            db_client.set_profile(profile)?;

            let preset = LocalNetworkPreset {
                dns_pkarr: &self.dns_pkarr,
                relay_map: self.relay_map.clone(),
            };

            let nw_core = NwCore::spawn(db_client.clone(), preset)?;
            Ok::<_, anyhow::Error>((db_manager, db_client, nw_core, temp_dir))
        })?;

        Ok(TestNode {
            _db_manager: db_manager,
            db_client,
            nw_core,
            peer_span,
            _temp_dir: temp_dir,
        })
    }

    pub async fn spawn_gossip_peer(&self, secret_key: SecretKey) -> Result<TestGossipPeer> {
        let endpoint = Endpoint::builder(LocalNetworkPreset {
            dns_pkarr: &self.dns_pkarr,
            relay_map: self.relay_map.clone(),
        })
        .secret_key(secret_key)
        .alpns(vec![iroh_gossip::ALPN.to_vec()])
        .bind()
        .await?;

        let endpoint_id = endpoint.id();
        let gossip = Gossip::builder().spawn(endpoint.clone());
        let router = Router::builder(endpoint)
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();

        Ok(TestGossipPeer {
            endpoint_id,
            gossip,
            router,
        })
    }
}

pub struct TestGossipPeer {
    pub endpoint_id: iroh::EndpointId,
    pub gossip: Gossip,
    router: Router,
}

impl TestGossipPeer {
    pub async fn shutdown(self) -> Result<()> {
        self.gossip.shutdown().await?;
        self.router.shutdown().await?;
        Ok(())
    }
}

struct LocalNetworkPreset<'a> {
    dns_pkarr: &'a DnsPkarrServer,
    relay_map: RelayMap,
}

impl Preset for LocalNetworkPreset<'_> {
    fn apply(self, builder: Builder) -> Builder {
        builder
            .preset(iroh::endpoint::presets::Minimal)
            .preset(self.dns_pkarr.preset())
            .relay_mode(RelayMode::Custom(self.relay_map))
            .ca_tls_config(CaTlsConfig::insecure_skip_verify())
    }
}

pub struct TestNode {
    pub _db_manager: DbManager,
    pub db_client: DbClient,
    pub nw_core: NwCore,
    pub peer_span: tracing::Span,

    // Keeps the temporary database alive for the lifetime of the node.
    _temp_dir: TempDir,
}

fn spawn_test_db_manager() -> Result<(DbManager, TempDir)> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("app.db");

    let db_manager = DbManager::spawn(db_path.to_string_lossy().into_owned())?;

    Ok((db_manager, temp_dir))
}
