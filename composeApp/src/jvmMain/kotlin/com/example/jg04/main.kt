package com.example.jg04

import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import com.example.jg04.ui.App
import uniffi.rust_api.UiDbManager
import uniffi.rust_api.addSampleMessages
import uniffi.rust_api.deleteDb

import java.io.File

fun main() = application {

    val dbPath = File(
        System.getProperty("user.home"),
        ".jg04/app.db"
    ).apply {
        parentFile?.mkdirs()
    }.absolutePath

    deleteDb(dbPath)
    addSampleMessages(dbPath)
    var manager = UiDbManager.spawn(dbPath)
    var dB = manager.getClient()

    Window(
        onCloseRequest = ::exitApplication,
        title = "jg04",
    ) {
        App(dB)
    }
}