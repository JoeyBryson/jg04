package com.example.jg04.state

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import com.example.jg04.KotlinLogger
import kotlinx.coroutines.channels.Channel
import uniffi.rust_api.UiEvent
import uniffi.rust_api.UiEventListener

class UiEventListenerImpl : UiEventListener {

    val events = Channel<UiEvent>(capacity = 128)

    override fun onEvent(event: UiEvent) {
        val result = events.trySend(event)

        if (result.isFailure) {
            KotlinLogger.error(
                "Notification system",
                "UI event dropped (buffer full): $event"
            )
        }
    }
}

class UiEventProcessor(
    private val listener: UiEventListenerImpl
) {

    fun start(scope: CoroutineScope) {

        scope.launch {

            while (true) {

                val first = listener.events.receive()

                val batch = mutableListOf(first)

                while (true) {
                    val next = listener.events.tryReceive().getOrNull()
                        ?: break
                    batch.add(next)
                }

                handleBatch(batch)
            }
        }
    }

    private fun handleBatch(batch: List<UiEvent>) {

        var chatsDirty = false
        var contactsDirty = false

        val messageTopics = mutableSetOf<String>()

        for (event in batch) {
            when (event) {

                is UiEvent.ChatListChanged -> {
                    chatsDirty = true
                }

                is UiEvent.ContactsChanged -> {
                    contactsDirty = true
                }

                is UiEvent.ChatMessagesChanged -> {
                    messageTopics.add(event.topicId)
                }
            }
        }

        if (chatsDirty) {
            reloadChats()
        }

        if (contactsDirty) {
            reloadContacts()
        }

        if (messageTopics.isNotEmpty()) {
            reloadMessages(messageTopics)
        }
    }

    private fun reloadChats() {
        KotlinLogger.info("UI", "reloadChats")
    }

    private fun reloadContacts() {
        KotlinLogger.info("UI", "reloadContacts")
    }

    private fun reloadMessages(topics: Set<String>) {
        KotlinLogger.info("UI", "reloadMessages: $topics")
    }
}