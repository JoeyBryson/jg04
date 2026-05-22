PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS contacts (
    endpoint_id BLOB PRIMARY KEY,
    contact_name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS chats (
    topic_id BLOB PRIMARY KEY,
    chat_name TEXT
);

CREATE TABLE IF NOT EXISTS chat_members (
    topic_id BLOB NOT NULL,
    endpoint_id BLOB NOT NULL,
    PRIMARY KEY (topic_id, endpoint_id),
    FOREIGN KEY (topic_id) REFERENCES chats (topic_id) ON DELETE CASCADE,
    FOREIGN KEY (endpoint_id) REFERENCES contacts (endpoint_id)
);

CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    topic_id BLOB NOT NULL,
    is_me INTEGER NOT NULL,
    endpoint_id BLOB,
    content TEXT NOT NULL,
    sent_at INTEGER NOT NULL,
    FOREIGN KEY (topic_id) REFERENCES chats (topic_id) ON DELETE CASCADE,
    FOREIGN KEY (endpoint_id) REFERENCES contacts (endpoint_id)
);

CREATE TRIGGER IF NOT EXISTS delete_chats_for_deleted_contact
BEFORE DELETE ON contacts FOR EACH ROW BEGIN
    DELETE FROM chats
    WHERE chats.topic_id IN (
        SELECT chat_members.topic_id
        FROM chat_members
        WHERE chat_members.endpoint_id = old.endpoint_id
    );
END;
