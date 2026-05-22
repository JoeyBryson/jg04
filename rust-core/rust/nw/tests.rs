use super::*;
use std::collections::BTreeMap;
use std::fs;
use std::thread;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

fn remove_test_db(db_path: &str) {
    let _ = fs::remove_file(db_path);
}

fn spawn_db_worker(db_path: &'static str) -> (mpsc::Sender<db::Command>, thread::JoinHandle<()>, Arc<Mutex<Option<String>>>) {
    let (db_tx, db_rx) = mpsc::channel::<db::Command>(32);
    let db_panic_msg = Arc::new(Mutex::new(None));
    let db_panic_msg_clone = db_panic_msg.clone();
    let handle = thread::spawn(move || {
        db::worker(db_rx, db_path);
    });
    (db_tx, handle, db_panic_msg)
}

#[test]
fn add_and_read_chats() {
    let db_path = "test_chats.db";
    remove_test_db(db_path);
    let (db_tx, handle, db_panic_msg) = spawn_db_worker(db_path);

    let topic_id = vec![1, 1, 1, 1];
    let endpoint_id = vec![2, 2, 2, 2];
    let chat_name = Some("room-1".to_string());
    let contact_name = "alice".to_string();

    Runtime::new().expect("TST001: runtime init").block_on(async {
        let entrypoint = create_async_db_entrypoint(db_tx.clone(), db_panic_msg.clone());
        
        entrypoint.add_contact(
            Contact {
                name: contact_name.clone(),
                endpoint_id: endpoint_id.clone(),
            },
        )
        .await
        .unwrap();

        entrypoint.add_chat(
            Chat {
                name: chat_name.clone(),
                members: vec![Contact {
                    name: contact_name.clone(),
                    endpoint_id: endpoint_id.clone(),
                }],
                topic_id: topic_id.clone(),
            },
        )
        .await
        .unwrap();

        let chat = entrypoint.get_chat(topic_id.clone()).await.unwrap();
        assert_eq!(chat.topic_id, topic_id);
        assert_eq!(chat.name, chat_name);
        assert_eq!(chat.members.len(), 1);
        assert_eq!(chat.members[0].name, contact_name);
        assert_eq!(chat.members[0].endpoint_id, endpoint_id);

        let members = entrypoint.get_chat_members(topic_id.clone()).await.unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].name, "alice");
        assert_eq!(members[0].endpoint_id, vec![2, 2, 2, 2]);

        let chats = entrypoint.get_chats().await.unwrap();
        assert_eq!(chats.len(), 1);
        assert_eq!(chats[0].topic_id, topic_id);
    });

    drop(db_tx);
    handle.join().unwrap();
    remove_test_db(db_path);
}

#[test]
fn add_and_read_messages() {
    let db_path = "test_messages.db";
    remove_test_db(db_path);
    let (db_tx, handle, db_panic_msg) = spawn_db_worker(db_path);

    let topic_id = vec![9, 9, 9, 9];
    let endpoint_id = vec![3, 3, 3, 3];

    Runtime::new().unwrap().block_on(async {
        let entrypoint = create_async_db_entrypoint(db_tx.clone(), db_panic_msg.clone());
        
        entrypoint.add_contact(
            Contact {
                name: "bob".to_string(),
                endpoint_id: endpoint_id.clone(),
            },
        )
        .await
        .unwrap();

        entrypoint.add_chat(
            Chat {
                name: Some("room-2".to_string()),
                members: vec![Contact {
                    name: "bob".to_string(),
                    endpoint_id: endpoint_id.clone(),
                }],
                topic_id: topic_id.clone(),
            },
        )
        .await
        .unwrap();

        entrypoint.add_message(
            Message {
                topic_id: topic_id.clone(),
                from_me: true,
                endpoint_id: Some(endpoint_id.clone()),
                content: "hello".to_string(),
                sent_at: 100,
            },
        )
        .await
        .unwrap();

        entrypoint.add_message(
            Message {
                topic_id: topic_id.clone(),
                from_me: false,
                endpoint_id: Some(endpoint_id.clone()),
                content: "world".to_string(),
                sent_at: 101,
            },
        )
        .await
        .unwrap();

        let chat_messages = entrypoint.get_chat_messages(topic_id.clone()).await.unwrap();
        assert_eq!(chat_messages.len(), 2);
        assert_eq!(chat_messages[0].topic_id, topic_id);
        assert_eq!(chat_messages[0].content, "hello");
        assert!(chat_messages[0].from_me);
        assert_eq!(chat_messages[1].content, "world");
        assert!(!chat_messages[1].from_me);

        let messages = entrypoint.get_messages().await.unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].endpoint_id, Some(endpoint_id.clone()));
        assert_eq!(messages[1].endpoint_id, Some(endpoint_id));
    });

    drop(db_tx);
    handle.join().unwrap();
    remove_test_db(db_path);
}

