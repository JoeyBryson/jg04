package com.example.jg04

import uniffi.rust_api.LogLevel
import uniffi.rust_api.LogListener

class NativeLogListener : LogListener {

    override fun log(level: LogLevel, line: String) {
        PlatformLogger.logOut(level, line)
    }
}

expect object PlatformLogger {
    fun logOut(level: LogLevel, line: String)
}

object KotlinLogger {

    fun trace(tag: String, message: String) {
        PlatformLogger.logOut(LogLevel.TRACE, "$tag: $message")
    }

    fun debug(tag: String, message: String) {
        PlatformLogger.logOut(LogLevel.DEBUG, "$tag: $message")
    }

    fun info(tag: String, message: String) {
        PlatformLogger.logOut(LogLevel.INFO, "$tag: $message")
    }

    fun warn(tag: String, message: String) {
        PlatformLogger.logOut(LogLevel.WARN, "$tag: $message")
    }

    fun error(
        tag: String,
        message: String,
        throwable: Throwable? = null,
    ) {
        val finalMessage = buildString {
            append(message)

            throwable?.let {
                append("\n")
                append(it.stackTraceToString())
            }
        }

        PlatformLogger.logOut(LogLevel.ERROR, "$tag: $finalMessage")
    }
}