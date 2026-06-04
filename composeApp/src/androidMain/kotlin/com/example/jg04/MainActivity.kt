package com.example.jg04

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.lifecycle.lifecycleScope
import com.example.jg04.data.AppController
import com.example.jg04.data.AppEventBridge
import com.example.jg04.data.AppModel
import com.example.jg04.ui.App
import kotlinx.coroutines.launch
import uniffi.rust_api.UiDbManager
import uniffi.rust_api.addSampleMessages
import uniffi.rust_api.addSampleMessage
import uniffi.rust_api.resetDbForWal
import uniffi.rust_api.initNativeLogger
import com.example.jg04.data.AppBackgroundTicker

class MainActivity : ComponentActivity() {

    // Keep a strong reference here so the Garbage Collector never drops it!
    private lateinit var bridge: AppEventBridge

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        initNativeLogger(NativeLogForwarder())

        val dbPath = getDatabasePath("app.db").apply {
            parentFile?.mkdirs()
        }.absolutePath

        KotlinLogger.error("TESTING_LOGGING", "it works!")

        resetDbForWal(dbPath)

        val dbPathString = dbPath
        val manager = UiDbManager.spawn(dbPathString)
        val dB = manager.getClient()

        val model = AppModel()
        val controller = AppController(model, dB)

        // Assign to the class-level property
        bridge = AppEventBridge(controller)

        addSampleMessages(dbPath, bridge)

        val ticker = AppBackgroundTicker(
            lifecycleScope,
            dbPath,
            bridge,
            { path, eventBridge, count ->
                addSampleMessage(path, eventBridge, count)
            }
        )
        ticker.start()

        lifecycleScope.launch {
            controller.loadInitialState()
        }

        setContent {
            App(model) // Clean and separate!
        }
    }
}