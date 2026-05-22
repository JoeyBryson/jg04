package com.example.jg04.data

import uniffi.rust_api.Chat
import uniffi.rust_api.Contact
import uniffi.rust_api.DbEntrypoint

public data class UIContact(
    val name: String,
    val endpointId: ByteArray
)

public data class UIChat(
    val name: String?,
    val members: List<UIContact>,
    val topicId: ByteArray
)

public data class UIMessage(
    val fromMe: Boolean,
    val contact: UIContact?,
    val content: String,
    val sentAt: Long
)

public data class UIChatWithMessages(
    val chat: UIChat,
    val messages: List<UIMessage>
)

fun buildUIContact(contact: Contact): UIContact {
    return UIContact(
        name = contact.name,
        endpointId = contact.endpointId
    )
}

fun buildUIChat(chat: Chat): UIChat {
    return UIChat(
        name = chat.name,
        members = chat.members.map { buildUIContact(it) },
        topicId = chat.topicId
    )
}

fun BuildUIChatsWithMessages(dB: DbEntrypoint): List<UIChatWithMessages> {

    val dBchats = dB.getChats()

    val result = mutableListOf<UIChatWithMessages>()

    for (dBchat in dBchats) {

        val dBmessages = dB.getChatMessages(dBchat.topicId)

        val messages = mutableListOf<UIMessage>()

        for (dBmessage in dBmessages) {

            val contact =
                if (dBmessage.fromMe) {
                    null
                } else {
                    val endpoint = dBmessage.endpointId

                    val match = dBchat.members.firstOrNull { member ->
                        member.endpointId.contentEquals(endpoint)
                    }

                    match?.let { buildUIContact(it) }
                }

            messages.add(
                UIMessage(
                    fromMe = dBmessage.fromMe,
                    contact = contact,
                    content = dBmessage.content,
                    sentAt = dBmessage.sentAt
                )
            )
        }

        result.add(
            UIChatWithMessages(
                chat = buildUIChat(dBchat),
                messages = messages
            )
        )
    }

    return result
}