package com.example.jg04

import uniffi.rust_api.LogLevel

actual object PlatformLogger {

    actual fun logOut(
        level: LogLevel,
        target: String,
        file: String?,
        line: UInt?,
        msg: String,
    ) {

        // ANSI escape colors
        val color = when (level) {
            LogLevel.ERROR -> "\u001B[31m" // Red
            LogLevel.WARN  -> "\u001B[33m" // Yellow
            LogLevel.INFO  -> "\u001B[32m" // Green
            LogLevel.DEBUG -> "\u001B[36m" // Cyan
            LogLevel.TRACE -> "\u001B[35m" // Magenta
        }

        val reset = "\u001B[0m"

        val location = buildString {
            append(file ?: "?")

            line?.let {
                append(":")
                append(it)
            }
        }

        val formatted =
            "$color[$level][$target][$location] $msg$reset"

        // stderr for warnings/errors
        if (level == LogLevel.ERROR || level == LogLevel.WARN) {
            System.err.println(formatted)
        } else {
            System.out.println(formatted)
        }
    }
}