use rust_api::setup_test_db;
use std::fs;

#[test]
fn setup_test_db_inspect_ui() {
    let _ = fs::remove_file("test.db");

    let entrypoint = setup_test_db();

    let chats = entrypoint.get_chats().expect("TST002: get chats");
    assert_eq!(chats.len(), 1);
    let chat = &chats[0];
    assert_eq!(chat.name.as_deref(), Some("Sample Chat"));
    assert_eq!(chat.members.len(), 2);
    assert!(chat.members.iter().any(|c| c.name == "Alice"));
    assert!(chat.members.iter().any(|c| c.name == "Bob"));

    let retrieved_chat = entrypoint.get_chat(chat.topic_id.clone()).expect("TST003: get chat");
    assert_eq!(retrieved_chat.topic_id, chat.topic_id);
    assert_eq!(retrieved_chat.members.len(), 2);

    let messages = entrypoint.get_messages().expect("TST004: get messages");
    assert_eq!(messages.len(), 30);

    let chat_messages = entrypoint.get_chat_messages(chat.topic_id.clone()).expect("TST005: get chat messages");
    assert_eq!(chat_messages.len(), 30);
    assert_eq!(chat_messages[0].content, "Sample message 1");
    assert_eq!(chat_messages[29].content, "Sample message 30");

    let members = entrypoint.get_chat_members(chat.topic_id.clone()).expect("TST006: get chat members");
    assert_eq!(members.len(), 2);

    drop(entrypoint);
    let _ = fs::remove_file("test.db");
}
