package com.example.jg04.state

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.example.jg04.NativeLogForwarder
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.rust_api.*

class HomePageVM(
    private val dbClient: UiDbClient,
) : ViewModel() {

    private val _chatHeaders = MutableStateFlow<Map<String, UiChatHeader>>(emptyMap())
    val chatHeaders = _chatHeaders.asStateFlow()

    init {
        initNativeLogger(NativeLogForwarder())

        initializeDbEventListener(
            model = this,
            scope = viewModelScope
        )

        viewModelScope.launch {
            refreshChatHeaders()
        }
    }

    suspend fun handleEvent(event: UiEvent) {
        when (event) {
            is UiEvent.ChatListChanged -> refreshChatHeaders()
            is UiEvent.ChatMessagesChanged -> refreshChatHeader(event.topicId)
            else -> {}
        }
    }

    private suspend fun refreshChatHeaders() {
        _chatHeaders.value = dbClient.getChatHeaders().associateBy { it.topicId }
    }

    private suspend fun refreshChatHeader(topicId: String) {
        val chat = dbClient.getChatHeader(topicId)
        _chatHeaders.update { it + (chat.topicId to chat) }
    }
}

class ChatPageVM(
    private val dbClient: UiDbClient,
    chatHeader: UiChatHeader,
) : ViewModel() {

    private val _chatData =
        MutableStateFlow(UiChatData(chatHeader, listOfNotNull(chatHeader.lastMessage)))
    val chatData = _chatData.asStateFlow()

    init {
        viewModelScope.launch {
            _chatData.value = dbClient.getChatData(chatHeader.topicId)
        }
    }
}

class NewChatPageVM(
    private val dbClient: UiDbClient,
) : ViewModel() {

    private val _contacts = MutableStateFlow<Map<String, UiContact>>(emptyMap())
    val contacts = _contacts.asStateFlow()

    init {
        viewModelScope.launch {
            _contacts.value = dbClient.getContacts().associateBy { it.endpointId }
        }
    }
}

val HomePageVMFactory = viewModelFactory {
    initializer {
        HomePageVM(
            dbClient = Db.manager.getClient()
        )
    }
}

fun chatPageVMFactory(topicId: String) = viewModelFactory {
    initializer {
        val dbClient = Db.manager.getClient()

        ChatPageVM(
            dbClient = dbClient,
            chatHeader = dbClient.getChatHeader(topicId)
        )
    }
}

val NewChatPageVMFactory = viewModelFactory {
    initializer {
        NewChatPageVM(
            dbClient = Db.manager.getClient()
        )
    }
}