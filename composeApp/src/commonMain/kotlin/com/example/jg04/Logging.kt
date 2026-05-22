package com.example.jg04

import uniffi.rust_api.LogLevel
import uniffi.rust_api.RustLogger

class NativeLogForwarder : RustLogger {

    override fun log(
        level: LogLevel,
        target: String,
        file: String?,
        line: UInt?,
        msg: String,
    ) {
        PlatformLogger.logOut(
            level = level,
            target = target,
            file = file,
            line = line,
            msg = msg,
        )
    }
}

expect object PlatformLogger {
    fun logOut(
        level: LogLevel,
        target: String,
        file: String?,
        line: UInt?,
        msg: String,
    )
}

object KotlinLogger {

    fun trace(tag: String, message: String) {
        PlatformLogger.logOut(
            level = LogLevel.TRACE,
            target = tag,
            file = null,
            line = null,
            msg = message,
        )
    }

    fun debug(tag: String, message: String) {
        PlatformLogger.logOut(
            level = LogLevel.DEBUG,
            target = tag,
            file = null,
            line = null,
            msg = message,
        )
    }

    fun info(tag: String, message: String) {
        PlatformLogger.logOut(
            level = LogLevel.INFO,
            target = tag,
            file = null,
            line = null,
            msg = message,
        )
    }

    fun warn(tag: String, message: String) {
        PlatformLogger.logOut(
            level = LogLevel.WARN,
            target = tag,
            file = null,
            line = null,
            msg = message,
        )
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

        PlatformLogger.logOut(
            level = LogLevel.ERROR,
            target = tag,
            file = null,
            line = null,
            msg = finalMessage,
        )
    }
}