use super::DbWriter;
use crate::network::{NwChat, NwChatMember, NwChatMemberStatus, NwContact, NwMessage, NwProfile};
use crate::ui::UiContact;
use anyhow::{Context, Result, anyhow, bail};
use iroh::EndpointId;
use iroh_gossip::proto::TopicId;
use rusqlite::OptionalExtension;

impl DbWriter {
    pub fn set_nw_profile(&self, profile: NwProfile) -> Result<()> {
        let existing = self
            .conn
            .query_row(
                "SELECT secret_key, contact_name FROM user_profile WHERE id = 1",
                [],
                |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;

        match existing {
            Some((secret_key, contact_name))
                if secret_key == profile.secret_key.to_bytes()
                    && contact_name == profile.contact.name =>
            {
                Ok(())
            }
            Some(_) => bail!("conflicting network profile already exists"),
            None => {
                self.conn.execute(
                    "
                    INSERT INTO user_profile (id, secret_key, contact_name)
                    VALUES (?1, ?2, ?3)
                    ",
                    (1, profile.secret_key.to_bytes(), profile.contact.name),
                )?;

                Ok(())
            }
        }
    }

    pub fn add_nw_message(&self, message: NwMessage) -> Result<()> {
        self.conn.execute(
            "INSERT INTO messages (topic_id, is_me, endpoint_id, content, sent_at)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                message.topic_id.as_bytes(),
                message.from_me as i32,
                message.endpoint_id.as_deref(),
                message.content,
                message.sent_at,
            ),
        )?;

        Ok(())
    }

    pub fn add_nw_contact(&self, contact: NwContact) -> Result<()> {
        let existing = self
            .conn
            .query_row(
                "SELECT contact_name FROM contacts WHERE endpoint_id = ?1",
                [contact.endpoint_id.as_slice()],
                |row| row.get::<_, String>(0),
            )
            .optional()?;

        match existing {
            Some(name) if name == contact.name => Ok(()),
            Some(_) => bail!("conflicting contact already exists"),
            None => {
                self.conn.execute(
                    "INSERT INTO contacts (endpoint_id, contact_name)
                    VALUES (?1, ?2)",
                    (contact.endpoint_id.as_slice(), contact.name),
                )?;

                Ok(())
            }
        }
    }

    pub fn add_ui_contact(&self, contact: UiContact) -> Result<()> {
        let endpoint_id = contact.endpoint_id.parse()?;

        self.add_nw_contact(NwContact {
            name: contact.name,
            endpoint_id,
        })
    }

    pub fn add_nw_chat(&mut self, chat: NwChat) -> Result<()> {
        let transaction = self
            .conn
            .transaction()
            .context("failed to start transaction")?;

        let existing_name = transaction
            .query_row(
                "SELECT chat_name FROM chats WHERE topic_id = ?1",
                [chat.topic_id.as_bytes()],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()?;

        if let Some(existing_name) = existing_name {
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

        transaction
            .execute(
                "INSERT INTO chats (topic_id, chat_name)
                VALUES (?1, ?2)",
                (chat.topic_id.as_bytes(), chat.name),
            )
            .context("failed to insert chat")?;

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

    pub fn mark_nw_chat_member_joined(
        &self,
        topic_id: TopicId,
        endpoint_id: EndpointId,
    ) -> Result<()> {
        let status = self
            .conn
            .query_row(
                "SELECT status FROM chat_members
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
        let members = contacts
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
            .collect::<Result<Vec<_>>>()?;

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
