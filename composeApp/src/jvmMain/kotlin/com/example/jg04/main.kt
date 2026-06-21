package com.example.jg04

import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import com.example.jg04.initializeApp
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import com.example.jg04.ui.App

import java.io.File

fun main() = application {

    val dbPath = File(
        System.getProperty("user.home"),
        ".jg04/app.db"
    ).apply {
        parentFile?.mkdirs()
    }.absolutePath

    val runtime = initializeApp(
        AppDependencies(
            dbPath = dbPath,
            scope = CoroutineScope(Dispatchers.Default)
        )
    )

    Window(
        onCloseRequest = ::exitApplication,
        title = "jg04"
    ) {
        App(runtime.model)
    }
}