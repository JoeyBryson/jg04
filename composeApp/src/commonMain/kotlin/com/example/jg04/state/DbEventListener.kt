package com.example.jg04.data

import uniffi.rust_api.UiEvent
import uniffi.rust_api.UiEventListener
import uniffi.rust_api.registerUiEventListener
import com.example.jg04.KotlinLogger
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

class DBEventListener(
    private val model: HomePageVM,
    private val scope: CoroutineScope
) : UiEventListener {

    override fun onEvent(event: UiEvent) {
        scope.launch {
            model.handleEvent(event)
        }
    }
}

fun initializeDbEventListener(
    model: HomePageVM,
    scope: CoroutineScope
) {
    val listener = DBEventListener(
        model = model,
        scope = scope
    )

    KotlinLogger.info("AppEventBridge", "Initializing ")

    val success = registerUiEventListener(listener)

    if (!success) {
        KotlinLogger.error(
            "AppEventBridge",
            "bridge initialization failed (already initialized?)."
        )
    }

    KotlinLogger.info("AppEventBridge", "Initialization complete.")
}