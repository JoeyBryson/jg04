use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use rust_api::ui::UiContact;

mod common;
use common::TestNetwork;

fn wait_until(
    name: &str,
    timeout: Duration,
    mut condition: impl FnMut() -> Result<bool>,
) -> Result<()> {
    let deadline = Instant::now() + timeout;

    loop {
        if condition()? {
            tracing::info!("[TEST] {name}: condition met");
            return Ok(());
        }

        if Instant::now() >= deadline {
            bail!("[TEST] {name}: condition was not met within {timeout:?}");
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn two_nodes_create_a_chat_and_exchange_messages() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();

    tracing::info!("[TEST] spawning network");
    let network = TestNetwork::spawn()?;

    tracing::info!("[TEST] spawning Alice");
    let alice = network.spawn_node("Alice")?;

    tracing::info!("[TEST] spawning Bob");
    let bob = network.spawn_node("Bob")?;

    let alice_contact = alice.db_client.get_nw_profile()?.contact;
    let bob_contact = bob.db_client.get_nw_profile()?.contact;

    tracing::info!(
        "[TEST] Alice endpoint: {}",
        alice_contact.endpoint_id
    );
    tracing::info!(
        "[TEST] Bob endpoint: {}",
        bob_contact.endpoint_id
    );

    // Each side needs to know the other as a contact before invites can be accepted.
    tracing::info!("[TEST] adding contacts");

    alice.db_client.add_nw_contact_sync(bob_contact.clone())?;
    bob.db_client.add_nw_contact_sync(alice_contact.clone())?;

    tracing::info!("[TEST] contacts added");

    let bob_ui_contact = UiContact {
        name: bob_contact.name.clone(),
        endpoint_id: bob_contact.endpoint_id.to_string(),
    };

    tracing::info!("[TEST] Alice creating chat");

    let chat_id = alice
        .nw_core
        .create_chat(vec![bob_ui_contact], Some("Alice & Bob".to_string()))?;

    tracing::info!("[TEST] created chat: {chat_id}");

    wait_until("Alice receives chat", Duration::from_secs(15), || {
        let chats = alice.db_client.get_nw_chats_sync()?;

        tracing::debug!(
            "[TEST] Alice chats: {:?}",
            chats.iter().map(|c| c.topic_id).collect::<Vec<_>>()
        );

        Ok(chats.iter().any(|c| c.topic_id.to_string() == chat_id))
    })?;

    wait_until("Bob receives chat", Duration::from_secs(15), || {
        let chats = bob.db_client.get_nw_chats_sync()?;

        tracing::debug!(
            "[TEST] Bob chats: {:?}",
            chats.iter().map(|c| c.topic_id).collect::<Vec<_>>()
        );

        Ok(chats.iter().any(|c| c.topic_id.to_string() == chat_id))
    })?;

    tracing::info!("[TEST] both nodes have chat");

    tracing::info!("[TEST] sending Alice message");

    alice
        .nw_core
        .send_message("hello from alice".to_string(), chat_id.clone())?;

    tracing::info!("[TEST] sending Bob message");

    bob.nw_core
        .send_message("hello from bob".to_string(), chat_id.clone())?;

    wait_until(
        "Alice receives both messages",
        Duration::from_secs(15),
        || {
            let messages = alice.db_client.get_ui_chat_messages(chat_id.clone())?;

            tracing::debug!(
                "[TEST] Alice messages: {:?}",
                messages
                    .iter()
                    .map(|m| &m.content)
                    .collect::<Vec<_>>()
            );

            Ok(messages.len() == 2)
        },
    )?;

    wait_until(
        "Bob receives both messages",
        Duration::from_secs(15),
        || {
            let messages = bob.db_client.get_ui_chat_messages(chat_id.clone())?;

            tracing::debug!(
                "[TEST] Bob messages: {:?}",
                messages
                    .iter()
                    .map(|m| &m.content)
                    .collect::<Vec<_>>()
            );

            Ok(messages.len() == 2)
        },
    )?;

    let alice_messages = alice.db_client.get_ui_chat_messages(chat_id.clone())?;
    let bob_messages = bob.db_client.get_ui_chat_messages(chat_id.clone())?;

    tracing::info!("[TEST] Alice messages: {:?}", alice_messages);
    tracing::info!("[TEST] Bob messages: {:?}", bob_messages);

    assert!(
        alice_messages
            .iter()
            .any(|m| m.content == "hello from alice")
    );
    assert!(
        alice_messages
            .iter()
            .any(|m| m.content == "hello from bob")
    );
    assert!(
        bob_messages
            .iter()
            .any(|m| m.content == "hello from alice")
    );
    assert!(
        bob_messages
            .iter()
            .any(|m| m.content == "hello from bob")
    );

    tracing::info!("[TEST] test completed successfully");

    Ok(())
}
