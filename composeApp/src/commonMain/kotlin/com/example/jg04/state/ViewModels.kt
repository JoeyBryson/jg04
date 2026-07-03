package com.example.jg04.state

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.example.jg04.NativeLogForwarder
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import uniffi.rust_api.*

class HomePageVM(
    private val dbClient: UiDbClient,
    private val chatHeadersInvalidated: SharedFlow<Unit>
) : ViewModel() {

    private val _chatHeaders = MutableStateFlow<Map<String, UiChatHeader>>(emptyMap())
    val chatHeaders = _chatHeaders.asStateFlow()

    init {
        viewModelScope.launch {
            refreshChatHeaders()
        }
        checkInvalidation()
    }

    private fun checkInvalidation() {
        viewModelScope.launch {
            chatHeadersInvalidated.collect {
                refreshChatHeaders()
            }
        }
    }

    private suspend fun refreshChatHeaders() {
        _chatHeaders.value = dbClient.getChatHeaders().associateBy { it.topicId }
    }
}

class ChatPageVM(
    private val dbClient: UiDbClient,
    private val chatDataInvalidated: SharedFlow<String>,
    chatHeader: UiChatHeader,
    ) : ViewModel() {

    private val topicId = chatHeader.topicId
    private val _chatData =
        MutableStateFlow(UiChatData(chatHeader, listOfNotNull(chatHeader.lastMessage)))
    val chatData = _chatData.asStateFlow()

    init {
        viewModelScope.launch {
            refreshChatData()
        }
        checkInvalidation()
    }

    private fun checkInvalidation() {
        viewModelScope.launch {
            chatDataInvalidated.collect {
                if (it == topicId) {
                    refreshChatData()
                }
            }
        }
    }

    private suspend fun refreshChatData() {
        _chatData.value = dbClient.getChatData(topicId)
    }
}

class NewChatPageVM(
    private val dbClient: UiDbClient,
    private val contactsInvalidated: SharedFlow<Unit>
) : ViewModel() {

    private val _contacts = MutableStateFlow<Map<String, UiContact>>(emptyMap())
    val contacts = _contacts.asStateFlow()

    init {
        viewModelScope.launch {
            refreshChatHeaders()
        }
        checkInvalidation()
    }

    private fun checkInvalidation() {
        viewModelScope.launch {
            contactsInvalidated.collect {
                refreshChatHeaders()
            }
        }
    }

    private suspend fun refreshChatHeaders() {
        _contacts.value = dbClient.getContacts().associateBy { it.endpointId }
    }
}

val HomePageVMFactory = viewModelFactory {
    initializer {
        HomePageVM(
            dbClient = AppCore.dbManager.getClient(),
            chatHeadersInvalidated = AppCore.getChatHeadersInvalidation()
        )
    }
}

fun chatPageVMFactory(topicId: String) = viewModelFactory {
    initializer {
        val dbClient = AppCore.dbManager.getClient()

        ChatPageVM(
            dbClient = dbClient,
            chatHeader = dbClient.getChatHeader(topicId),
            chatDataInvalidated = AppCore.getChatDataInvalidation()
        )
    }
}

val NewChatPageVMFactory = viewModelFactory {
    initializer {
        NewChatPageVM(
            dbClient = AppCore.dbManager.getClient(),
            contactsInvalidated = AppCore.getContactsInvalidation()
        )
    }
}