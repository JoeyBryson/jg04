//! database writes
//! For database design read ../sql/schema.sql
//! Quick function summary (inputs -> output):
//! - `set_nw_profile(profile: NwProfile) -> Result<()>`
//! - `add_nw_message(message: NwMessage) -> Result<()>`
//! - `add_nw_contact(contact: NwContact) -> Result<()>`
//! - `add_ui_contact(contact: UiContact) -> Result<()>`
//! - `add_nw_chat(chat: NwChat) -> Result<()>`
//! - `update_contact_name(endpoint_id: EndpointId, name: String) -> Result<()>`
//! - `update_chat_members(topic_id: TopicId, members: Vec<NwChatMember>) -> Result<()>`
//! - `mark_nw_chat_member_joined(topic_id: TopicId, endpoint_id: EndpointId) -> Result<()>`
//! - `add_chat_ui(contacts: Vec<UiContact>, name: Option<String>) -> Result<String>`
//! - `reset_database() -> Result<()>`

use super::DbWriter;
use crate::network::{
    NwChat, NwChatMember, NwChatMemberStatus, NwContact, NwMessage, NwProfile,
};
use crate::ui::UiContact;
use anyhow::{Context, Result, anyhow, bail};
use iroh::EndpointId;
use iroh_gossip::proto::TopicId;
use rusqlite::OptionalExtension;

impl DbWriter {
    pub fn set_nw_profile(&self, profile: NwProfile) -> Result<()> {
        let endpoint_bytes = profile.contact.endpoint_id.as_bytes();
        let secret_bytes = profile.secret_key.to_bytes();
        let profile_name = profile.contact.name.clone();

        let inserted = self.conn.execute(
            "
            INSERT INTO user_profile (id, endpoint_id, secret_key, contact_name)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(id) DO NOTHING
            ",
            (
                1,
                endpoint_bytes,
                secret_bytes,
                profile_name.clone(),
            ),
        )?;

        if inserted == 0 {
            bail!("network profile is already set")
        }

        let contact_inserted = self.conn.execute(
            "INSERT INTO contacts (endpoint_id, contact_name)
             VALUES (?1, ?2)
             ON CONFLICT(endpoint_id) DO NOTHING",
            (endpoint_bytes, profile_name.clone()),
        )?;

        if contact_inserted == 1 {
            return Ok(());
        }

        let existing_contact_name = self.conn.query_row(
            "SELECT contact_name
             FROM contacts
             WHERE endpoint_id = ?1",
            [profile.contact.endpoint_id.as_slice()],
            |row| row.get::<_, String>(0),
        )?;

        if existing_contact_name == profile_name {
            return Ok(());
        }

        bail!("conflicting contact already exists for profile endpoint")
    }

    pub fn add_nw_message(&self, message: NwMessage) -> Result<()> {
        self.conn.execute(
            "INSERT INTO messages (topic_id, endpoint_id, content, sent_at)
             VALUES (?1, ?2, ?3, ?4)",
            (
                message.topic_id.as_bytes(),
                message.endpoint_id.as_bytes(),
                message.content,
                message.sent_at,
            ),
        )?;

        Ok(())
    }

    pub fn add_nw_contact(&self, contact: NwContact) -> Result<()> {
        let inserted = self.conn.execute(
            "INSERT INTO contacts (endpoint_id, contact_name)
             VALUES (?1, ?2)
             ON CONFLICT(endpoint_id) DO NOTHING",
            (contact.endpoint_id.as_slice(), contact.name.clone()),
        )?;

        if inserted == 1 {
            return Ok(());
        }

        let existing_name = self.conn.query_row(
            "SELECT contact_name
             FROM contacts
             WHERE endpoint_id = ?1",
            [contact.endpoint_id.as_slice()],
            |row| row.get::<_, String>(0),
        )?;

        if existing_name == contact.name {
            return Ok(());
        }

        bail!("conflicting contact already exists")
    }

    pub fn add_ui_contact(&self, contact: UiContact) -> Result<()> {
        let endpoint_id: EndpointId = contact.endpoint_id.parse()?;
        self.conn.execute(
            "INSERT INTO contacts (endpoint_id, contact_name)
             VALUES (?1, ?2)",
            (endpoint_id.as_slice(), contact.name),
        )?;

        Ok(())
    }

    pub fn add_nw_chat(&mut self, chat: NwChat) -> Result<()> {
        let transaction = self
            .conn
            .transaction()
            .context("failed to start transaction")?;

        let inserted_chat = transaction
            .execute(
                "INSERT INTO chats (topic_id, chat_name)
                 VALUES (?1, ?2)
                 ON CONFLICT(topic_id) DO NOTHING",
                (chat.topic_id.as_bytes(), chat.name.clone()),
            )
            .context("failed to insert chat")?;

        if inserted_chat == 0 {
            let existing_name = transaction
                .query_row(
                    "SELECT chat_name
                     FROM chats
                     WHERE topic_id = ?1",
                    [chat.topic_id.as_bytes()],
                    |row| row.get::<_, Option<String>>(0),
                )
                .optional()?
                .ok_or_else(|| anyhow!("chat exists but could not be loaded"))?;

            let existing_members = transaction
                .prepare(
                    "SELECT endpoint_id, status
                     FROM chat_members
                     WHERE topic_id = ?1
                     ORDER BY endpoint_id",
                )?
                .query_map([chat.topic_id.as_bytes()], |row| {
                    Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, i32>(1)?))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;

            let mut requested_members = chat
                .members
                .iter()
                .map(|member| {
                    (
                        member.contact.endpoint_id.as_bytes().to_vec(),
                        member.status as i32,
                    )
                })
                .collect::<Vec<_>>();

            requested_members.sort_unstable();

            if existing_name == chat.name && existing_members == requested_members {
                transaction.commit()?;
                return Ok(());
            }

            return Err(anyhow!("conflicting chat already exists"));
        }

        for member in chat.members {
            transaction
                .execute(
                    "INSERT INTO chat_members (topic_id, endpoint_id, status)
                     VALUES (?1, ?2, ?3)",
                    (
                        chat.topic_id.as_bytes(),
                        member.contact.endpoint_id.as_bytes(),
                        member.status as i32,
                    ),
                )
                .context("failed to insert chat member")?;
        }

        transaction
            .commit()
            .context("failed to commit chat transaction")?;

        Ok(())
    }

