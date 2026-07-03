package com.example.jg04.state

import com.example.jg04.KotlinLogger
import com.example.jg04.NativeLogForwarder
import com.example.jg04.testing.AppBackgroundTicker
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharedFlow
import uniffi.rust_api.UiDbManagerUniffiObject
import uniffi.rust_api.addSampleMessage
import uniffi.rust_api.initNativeLogger
import uniffi.rust_api.registerUiEventListener

object AppCore {

    var initialized: Boolean = false

    lateinit var _dbPath: String
        private set
    lateinit var dbManager: UiDbManagerUniffiObject
        private set

    lateinit var ticker: AppBackgroundTicker
        private set
    private val dbEventListener = UiEventListenerImpl()

    fun getChatHeadersInvalidation(): SharedFlow<Unit> = dbEventListener.chatHeadersInvalidated
    fun getContactsInvalidation(): SharedFlow<Unit> = dbEventListener.contactsInvalidated
    fun getChatDataInvalidation(): SharedFlow<String> = dbEventListener.chatDataInvalidated


    fun initialize(dbPath: String) {

        initNativeLogger(NativeLogForwarder())

        if (initialized) {
            KotlinLogger.warn("AppCore", "AppCore is already initialized. Skipping.")
            return
        }
        _dbPath = dbPath
        KotlinLogger.info("AppCore", "Initializing Native Core Components...")

        dbManager = UiDbManagerUniffiObject.spawn(dbPath)

        val success = registerUiEventListener(dbEventListener)
        if (!success) {
            KotlinLogger.error(
                "AppCore",
                "Event Listener Registration failed (already registered?)."
            )
        }
        dbEventListener.start(CoroutineScope(Dispatchers.Default))

        ticker = AppBackgroundTicker(
            CoroutineScope(Dispatchers.Default),
            dbPath = dbPath,
            onTick = { dbPath, count -> addSampleMessage(dbPath, count) }
        )

        ticker.start()

        KotlinLogger.info("AppCore", "Initialization complete.")
    }
}

interface DbManagerProvider {
    val dbManager: UiDbManagerUniffiObject
}

