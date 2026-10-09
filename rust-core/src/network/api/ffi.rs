use std::sync::Arc;

use iroh::endpoint::presets;
use iroh_gossip::proto::TopicId;

use crate::database::client::DbClient;
use crate::ffi_error::FfiError;
use crate::network::NwChatMemberStatus::Pending;
use crate::network::{NwChatMember, NwContact};
use crate::ui::UiContact;

use super::core::NwCore;

/// UniFFI-facing adapter that converts UI values and IDs to internal network
/// types before delegating operations to [`NwCore`].
#[derive(uniffi::Object)]
pub struct NwCoreInterface {
    core: NwCore,
}

#[uniffi::export]
impl NwCoreInterface {
    #[uniffi::constructor]
    pub fn spawn(db_client: Arc<DbClient>) -> Result<Self, FfiError> {
        let db_client = Arc::unwrap_or_clone(db_client);
        let core = NwCore::spawn(db_client, presets::N0).map_err(FfiError::from)?;
        Ok(Self { core })
    }

    pub fn create_chat(
        &self,
        contacts: Vec<UiContact>,
        name: Option<String>,
    ) -> Result<String, FfiError> {
        let members = contacts
            .into_iter()
            .map(NwContact::from)
            .map(|contact| NwChatMember {
                contact,
                status: Pending,
            })
            .collect::<Vec<NwChatMember>>();

        self.core.create_chat(members, name)
    }

    pub fn send_message(&self, content: String, chat_id: String) -> Result<(), FfiError> {
        let mut topic_id_bytes = [0u8; 32];
        hex::decode_to_slice(&chat_id, &mut topic_id_bytes)
            .map_err(anyhow::Error::from)?;

        let topic_id = TopicId::from_bytes(topic_id_bytes);
        self.core.send_message(content, topic_id)
    }
}

impl From<UiContact> for NwContact {
    fn from(contact: UiContact) -> Self {
        Self {
            name: contact.name,
            endpoint_id: contact.endpoint_id.parse().unwrap(),
        }
    }
}
