package com.example.jg04

import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import uniffi.rust_api.DbManager
import com.example.jg04.state.DbManagerProvider
import com.example.jg04.state.AppCore
import com.example.jg04.ui.MainComposable
import uniffi.rust_api.addChat
import uniffi.rust_api.addContactId
import uniffi.rust_api.addSampleData
import uniffi.rust_api.initNativeLogger


fun main() = application {
    //initNativeLogger(NativeLogForwarder())

    val dbPath = System.getProperty("user.home") + "/app.db"

    AppCore.initialize(dbPath)


    AppCore.initialize(dbPath)

//    addContactId(AppCore.dbManager.spawnClient(), "computer", "24c6a9901e5fb62ae68a1d752026ea9e7dba3c45e06ae1139de0718852f38c71")
//    val endpointIds = arrayListOf("24c6a9901e5fb62ae68a1d752026ea9e7dba3c45e06ae1139de0718852f38c71")
//    addChat(AppCore.dbManager.spawnClient(), "it works!",
//        endpointIds,
//        topicIdHex = "4a2e8f1b9c3d5e7f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f"
//    )

    Window(
        onCloseRequest = ::exitApplication,
        title = "jg04"
    ) {
        MainComposable()
    }
}