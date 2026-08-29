package com.example.jg04.state

import com.example.jg04.KotlinLogger
import com.example.jg04.NativeLogForwarder
import com.example.jg04.testing.AppBackgroundTicker
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharedFlow
import uniffi.rust_api.UiDbManager
import uniffi.rust_api.addSampleMessage
import uniffi.rust_api.initNativeLogger
import uniffi.rust_api.registerUiEventListener

object AppCore {

    var initialized: Boolean = false

    lateinit var _dbPath: String
        private set
    lateinit var dbManager: UiDbManager
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

        dbManager = UiDbManager.spawn(dbPath)
        start_listener()
        start_ticker()

        KotlinLogger.info("AppCore", "Initialization complete.")
        initialized = true
    }

    fun profile_exists(): Boolean {
        val dbClient = dbManager.getClient()
        return dbClient.profileExists()
    }

    fun start_listener() {
        runCatching {
            registerUiEventListener(dbEventListener)
        }.onFailure {
            KotlinLogger.error("AppCore", "${it.message}")
        }

        dbEventListener.start(CoroutineScope(Dispatchers.Default))
    }

    fun start_ticker() {
        ticker = AppBackgroundTicker(
            CoroutineScope(Dispatchers.Default),
            dbPath = _dbPath,
            onTick = { dbPath, count -> addSampleMessage(dbPath, count) }
        )

        ticker.start()
    }
}

interface DbManagerProvider {
    val dbManager: UiDbManager
}

