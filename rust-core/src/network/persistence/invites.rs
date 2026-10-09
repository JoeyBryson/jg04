use anyhow::Result;
use iroh::EndpointId;
use iroh_gossip::proto::TopicId;

use crate::database::client::DbClient;
use crate::network::{NwChat, NwChatMemberStatus, NwContact};

#[derive(Clone)]
pub(in crate::network) struct InviteStore {
    db: DbClient,
}

#[derive(Clone, Debug)]
pub(in crate::network) struct PendingChatInvite {
    pub(in crate::network) chat: NwChat,
    pub(in crate::network) contact: NwContact,
}

impl InviteStore {
    pub(in crate::network) fn new(db: DbClient) -> Self {
        Self { db }
    }

    pub(in crate::network) async fn pending_invites(&self) -> Result<Vec<PendingChatInvite>> {
        let local_endpoint_id = self.db.get_nw_profile_async().await?.contact.endpoint_id;
        let chats = self.db.get_nw_chats().await?;
        let mut invites = Vec::new();

        for chat in chats {
            for member in chat
                .members
                .iter()
                .filter(|member| {
                    member.status != NwChatMemberStatus::Joined
                        && member.contact.endpoint_id != local_endpoint_id
                })
            {
                invites.push(PendingChatInvite {
                    chat: chat.clone(),
                    contact: member.contact.clone(),
                });
            }
        }

        Ok(invites)
    }

    pub(in crate::network) async fn mark_joined(
        &self,
        topic_id: TopicId,
        member: EndpointId,
    ) -> Result<()> {
        self.db
            .mark_nw_chat_member_joined(topic_id, member)
            .await?;

        Ok(())
    }
}