    pub fn update_contact_name(
        &mut self,
        endpoint_id: EndpointId,
        name: String,
    ) -> Result<()> {
        let transaction = self
            .conn
            .transaction()
            .context("failed to start transaction")?;

        let updated = transaction
            .execute(
                "UPDATE contacts
                 SET contact_name = ?1
                 WHERE endpoint_id = ?2",
                (name.clone(), endpoint_id.as_bytes()),
            )
            .context("failed to update contact name")?;

        if updated == 0 {
            bail!("contact does not exist")
        }

        transaction
            .execute(
                "UPDATE user_profile
                 SET contact_name = ?1
                 WHERE endpoint_id = ?2",
                (name, endpoint_id.as_bytes()),
            )
            .context("failed to update profile name")?;

        transaction
            .commit()
            .context("failed to commit contact name update")?;

        Ok(())
    }

    pub fn update_chat_members(
        &mut self,
        topic_id: TopicId,
        members: Vec<NwChatMember>,
    ) -> Result<()> {
        if members.is_empty() {
            bail!("chat members cannot be empty")
        }

        let transaction = self
            .conn
            .transaction()
            .context("failed to start transaction")?;

        let exists = transaction
            .query_row(
                "SELECT 1
                 FROM chats
                 WHERE topic_id = ?1",
                [topic_id.as_bytes()],
                |_row| Ok(()),
            )
            .optional()?;

        if exists.is_none() {
            bail!("chat does not exist")
        }

        transaction
            .execute(
                "DELETE FROM chat_members
                 WHERE topic_id = ?1",
                [topic_id.as_bytes()],
            )
            .context("failed to clear chat members")?;

        for member in members {
            transaction
                .execute(
                    "INSERT INTO chat_members (topic_id, endpoint_id, status)
                     VALUES (?1, ?2, ?3)",
                    (
                        topic_id.as_bytes(),
                        member.contact.endpoint_id.as_bytes(),
                        member.status as i32,
                    ),
                )
                .context("failed to insert updated chat member")?;
        }

        transaction
            .commit()
            .context("failed to commit chat member update")?;

        Ok(())
    }

    pub fn mark_nw_chat_member_joined(
        &self,
        topic_id: TopicId,
        endpoint_id: EndpointId,
    ) -> Result<()> {
        let status = self
            .conn
            .query_row(
                "SELECT status
                 FROM chat_members
                 WHERE topic_id = ?1 AND endpoint_id = ?2",
                (topic_id.as_bytes(), endpoint_id.as_bytes()),
                |row| row.get::<_, i32>(0),
            )
            .optional()?
            .ok_or_else(|| anyhow!("chat member does not exist"))?;

        if status != NwChatMemberStatus::Joined as i32 {
            self.conn.execute(
                "UPDATE chat_members
                 SET status = ?1
                 WHERE topic_id = ?2 AND endpoint_id = ?3",
                (
                    NwChatMemberStatus::Joined as i32,
                    topic_id.as_bytes(),
                    endpoint_id.as_bytes(),
                ),
            )?;
        }

        Ok(())
    }

    pub fn add_chat_ui(
        &mut self,
        contacts: Vec<UiContact>,
        name: Option<String>,
    ) -> Result<String> {
        let my_profile = self
            .conn
            .query_row(
                "SELECT endpoint_id, contact_name
                 FROM user_profile
                 WHERE id = 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, [u8; 32]>(0)?,
                        row.get::<_, String>(1)?,
                    ))
                },
            )?;

        let mut members = Vec::with_capacity(contacts.len() + 1);

        members.push(NwChatMember {
            contact: NwContact {
                name: my_profile.1,
                endpoint_id: EndpointId::from_bytes(&my_profile.0)
                    .context("failed to convert profile endpoint ID")?,
            },
            status: NwChatMemberStatus::Joined,
        });

        members.extend(
            contacts
                .into_iter()
                .map(|contact| -> Result<NwChatMember> {
                    Ok(NwChatMember {
                        contact: NwContact {
                            name: contact.name,
                            endpoint_id: contact
                                .endpoint_id
                                .parse()
                                .context("failed to convert UiContact to NwContact")?,
                        },
                        status: NwChatMemberStatus::Pending,
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        );

        let topic_id = TopicId::from_bytes(rand::random());

        self.add_nw_chat(NwChat {
            name,
            members,
            topic_id,
        })
        .context("failed to add NwChat")?;

        Ok(topic_id.to_string())
    }

    pub fn reset_database(&mut self) -> Result<()> {
        let transaction = self.conn.transaction()?;

        transaction.execute_batch(
            "
            DELETE FROM user_profile;
            DELETE FROM contacts;
            ",
        )?;

        transaction.commit()?;

        Ok(())
    }
}