package com.example.jg04.data

import uniffi.rust_api.UiEvent
import uniffi.rust_api.UiEventListener
import uniffi.rust_api.registerUiEventListener
import com.example.jg04.KotlinLogger

class DBEventListener(
    private val controller: ModelController
) : UiEventListener {

    override fun onEvent(event: UiEvent) {
        controller.handleEvent(event)
    }
}

fun initializeDbEventListener(controller: ModelController) {
    val listener = DBEventListener(controller)
    KotlinLogger.info("AppEventBridge", "Initializing ")
    val success = registerUiEventListener(listener)
    if (!success) {
        KotlinLogger.error("AppEventBridge", "bridge initialization failed (already initialized?).)!")
    }
    KotlinLogger.info("AppEventBridge","Initialization complete.")
}