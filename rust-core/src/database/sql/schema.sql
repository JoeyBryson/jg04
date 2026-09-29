/*
    The database stores the local profile, contacts, chats, chat membership,
    and messages.

    The local user is also stored in `contacts`. This means `contacts` is the
    source of endpoint -> name mappings, and `chat_members` can represent
    every member of a chat, including the local user.


    Profile
    -------

    `user_profile` contains the local user's identity and secret key.

        user_profile
        ┌───────────────┐
        │ endpoint_id   │
        │ secret_key    │
        │ contact_name  │
        └───────┬───────┘
                │
                │ same endpoint
                ▼
        contacts
        ┌────────────────────┐
        │ endpoint_id        │
        │ contact_name       │
        ├────────────────────┤
        │ me                 │
        │ Alice              │
        │ Bob                │
        └────────────────────┘


    Chats and members
    -----------------

    A chat has a topic ID and optional name. Its members are stored
    separately in `chat_members`.

        chats                         chat_members
        ┌───────────────┐             ┌──────────────────────┐
        │ topic_id      │◄────────────┤ topic_id             │
        │ chat_name     │             │ endpoint_id          │
        └───────────────┘             │ status               │
                                      └──────────┬───────────┘
                                                 │
                                                 │ endpoint_id
                                                 ▼
                                      contacts
                                      ┌──────────────────────┐
                                      │ endpoint_id          │
                                      │ contact_name         │
                                      └──────────────────────┘

    Example:

        contacts
        ┌────────────┬──────────────┐
        │ endpoint   │ name         │
        ├────────────┼──────────────┤
        │ AAAA...    │ Me           │
        │ BBBB...    │ Alice        │
        │ CCCC...    │ Bob          │
        └────────────┴──────────────┘

        chat_members
        ┌────────────┬────────────┬────────┐
        │ topic      │ endpoint   │ status │
        ├────────────┼────────────┼────────┤
        │ Chat 1     │ AAAA...    │ Joined │
        │ Chat 1     │ BBBB...    │ Joined │
        │ Chat 1     │ CCCC...    │ Pending│
        └────────────┴────────────┴────────┘

    Messages
    --------

    Every message belongs to a chat and has an endpoint identifying its
    sender.

        chats                    messages
        ┌───────────────┐        ┌─────────────────────┐
        │ topic_id      │◄───────┤ topic_id            │
        └───────────────┘        │ endpoint_id         │
                                 │ content             │
                                 │ sent_at             │
                                 └──────────┬──────────┘
                                            │
                                            │ endpoint_id
                                            ▼
                                         contacts

    "Is this message mine?" is derived by comparing:

        messages.endpoint_id
                    ==
        user_profile.endpoint_id


    Relationships
    -------------

        user_profile
             │
             │ endpoint_id
             ▼
          contacts
             ▲
             │
             │ endpoint_id
             │
        chat_members ──────► chats
             │
             │ endpoint_id
             ▼
          contacts

        messages ──────────► chats
             │
             │ endpoint_id
             ▼
          contacts


    Deleting contacts
    -----------------

    Deleting a contact also deletes every chat containing that contact via
    the `delete_chats_for_deleted_contact` trigger.

    Deleting a chat cascades to:

        chats
          │
          ├──► chat_members
          └──► messages

    The foreign keys and trigger therefore ensure that chat membership and
    messages do not remain after their associated chat is deleted.
*/


PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS user_profile (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    endpoint_id BLOB NOT NULL,
    secret_key BLOB NOT NULL,
    contact_name TEXT NOT NULL
);

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
    status INTEGER NOT NULL DEFAULT 0 CHECK (status IN (0, 1)),

    PRIMARY KEY (topic_id, endpoint_id),

    FOREIGN KEY (topic_id)
    REFERENCES chats (topic_id)
    ON DELETE CASCADE,

    FOREIGN KEY (endpoint_id)
    REFERENCES contacts (endpoint_id)
);

CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    topic_id BLOB NOT NULL,
    endpoint_id BLOB NOT NULL,
    content TEXT NOT NULL,
    sent_at INTEGER NOT NULL,

    FOREIGN KEY (topic_id)
    REFERENCES chats (topic_id)
    ON DELETE CASCADE
);

CREATE TRIGGER IF NOT EXISTS delete_chats_for_deleted_contact
BEFORE DELETE ON contacts
FOR EACH ROW
BEGIN
    DELETE FROM chats
    WHERE chats.topic_id IN (
        SELECT chat_members.topic_id
        FROM chat_members
        WHERE chat_members.endpoint_id = old.endpoint_id
    );
END;
