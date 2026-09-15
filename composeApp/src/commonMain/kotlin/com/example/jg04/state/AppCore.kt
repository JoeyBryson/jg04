package com.example.jg04.state

import com.example.jg04.KotlinLogger
import com.example.jg04.NativeLogForwarder
import com.example.jg04.testing.AppBackgroundTicker
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharedFlow
import uniffi.rust_api.NwCore
import uniffi.rust_api.DbManager
import uniffi.rust_api.addSampleData
import uniffi.rust_api.addSampleMessage
import uniffi.rust_api.initNativeLogger
import uniffi.rust_api.registerUiEventListener
import uniffi.rust_api.setSecretKey
import uniffi.rust_api.printEndpointId

object AppCore {

    private var initialized = false

    lateinit var _dbPath: String
        private set

    lateinit var dbManager: DbManager
        private set

    lateinit var ticker: AppBackgroundTicker
        private set

    private val dbEventListener = UiEventListenerImpl()

    lateinit var nwCore: NwCore
        private set

    fun getChatHeadersInvalidation(): SharedFlow<Unit> =
        dbEventListener.chatHeadersInvalidated

    fun getContactsInvalidation(): SharedFlow<Unit> =
        dbEventListener.contactsInvalidated

    fun getChatDataInvalidation(): SharedFlow<String> =
        dbEventListener.chatDataInvalidated

    fun isNwCoreInitialized(): Boolean =
        ::nwCore.isInitialized

    fun reset_db() {
        dbManager = DbManager.reset(_dbPath)
    }

    fun initialize(dbPath: String) {

//        resetDbForWal(dbPath)
//        addSampleData(dbPath)

        initNativeLogger(NativeLogForwarder())

        if (initialized) {
            KotlinLogger.warn(
                "AppCore",
                "AppCore is already initialized. Skipping."
            )
            return
        }

        _dbPath = dbPath

        KotlinLogger.info(
            "AppCore",
            "Initializing Native Core Components..."
        )

        runCatching {
            dbManager = DbManager.spawn(_dbPath)

            start_listener()
        }.onFailure { exception ->
            KotlinLogger.error(
                "AppCore",
                "Initialization failed: ${exception.message}"
            )
            return
        }

//        start_ticker()

        KotlinLogger.info(
            "AppCore",
            "Initialization complete."
        )

        initialized = true
    }

    fun profileExists(): Boolean {
        KotlinLogger.info("AppCore", "Checking for profile")
        val result = runCatching {
            val dbClient = dbManager.spawnClient()
            dbClient.profileExists()
        }.onSuccess { exists ->
            KotlinLogger.info("AppCore", "Profile exists: $exists")
        }.onFailure { exception ->
            KotlinLogger.error(
                "AppCore",
                "Failed to check profile: ${exception.message}"
            )
        }.getOrDefault(false)
        KotlinLogger.info("AppCore", "Returning from profile exists")
        return result
    }

    fun setProfile(name: String) {
        runCatching {
            val dbClient = dbManager.spawnClient()
            setSecretKey(dbClient, name)
        }.onFailure { exception ->
            KotlinLogger.error(
                "AppCore",
                "Failed to set profile: ${exception.message}"
            )
        }

        val dbClient = dbManager.spawnClient()
        printEndpointId(dbClient)
    }

    fun start_listener() {
        runCatching {
            registerUiEventListener(dbEventListener)
        }.onFailure { exception ->
            KotlinLogger.error(
                "AppCore",
                "Failed to register UI event listener: ${exception.message}"
            )
            return
        }

        dbEventListener.start(
            CoroutineScope(Dispatchers.Default)
        )
    }

    fun start_ticker() {
        runCatching {
            val client = dbManager.spawnClient()

            ticker = AppBackgroundTicker(
                CoroutineScope(Dispatchers.Default),
                dbClient = client,
                onTick = { count ->
                    addSampleMessage(client, count)
                }
            )

            ticker.start()
        }.onFailure { exception ->
            KotlinLogger.error(
                "AppCore",
                "Failed to start ticker: ${exception.message}"
            )
        }
    }

    fun startNetworking() {
        spawnNwCore()
    }

    fun spawnNwCore() {
        if (::nwCore.isInitialized) {
            KotlinLogger.warn(
                "AppCore",
                "NW core already exists. Skipping."
            )
            return
        }

        runCatching {
            NwCore.spawn(
                dbManager.spawnClient()
            )
        }.onSuccess { core ->
            nwCore = core

            KotlinLogger.info(
                "AppCore",
                "NW core spawned successfully."
            )
        }.onFailure { exception ->
            KotlinLogger.error(
                "AppCore",
                "Failed to spawn NW core: ${exception.message}"
            )
        }
    }
}

interface DbManagerProvider {
    val dbManager: DbManager
}