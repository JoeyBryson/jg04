package com.example.jg04

import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import uniffi.rust_api.UiDbManager
import com.example.jg04.state.DbManagerProvider
import com.example.jg04.state.AppCore
import com.example.jg04.ui.MainComposable
import uniffi.rust_api.addSampleData
import uniffi.rust_api.initNativeLogger
import uniffi.rust_api.resetDbForWal


fun main() = application {
    //initNativeLogger(NativeLogForwarder())

    val dbPath = System.getProperty("user.home") + "/app.db"

    resetDbForWal(dbPath)
    addSampleData(dbPath)

    AppCore.initialize(dbPath)

    Window(
        onCloseRequest = ::exitApplication,
        title = "jg04"
    ) {
        MainComposable()
    }
}