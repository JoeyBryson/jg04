package com.example.jg04.data

import uniffi.rust_api.UiContact
import uniffi.rust_api.UiChat
import uniffi.rust_api.UiChatWithMessages
import uniffi.rust_api.UiMessage
import uniffi.rust_api.UiSender
//
//public sealed class UiSender {
//
//
//    public data object Me : UiSender()
//
//
//    public data class Other(
//        val v1: UiContact,
//    ) : UiSender() {
//    }
//
//}
//
//public data class UiChatWithMessages (
//    var `chat`: UiChat,
//    var `messages`: List<UiMessage>
//)
//
//public data class UiChat (
//    var `name`: kotlin.String?,
//    var `members`: List<UiContact>,
//    var `topicId`: kotlin.ByteArray,
//    var `lastMessage`: UiMessage
//)
//
//public data class UiChat (
//    var `name`: kotlin.String?,
//    var `members`: List<UiContact>,
//    var `topicId`: kotlin.ByteArray,
//    var `lastMessage`: UiMessage
//)
//
//public data class UiContact (
//    var `name`: kotlin.String,
//    var `endpointId`: kotlin.ByteArray
//)
//
//public data class UiMessage (
//    var `sender`: UiSender,
//    var `content`: kotlin.String,
//    var `sentAt`: kotlin.Long
//)







//public data class UIContact(
//    val name: String,
//    val endpointId: ByteArray
//)
//
//public data class UIChat(
//    val name: String?,
//    val members: List<UIContact>,
//    val topicId: ByteArray
//)
//
//public data class UIMessage(
//    val fromMe: Boolean,
//    val contact: UIContact?,
//    val content: String,
//    val sentAt: Long
//)
//
//public data class UIChatMessages(
//    val chat: UIChat,
//    val messages: List<UIMessage>
//)
//
//fun buildUIContact(contact: Contact): UIContact {
//    return UIContact(
//        name = contact.name,
//        endpointId = contact.endpointId
//    )
//}
//
//fun buildUIChat(chat: Chat): UIChat {
//    return UIChat(
//        name = chat.name,
//        members = chat.members.map { buildUIContact(it) },
//        topicId = chat.topicId
//    )
//}

//fun BuildUIChatsWithMessages(dB: DbEntrypoint): List<UIChatMessages> {
//
//    val dBchats = dB.getChats()
//
//    val result = mutableListOf<UIChatMessages>()
//
//    for (dBchat in dBchats) {
//
//        val dBmessages = dB.getChatMessages(dBchat.topicId)
//
//        val messages = mutableListOf<UIMessage>()
//
//        for (dBmessage in dBmessages) {
//
//            val contact =
//                if (dBmessage.fromMe) {
//                    null
//                } else {
//                    val endpoint = dBmessage.endpointId
//
//                    val match = dBchat.members.firstOrNull { member ->
//                        member.endpointId.contentEquals(endpoint)
//                    }
//
//                    match?.let { buildUIContact(it) }
//                }
//
//            messages.add(
//                UIMessage(
//                    fromMe = dBmessage.fromMe,
//                    contact = contact,
//                    content = dBmessage.content,
//                    sentAt = dBmessage.sentAt
//                )
//            )
//        }
//
//        result.add(
//            UIChatMessages(
//                chat = buildUIChat(dBchat),
//                messages = messages
//            )
//        )
//    }
//
//    return result
//}