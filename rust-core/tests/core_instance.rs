use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use rust_api::harness::TestNetwork;
use rust_api::ui::UiContact;

fn wait_until(timeout: Duration, mut condition: impl FnMut() -> Result<bool>) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        if condition()? {
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!("condition was not met within {timeout:?}");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn two_nodes_create_a_chat_and_exchange_messages() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .try_init();

    let network = TestNetwork::spawn()?;
    let alice = network.spawn_node("Alice")?;
    let bob = network.spawn_node("Bob")?;

    let alice_contact = alice.db_client.get_nw_profile()?.contact;
    let bob_contact = bob.db_client.get_nw_profile()?.contact;

    // Each side needs to know the other as a contact before invites can be accepted.
    alice.db_client.add_nw_contact_sync(bob_contact.clone())?;
    bob.db_client.add_nw_contact_sync(alice_contact.clone())?;

    let bob_ui_contact = UiContact {
        name: bob_contact.name.clone(),
        endpoint_id: bob_contact.endpoint_id.to_string(),
    };

    let chat_id = alice
        .nw_core
        .create_chat(vec![bob_ui_contact], Some("Alice & Bob".to_string()))?;

    // Wait for the invite to be delivered, accepted, and the chat session to come up on both sides.
    wait_until(Duration::from_secs(15), || {
        Ok(alice
            .db_client
            .get_nw_chats_sync()?
            .iter()
            .any(|c| c.topic_id.to_string() == chat_id))
    })?;

    wait_until(Duration::from_secs(15), || {
        Ok(bob
            .db_client
            .get_nw_chats_sync()?
            .iter()
            .any(|c| c.topic_id.to_string() == chat_id))
    })?;

    alice
        .nw_core
        .send_message("hello from alice".to_string(), chat_id.clone())?;
    bob.nw_core
        .send_message("hello from bob".to_string(), chat_id.clone())?;

    wait_until(Duration::from_secs(15), || {
        let messages = alice.db_client.get_ui_chat_messages(chat_id.clone())?;
        Ok(messages.len() == 2)
    })?;

    wait_until(Duration::from_secs(15), || {
        let messages = bob.db_client.get_ui_chat_messages(chat_id.clone())?;
        Ok(messages.len() == 2)
    })?;

    let alice_messages = alice.db_client.get_ui_chat_messages(chat_id.clone())?;
    let bob_messages = bob.db_client.get_ui_chat_messages(chat_id.clone())?;

    assert!(
        alice_messages
            .iter()
            .any(|m| m.content == "hello from alice")
    );
    assert!(alice_messages.iter().any(|m| m.content == "hello from bob"));
    assert!(
        bob_messages
            .iter()
            .any(|m| m.content == "hello from alice")
    );
    assert!(bob_messages.iter().any(|m| m.content == "hello from bob"));

    Ok(())
}