#[test]
fn large_mixed_order_dataset_pretty_print() {
    let db_path = "test_large_mixed.db";
    remove_test_db(db_path);
    let (db_tx, handle, db_panic_msg) = spawn_db_worker(db_path);

    Runtime::new().unwrap().block_on(async {
        let entrypoint = create_async_db_entrypoint(db_tx.clone(), db_panic_msg.clone());
        
        let contacts = vec![
            Contact { name: "alice".to_string(), endpoint_id: vec![1, 0, 0, 1] },
            Contact { name: "bob".to_string(), endpoint_id: vec![1, 0, 0, 2] },
            Contact { name: "carol".to_string(), endpoint_id: vec![1, 0, 0, 3] },
            Contact { name: "dave".to_string(), endpoint_id: vec![1, 0, 0, 4] },
            Contact { name: "eve".to_string(), endpoint_id: vec![1, 0, 0, 5] },
            Contact { name: "frank".to_string(), endpoint_id: vec![1, 0, 0, 6] },
            Contact { name: "grace".to_string(), endpoint_id: vec![1, 0, 0, 7] },
            Contact { name: "heidi".to_string(), endpoint_id: vec![1, 0, 0, 8] },
        ];

        for c in contacts.iter().cloned() {
            entrypoint.add_contact(c).await.unwrap();
        }

        let chat_general_topic = vec![7, 7, 7, 1];
        let chat_ops_topic = vec![7, 7, 7, 2];
        let chat_random_topic = vec![7, 7, 7, 3];

        entrypoint.add_chat(
            Chat {
                name: Some("ops".to_string()),
                topic_id: chat_ops_topic.clone(),
                members: vec![
                    contacts[2].clone(),
                    contacts[3].clone(),
                    contacts[4].clone(),
                ],
            },
        )
        .await
        .unwrap();

        entrypoint.add_chat(
            Chat {
                name: Some("general".to_string()),
                topic_id: chat_general_topic.clone(),
                members: vec![
                    contacts[0].clone(),
                    contacts[1].clone(),
                    contacts[2].clone(),
                    contacts[3].clone(),
                ],
            },
        )
        .await
        .unwrap();

        entrypoint.add_chat(
            Chat {
                name: Some("random".to_string()),
                topic_id: chat_random_topic.clone(),
                members: vec![
                    contacts[4].clone(),
                    contacts[5].clone(),
                    contacts[6].clone(),
                    contacts[7].clone(),
                ],
            },
        )
        .await
        .unwrap();

        let mut expected_by_topic: BTreeMap<Vec<u8>, usize> = BTreeMap::new();
        expected_by_topic.insert(chat_general_topic.clone(), 0);
        expected_by_topic.insert(chat_ops_topic.clone(), 0);
        expected_by_topic.insert(chat_random_topic.clone(), 0);

        for i in 0..72usize {
            let topic_id = if i % 3 == 0 {
                chat_general_topic.clone()
            } else if i % 3 == 1 {
                chat_ops_topic.clone()
            } else {
                chat_random_topic.clone()
            };

            let endpoint = if topic_id == chat_general_topic {
                contacts[i % 4].endpoint_id.clone()
            } else if topic_id == chat_ops_topic {
                contacts[2 + (i % 3)].endpoint_id.clone()
            } else {
                contacts[4 + (i % 4)].endpoint_id.clone()
            };

            entrypoint.add_message(
                Message {
                    topic_id: topic_id.clone(),
                    from_me: i % 2 == 0,
                    endpoint_id: Some(endpoint),
                    content: format!("message-{i:03}"),
                    sent_at: 2_000_000 + i as i64,
                },
            )
            .await
            .unwrap();

            *expected_by_topic.get_mut(&topic_id).unwrap() += 1;
        }

        let chats = entrypoint.get_chats().await.unwrap();
        let all_messages = entrypoint.get_messages().await.unwrap();
        let general_messages = entrypoint.get_chat_messages(chat_general_topic.clone()).await.unwrap();
        let ops_messages = entrypoint.get_chat_messages(chat_ops_topic.clone()).await.unwrap();
        let random_messages = entrypoint.get_chat_messages(chat_random_topic.clone()).await.unwrap();
        let general_chat = entrypoint.get_chat(chat_general_topic.clone()).await.unwrap();
        let ops_chat = entrypoint.get_chat(chat_ops_topic.clone()).await.unwrap();
        let random_chat = entrypoint.get_chat(chat_random_topic.clone()).await.unwrap();

        assert_eq!(chats.len(), 3);
        assert_eq!(all_messages.len(), 72);
        assert_eq!(general_messages.len(), *expected_by_topic.get(&chat_general_topic).unwrap());
        assert_eq!(ops_messages.len(), *expected_by_topic.get(&chat_ops_topic).unwrap());
        assert_eq!(random_messages.len(), *expected_by_topic.get(&chat_random_topic).unwrap());
        assert_eq!(general_chat.members.len(), 4);
        assert_eq!(ops_chat.members.len(), 3);
        assert_eq!(random_chat.members.len(), 4);

        let mut by_topic_from_all: BTreeMap<Vec<u8>, usize> = BTreeMap::new();
        for m in &all_messages {
            *by_topic_from_all.entry(m.topic_id.clone()).or_insert(0) += 1;
        }
        assert_eq!(by_topic_from_all, expected_by_topic);

        println!("\n=== LARGE DATASET SNAPSHOT ===");
        println!("chats = {:#?}", chats);
        println!("general_chat = {:#?}", general_chat);
        println!("ops_chat = {:#?}", ops_chat);
        println!("random_chat = {:#?}", random_chat);
        println!("general_messages = {:#?}", general_messages);
        println!("ops_messages = {:#?}", ops_messages);
        println!("random_messages = {:#?}", random_messages);
        println!("all_messages = {:#?}", all_messages);
        println!("message_counts_by_topic = {:#?}", by_topic_from_all);
    });

    drop(db_tx);
    handle.join().unwrap();
    remove_test_db(db_path);
}

