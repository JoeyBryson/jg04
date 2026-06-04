package com.example.jg04.data

import uniffi.rust_api.UiEvent
import uniffi.rust_api.UiEventListener

class AppEventBridge(
    private val controller: AppController
) : UiEventListener {

    override fun onEvent(event: UiEvent) {
        controller.handleEvent(event)
    }
}