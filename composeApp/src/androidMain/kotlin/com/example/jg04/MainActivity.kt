package com.example.jg04

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.lifecycle.lifecycleScope
import com.example.jg04.ui.App
import uniffi.rust_api.initNativeLogger

class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        initNativeLogger(NativeLogForwarder())

        val dbPath = getDatabasePath("app.db").apply {
            parentFile?.mkdirs()
        }.absolutePath

        val runtime = initializeApp(
            AppDependencies(
                dbPath = dbPath,
                scope = lifecycleScope
            )
        )
        //var viewModel = AppViewModel(runtime.model)

        setContent {
            App(runtime.model)
        }
    }
}