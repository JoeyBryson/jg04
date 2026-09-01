package com.example.jg04

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import com.example.jg04.ui.MainComposable
import uniffi.rust_api.addSampleData
import uniffi.rust_api.resetDbForWal

import android.app.Application
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import com.example.jg04.state.DbManagerProvider
import com.example.jg04.state.AppCore
import uniffi.rust_api.UiDbManager

class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        enableEdgeToEdge()

        setContent {
            MainComposable()
        }
    }
}

class App : Application(), DbManagerProvider {

    override lateinit var dbManager: UiDbManager
        private set

    override fun onCreate() {
        super.onCreate()

        val dbPath = getDatabasePath("app.db")
            .apply { parentFile?.mkdirs() }
            .absolutePath

//        resetDbForWal(dbPath)
//        addSampleData(dbPath)

        AppCore.initialize(dbPath)

    }
}

