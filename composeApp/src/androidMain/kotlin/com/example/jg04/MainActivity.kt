package com.example.jg04

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import android.content.Context
import com.example.jg04.ui.App
import uniffi.rust_api.initNativeLogger
import uniffi.rust_api.setupTestDb
import android.util.Log
import com.example.jg04.KotlinLogger

class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        enableEdgeToEdge()

        initNativeLogger(NativeLogForwarder())

        val dbPath = getDatabasePath("app.db").apply {
            parentFile?.mkdirs()
        }.absolutePath

        KotlinLogger.error("TESTING_LOGGING", "it works!")

        val dB = setupTestDb(dbPath)

        setContent {
            App(dB)
        }
    }
}