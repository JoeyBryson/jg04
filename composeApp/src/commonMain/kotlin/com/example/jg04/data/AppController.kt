package com.example.jg04.data

import com.example.jg04.KotlinLogger
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import uniffi.rust_api.UiDbClient
import uniffi.rust_api.UiEvent

class AppController(
    private val model: AppModel,
    private val db: UiDbClient
) {

    private val scope =
        CoroutineScope(Dispatchers.IO)

    fun handleEvent(event: UiEvent) {
        KotlinLogger.info("AppController", "Received event: $event")

        when (event) {

            UiEvent.ChatListChanged -> {
                scope.launch {
                    refreshChats()
                }
            }

            UiEvent.ContactsChanged -> {
                scope.launch {
                    refreshContacts()
                }
            }

            is UiEvent.ChatMessagesChanged -> {
                scope.launch {
                    refreshChat(event.topicId)
                }
            }
        }
    }

    suspend fun loadInitialState() {
        KotlinLogger.info("AppController", "Loading initial state...")

        // 1. Refresh the baseline contact list
        refreshContacts()

        // 2. Fetch all baseline chat configurations from the DB
        val chats = db.getChats()
        model.replaceChats(chats)
        KotlinLogger.info("AppController", "Found ${chats.size} chats. Pre-fetching message data...")

        // 3. Loop through every chat found and hydrate its message data cache
        chats.forEach { chat ->
            refreshChat(chat.topicId)
        }

        KotlinLogger.info("AppController", "Initial state loaded completely. All chats fully hydrated.")
    }

    private fun refreshChats() {
        KotlinLogger.info("AppController", "Refreshing chats list from database...")
        val chats =
            db.getChats()

        model.replaceChats(chats)
        KotlinLogger.info("AppController", "Successfully updated model with ${chats.size} chats.")
    }

    private fun refreshContacts() {
        KotlinLogger.info("AppController", "Refreshing contacts list from database...")
        val contacts =
            db.getContacts()

        model.replaceContacts(contacts)
        KotlinLogger.info("AppController", "Successfully updated model with ${contacts.size} contacts.")
    }

    private fun refreshChat(topicId: String) {
        KotlinLogger.info("AppController", "Refreshing single chat messages for topicId: $topicId")
        val chat =
            db.getChatWithMessages(topicId)

        model.replaceChat(chat)
        KotlinLogger.info("AppController", "Successfully updated model for chat topicId: $topicId")
    }
}