#[test]
fn performance_smoke_analysis() {
    let db_path = "test_perf.db";
    remove_test_db(db_path);
    let (db_tx, handle, db_panic_msg) = spawn_db_worker(db_path);

    Runtime::new().unwrap().block_on(async {
        let entrypoint = create_async_db_entrypoint(db_tx.clone(), db_panic_msg.clone());
        
        let contacts: Vec<Contact> = (0..50)
            .map(|i| Contact {
                name: format!("user-{i:03}"),
                endpoint_id: vec![9, 9, (i / 256) as u8, (i % 256) as u8],
            })
            .collect();

        let chats: Vec<Chat> = (0..10)
            .map(|i| {
                let base = i * 5;
                Chat {
                    name: Some(format!("chat-{i:02}")),
                    topic_id: vec![8, 8, 0, i as u8],
                    members: vec![
                        contacts[base].clone(),
                        contacts[base + 1].clone(),
                        contacts[base + 2].clone(),
                        contacts[base + 3].clone(),
                        contacts[base + 4].clone(),
                    ],
                }
            })
            .collect();

        let t0 = Instant::now();
        for c in contacts.iter().cloned() {
            entrypoint.add_contact(c).await.unwrap();
        }
        let add_contacts_ms = t0.elapsed().as_millis();

        let t1 = Instant::now();
        for c in chats.iter().cloned() {
            entrypoint.add_chat(c).await.unwrap();
        }
        let add_chats_ms = t1.elapsed().as_millis();

        let mut total_messages = 0usize;
        let t2 = Instant::now();
        for i in 0..5000usize {
            let topic_id = vec![8, 8, 0, (i % 10) as u8];
            let endpoint_id = Some(vec![9, 9, ((i % 50) / 256) as u8, ((i % 50) % 256) as u8]);
            entrypoint.add_message(
                Message {
                    topic_id,
                    from_me: i % 2 == 0,
                    endpoint_id,
                    content: format!("perf-msg-{i}"),
                    sent_at: 3_000_000 + i as i64,
                },
            )
            .await
            .unwrap();
            total_messages += 1;
        }
        let add_messages_ms = t2.elapsed().as_millis();

        let t3 = Instant::now();
        let all_chats = entrypoint.get_chats().await.unwrap();
        let all_messages = entrypoint.get_messages().await.unwrap();
        let one_chat = entrypoint.get_chat(vec![8, 8, 0, 0]).await.unwrap();
        let one_chat_messages = entrypoint.get_chat_messages(vec![8, 8, 0, 0]).await.unwrap();
        let read_ms = t3.elapsed().as_millis();

        assert_eq!(all_chats.len(), 10);
        assert_eq!(all_messages.len(), total_messages);
        assert_eq!(one_chat.members.len(), 5);
        assert!(!one_chat_messages.is_empty());

        let msgs_per_sec = if add_messages_ms == 0 {
            0.0
        } else {
            (total_messages as f64) / (add_messages_ms as f64 / 1000.0)
        };

        println!("\n=== PERFORMANCE SMOKE ANALYSIS ===");
        println!("contacts inserted: {} in {} ms", contacts.len(), add_contacts_ms);
        println!("chats inserted: {} in {} ms", chats.len(), add_chats_ms);
        println!("messages inserted: {} in {} ms", total_messages, add_messages_ms);
        println!("read pass (get_chats + get_messages + get_chat + get_chat_messages): {} ms", read_ms);
        println!("insert throughput: {:.2} messages/sec", msgs_per_sec);
    });

    drop(db_tx);
    handle.join().unwrap();
    remove_test_db(db_path);
}
