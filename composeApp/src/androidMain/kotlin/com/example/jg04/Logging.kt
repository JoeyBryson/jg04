package com.example.jg04

import android.util.Log
import uniffi.rust_api.LogLevel

actual object PlatformLogger {

    actual fun logOut(
        level: LogLevel,
        target: String,
        file: String?,
        line: UInt?,
        msg: String,
    ) {

        val loggingData = if (file == null && line == null) {
            "kotlin: $target"
        } else {
            buildString {
                append(file ?: "unknown")

                line?.let {
                    append(":")
                    append(it)
                }
            }
        }

        val formatted = "[$loggingData] $msg"

        when (level) {
            LogLevel.ERROR -> Log.e(target, formatted)
            LogLevel.WARN  -> Log.w(target, formatted)
            LogLevel.INFO  -> Log.i(target, formatted)
            LogLevel.DEBUG -> Log.d(target, formatted)
            LogLevel.TRACE -> Log.v(target, formatted)
        }
    }
}