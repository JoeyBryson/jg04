package com.example.jg04.data

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import uniffi.rust_api.UiChat
import uniffi.rust_api.UiChatWithMessages
import uniffi.rust_api.UiContact

class AppModel {

    // =========================================================
    // Internal mutable state
    // =========================================================

    private val _chats =
        MutableStateFlow<Map<String, UiChat>>(emptyMap())

    private val _chatData =
        MutableStateFlow<Map<String, UiChatWithMessages>>(emptyMap())

    private val _contacts =
        MutableStateFlow<Map<String, UiContact>>(emptyMap())

    // =========================================================
    // Public read-only state
    // =========================================================

    val chats: StateFlow<Map<String, UiChat>>
            = _chats.asStateFlow()

    val chatData: StateFlow<Map<String, UiChatWithMessages>>
            = _chatData.asStateFlow()

    val contacts: StateFlow<Map<String, UiContact>>
            = _contacts.asStateFlow()

    // =========================================================
    // Chat mutations
    // =========================================================

    fun replaceChats(chats: List<UiChat>) {

        _chats.value =
            chats.associateBy {
                it.topicId
            }
    }

    fun replaceChat(chat: UiChatWithMessages) {

        val topicId = chat.chat.topicId

        _chatData.value =
            _chatData.value + (topicId to chat)

        // keep chat list in sync
        _chats.value =
            _chats.value + (topicId to chat.chat)
    }

    fun removeChat(topicId: String) {

        _chatData.value =
            _chatData.value - topicId

        _chats.value =
            _chats.value - topicId
    }

    // =========================================================
    // Contact mutations
    // =========================================================

    fun replaceContacts(contacts: List<UiContact>) {

        _contacts.value =
            contacts.associateBy {
                it.endpointId
            }
    }

    // =========================================================
    // Convenience lookups
    // =========================================================

    fun getChat(topicId: String): UiChat? {
        return _chats.value[topicId]
    }

    fun getChatData(topicId: String): UiChatWithMessages? {
        return _chatData.value[topicId]
    }

    fun getContact(endpointId: String): UiContact? {
        return _contacts.value[endpointId]
    }
}