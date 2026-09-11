package com.example.jg04.state

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.example.jg04.KotlinLogger
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.rust_api.DbClient
import uniffi.rust_api.UiProfile
import uniffi.rust_api.UiChatHeader
import uniffi.rust_api.UiChatData
import uniffi.rust_api.UiContact

class ProfileVM(
    private val dbClient: DbClient
) : ViewModel() {

    private val _profileExists =
        MutableStateFlow(AppCore.profileExists())

    val profileExists: StateFlow<Boolean> =
        _profileExists.asStateFlow()

    private val _profile =
        MutableStateFlow<UiProfile?>(null)

    val profile: StateFlow<UiProfile> =
        _profile.filterNotNull().stateIn(
            viewModelScope,
            SharingStarted.Eagerly,
            dbClient.getUiProfile()
        )

    fun profileIsSet() {
        if (AppCore.profileExists()) {
            _profile.value = dbClient.getUiProfile()
            _profileExists.value = true

            KotlinLogger.info("Profile Settup", "successfully set profile")
        } else {
            KotlinLogger.error("Profile Settup", "failed to set profile")
        }
    }
}

class HomePageVM(
    private val dbClient: DbClient,
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
        _chatHeaders.value = dbClient.getUiChatHeaders().associateBy { it.topicId }
    }
}

class ChatPageVM(
    private val dbClient: DbClient,
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
        _chatData.value = dbClient.getUiChatData(topicId)
    }
}

class NewChatPageVM(
    private val dbClient: DbClient,
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
        _contacts.value = dbClient.getUiContacts().associateBy { it.endpointId }
    }
}

val profileVMFactory = viewModelFactory {
    initializer {
        ProfileVM(dbClient = AppCore.dbManager.spawnClient())
    }
}

val HomePageVMFactory = viewModelFactory {
    initializer {
        HomePageVM(
            dbClient = AppCore.dbManager.spawnClient(),
            chatHeadersInvalidated = AppCore.getChatHeadersInvalidation()
        )
    }
}

fun chatPageVMFactory(topicId: String) = viewModelFactory {
    initializer {
        val dbClient = AppCore.dbManager.spawnClient()

        ChatPageVM(
            dbClient = dbClient,
            chatHeader = dbClient.getUiChatHeader(topicId),
            chatDataInvalidated = AppCore.getChatDataInvalidation()
        )
    }
}

val NewChatPageVMFactory = viewModelFactory {
    initializer {
        NewChatPageVM(
            dbClient = AppCore.dbManager.spawnClient(),
            contactsInvalidated = AppCore.getContactsInvalidation()
        )
    }
}