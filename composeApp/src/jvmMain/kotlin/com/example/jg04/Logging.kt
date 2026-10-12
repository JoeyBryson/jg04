package com.example.jg04

import uniffi.rust_api.LogLevel

actual object PlatformLogger {

    var minLevel: LogLevel = LogLevel.INFO

    actual fun logOut(level: LogLevel, line: String) {
        if (!shouldLog(level)) return

        val color = when (level) {
            LogLevel.ERROR -> "\u001B[31m" // Red
            LogLevel.WARN -> "\u001B[33m" // Yellow
            LogLevel.INFO -> "\u001B[32m" // Green
            LogLevel.DEBUG -> "\u001B[36m" // Cyan
            LogLevel.TRACE -> "\u001B[35m" // Magenta
        }
        val reset = "\u001B[0m"

        val formatted = "$color$line$reset"

        // 4. Send to appropriate standard output stream
        if (level == LogLevel.ERROR || level == LogLevel.WARN) {
            System.err.println(formatted)
        } else {
            System.out.println(formatted)
        }
    }

    private fun shouldLog(level: LogLevel): Boolean {
        return severity(level) >= severity(minLevel)
    }

    private fun severity(level: LogLevel): Int =
        when (level) {
            LogLevel.TRACE -> 0
            LogLevel.DEBUG -> 1
            LogLevel.INFO -> 2
            LogLevel.WARN -> 3
            LogLevel.ERROR -> 4
        }

}
