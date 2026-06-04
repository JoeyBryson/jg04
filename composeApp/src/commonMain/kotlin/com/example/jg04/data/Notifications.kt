package com.example.jg04.data

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import uniffi.rust_api.UiEvent
import uniffi.rust_api.UiEventListener

class UiQuery<T>(
    private val query: () -> T,
    events: Flow<UiEvent>,
    scope: CoroutineScope
) {
    var state by mutableStateOf(query())
        private set

    init {
        scope.launch(Dispatchers.Main) {
            events.collect {
                state = query()
            }
        }
    }

    fun refresh() {
        state = query()
    }
}

class UiEventBus {

    private val _events = MutableSharedFlow<UiEvent>(extraBufferCapacity = 64)
    val events = _events.asSharedFlow()

    fun emit(event: UiEvent) {
        _events.tryEmit(event)
    }
}

class UiEventListenerImpl(
    private val bus: UiEventBus
) : UiEventListener {

    override fun onEvent(event: UiEvent) {
        bus.emit(event)
    }
}