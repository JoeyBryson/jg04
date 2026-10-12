use std::sync::Once;
use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use iroh_gossip::proto::TopicId;
use rust_api::network::{NwChatMember, NwChatMemberStatus};
use rust_api::notifications::{UiEvent, UiEventListener, register_ui_event_listener};

mod common;
use common::TestNetwork;

struct StubUiEventListener;

impl UiEventListener for StubUiEventListener {
    fn on_event(&self, _event: UiEvent) {}
}

fn ensure_ui_listener_registered() {
    static UI_LISTENER_INIT: Once = Once::new();

    UI_LISTENER_INIT.call_once(|| {
        let _ = register_ui_event_listener(Box::new(StubUiEventListener));
    });
}

fn wait_until(
    name: &str,
    timeout: Duration,
    mut condition: impl FnMut() -> Result<bool>,
) -> Result<()> {
    let deadline = Instant::now() + timeout;

    loop {
        if condition()? {
            tracing::info!(stage = name, "condition met");
            return Ok(());
        }

        if Instant::now() >= deadline {
            bail!("{name} not reached within {timeout:?}");
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}

fn parse_topic_id(chat_id: &str) -> Result<TopicId> {
    let mut topic_id_bytes = [0u8; 32];
    hex::decode_to_slice(chat_id, &mut topic_id_bytes)?;
    Ok(TopicId::from_bytes(topic_id_bytes))
}

fn has_both_test_messages(messages: &[rust_api::ui::UiMessage]) -> bool {
    let has_alice = messages.iter().any(|m| m.content == "hello from alice");
    let has_bob = messages.iter().any(|m| m.content == "hello from bob");
    has_alice && has_bob
}

#[test]
fn two_nodes_create_a_chat_and_exchange_messages() -> Result<()> {
    ensure_ui_listener_registered();

    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    tracing_subscriber::EnvFilter::new(
                        "debug,iroh=off,iroh_gossip=off,iroh_relay=off,portmapper=off,igd_next=off,reqwest=off,hyper=off,hyper_util=off,rustls=off,hickory_net=off,hickory_resolver=off,noq_proto=off",
                    )
                }),
        )
        .try_init();

    tracing::info!("starting two-node chat exchange");

    tracing::debug!("starting local test network");
    let network = TestNetwork::spawn()?;

    let alice = network.spawn_node("Alice")?;

    let bob = network.spawn_node("Bob")?;
    tracing::info!("both peers are online");

    let alice_contact = alice.db_client.get_nw_profile_sync()?.contact;
    let bob_contact = bob.db_client.get_nw_profile_sync()?.contact;

    // Each side needs to know the other as a contact before invites can be accepted.
    alice.db_client.add_nw_contact_sync(bob_contact.clone())?;
    bob.db_client.add_nw_contact_sync(alice_contact.clone())?;
    tracing::debug!("peer contacts exchanged");

    let bob_member = NwChatMember {
        contact: bob_contact.clone(),
        status: NwChatMemberStatus::Pending,
    };

    let chat_id = alice
        .nw_core
        .create_chat(vec![bob_member], Some("Alice & Bob".to_string()))?;

    let topic_id = parse_topic_id(&chat_id)?;

    tracing::info!(topic_id = %&chat_id[..10], "chat created; waiting for both peers");

    alice.peer_span.in_scope(|| {
        wait_until("received chat", Duration::from_secs(15), || {
            let chats = alice.db_client.get_nw_chats_sync()?;
            Ok(chats.iter().any(|c| c.topic_id.to_string() == chat_id))
        })
    })?;

    bob.peer_span.in_scope(|| {
        wait_until("received chat", Duration::from_secs(15), || {
            let chats = bob.db_client.get_nw_chats_sync()?;
            Ok(chats.iter().any(|c| c.topic_id.to_string() == chat_id))
        })
    })?;

    tracing::info!(topic_id = %&chat_id[..10], "chat visible to both peers; sending messages");
    alice
        .nw_core
        .send_message("hello from alice".to_string(), topic_id)?;

    bob.nw_core
        .send_message("hello from bob".to_string(), topic_id)?;

    // Give gossip a brief chance to establish neighbors before we evaluate delivery.
    std::thread::sleep(Duration::from_millis(300));

    let first_attempt = wait_until(
        "Alice receives both messages",
        Duration::from_secs(5),
        || {
            let messages = alice.db_client.get_ui_chat_messages(chat_id.clone())?;

            Ok(has_both_test_messages(&messages))
        },
    );

    if let Err(error) = first_attempt {
        tracing::warn!(error = %error, "message delivery delayed; retrying once");

        alice
            .nw_core
            .send_message("hello from alice".to_string(), topic_id)?;
        bob.nw_core
            .send_message("hello from bob".to_string(), topic_id)?;
    }

    alice.peer_span.in_scope(|| {
        wait_until("received both messages", Duration::from_secs(15), || {
            let messages = alice.db_client.get_ui_chat_messages(chat_id.clone())?;
            Ok(has_both_test_messages(&messages))
        })
    })?;

    bob.peer_span.in_scope(|| {
        wait_until("received both messages", Duration::from_secs(15), || {
            let messages = bob.db_client.get_ui_chat_messages(chat_id.clone())?;
            Ok(has_both_test_messages(&messages))
        })
    })?;

    let alice_messages = alice.db_client.get_ui_chat_messages(chat_id.clone())?;
    let bob_messages = bob.db_client.get_ui_chat_messages(chat_id.clone())?;

    assert!(
        alice_messages
            .iter()
            .any(|m| m.content == "hello from alice")
    );
    assert!(alice_messages.iter().any(|m| m.content == "hello from bob"));
    assert!(bob_messages.iter().any(|m| m.content == "hello from alice"));
    assert!(bob_messages.iter().any(|m| m.content == "hello from bob"));

    tracing::info!("both peers received both messages");

    Ok(())
}
