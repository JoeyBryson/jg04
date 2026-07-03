package com.example.jg04.state

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import com.example.jg04.KotlinLogger
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import uniffi.rust_api.UiEvent
import uniffi.rust_api.UiEventListener

class UiEventListenerImpl : UiEventListener {
    
    private val _chatHeadersInvalidated = MutableSharedFlow<Unit>(extraBufferCapacity = 16)
    val chatHeadersInvalidated = _chatHeadersInvalidated.asSharedFlow()

    private val _contactsInvalidated = MutableSharedFlow<Unit>(extraBufferCapacity = 16)
    val contactsInvalidated = _contactsInvalidated.asSharedFlow()

    private val _chatDataChanged = MutableSharedFlow<String>(extraBufferCapacity = 16)
    val chatDataInvalidated = _chatDataChanged.asSharedFlow()

    val events = Channel<UiEvent>(capacity = 128)

    override fun onEvent(event: UiEvent) {
        val result = events.trySend(event)

        KotlinLogger.info("UI events", "onEvent triggered, emitting: $result")
        if (result.isFailure) {
            KotlinLogger.error(
                "Notification system",
                "UI event dropped (buffer full): $event"
            )
        }
    }

    fun start(scope: CoroutineScope) {

        scope.launch {

            while (true) {

                val first = events.receive()

                val batch = mutableListOf(first)

                while (true) {
                    val next = events.tryReceive().getOrNull()
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
            KotlinLogger.info("UI events", "event observed")
            when (event) {

                is UiEvent.ChatHeadersChanged -> {
                    chatsDirty = true
                }

                is UiEvent.ContactsChanged -> {
                    contactsDirty = true
                }

                is UiEvent.ChatDataChanged -> {
                    messageTopics.add(event.topicId)
                }
            }
        }

        if (chatsDirty) {
            KotlinLogger.info("UI events", "chat reload triggered")
            reloadChatHeaders()
        }

        if (contactsDirty) {
            reloadContacts()
        }

        if (messageTopics.isNotEmpty()) {
            reloadChatData(messageTopics)
        }
    }

    private fun reloadChatHeaders() {
        KotlinLogger.info("EventListener", "reloadChats")
        _chatHeadersInvalidated.tryEmit(Unit)
    }

    private fun reloadContacts() {
        KotlinLogger.info("EventListener", "reloadContacts")
        _contactsInvalidated.tryEmit(Unit)
    }

    private fun reloadChatData(topics: Set<String>) {
        KotlinLogger.info("EventListener", "reloadMessages: $topics")
        for (topic in topics) {
            _chatDataChanged.tryEmit(topic)
        }
    }
}