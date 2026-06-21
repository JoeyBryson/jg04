package com.example.jg04.data

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import uniffi.rust_api.UiChat
import uniffi.rust_api.UiChatWithMessages
import uniffi.rust_api.UiContact

class AppModel {

    private val _chats = MutableStateFlow<Map<String, UiChat>>(emptyMap())
    val chats = _chats.asStateFlow()

    private val _chatData = MutableStateFlow<Map<String, UiChatWithMessages>>(emptyMap())
    val chatData = _chatData.asStateFlow()

    private val _contacts = MutableStateFlow<Map<String, UiContact>>(emptyMap())
    val contacts = _contacts.asStateFlow()

    fun replaceChats(chats: List<UiChat>) {
        _chats.value = chats.associateBy { it.topicId }
    }

    fun replaceChat(chat: UiChatWithMessages) {
        val topicId = chat.chat.topicId
        _chatData.value += (topicId to chat)
        _chats.value += (topicId to chat.chat)
    }

    fun removeChat(topicId: String) {
        _chatData.value -= topicId
        _chats.value -= topicId
    }

    fun replaceContacts(contacts: List<UiContact>) {
        _contacts.value = contacts.associateBy { it.endpointId }
    }
    
    fun getChat(topicId: String): UiChat? = _chats.value[topicId]

    fun getChatData(topicId: String): UiChatWithMessages? = _chatData.value[topicId]

    fun getContact(endpointId: String): UiContact? = _contacts.value[endpointId]

    fun getChats(): List<UiChat> {
        return _chats.value.values.toList()
    }
}

