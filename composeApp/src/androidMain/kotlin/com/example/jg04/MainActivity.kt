package com.example.jg04

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import com.example.jg04.ui.MainComposable
import uniffi.rust_api.addSampleData

import android.app.Application
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import com.example.jg04.state.DbManagerProvider
import com.example.jg04.state.AppCore
import uniffi.rust_api.DbManager
import uniffi.rust_api.addContactId
import uniffi.rust_api.addChat

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

    override lateinit var dbManager: DbManager
        private set

    override fun onCreate() {
        super.onCreate()

        val dbPath = getDatabasePath("app.db")
            .apply { parentFile?.mkdirs() }
            .absolutePath


        AppCore.initialize(dbPath)

//        addContactId(AppCore.dbManager.spawnClient(), "computer", "d7d6dda4006dce294fdd52d4ec2eab5814aa7617a7712fa2f4ff6a2050ba15d0")
//        val endpointIds = arrayListOf("d7d6dda4006dce294fdd52d4ec2eab5814aa7617a7712fa2f4ff6a2050ba15d0")
//        addChat(AppCore.dbManager.spawnClient(), "it works!",
//            endpointIds,
//            topicIdHex = "4a2e8f1b9c3d5e7f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f"
//            )

    }
}

