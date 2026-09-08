package com.example.jg04

import uniffi.rust_api.LogLevel

actual object PlatformLogger {

    var minLevel: LogLevel = LogLevel.INFO

    actual fun logOut(
        level: LogLevel,
        target: String,
        file: String?,
        line: UInt?,
        msg: String,
    ) {
        if (!shouldLog(level)) return

        // 1. If file or line are null (meaning it came from Kotlin, not Rust native),
        // look them up via the JVM stack trace.
        val (finalFile, finalLine) = if (file == null || line == null) {
            getCallerStackTraceInfo()
        } else {
            Pair(file, line)
        }

        // 2. ANSI escape colors
        val color = when (level) {
            LogLevel.ERROR -> "\u001B[31m" // Red
            LogLevel.WARN -> "\u001B[33m" // Yellow
            LogLevel.INFO -> "\u001B[32m" // Green
            LogLevel.DEBUG -> "\u001B[36m" // Cyan
            LogLevel.TRACE -> "\u001B[35m" // Magenta
        }
        val reset = "\u001B[0m"

        // 3. Format the location string safely
        val location = buildString {
            append(finalFile ?: "unknown")
            finalLine?.let {
                append(":")
                append(it)
            }
        }

        val formatted = "$color[$level][$target][$location] $msg$reset"

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

    /**
     * Climbs up the execution stack frame to bypass logging utilities
     * and discover the true source file and line number.
     */
    private fun getCallerStackTraceInfo(): Pair<String?, UInt?> {
        val elements = Thread.currentThread().stackTrace

        for (element in elements) {
            val className = element.className

            // Skip the native thread dump infrastructure and both logger objects
            if (className != Thread::class.java.name &&
                className != "com.example.jg04.KotlinLogger" &&
                className != "com.example.jg04.PlatformLogger"
            ) {
                return Pair(
                    element.fileName,
                    element.lineNumber.takeIf { it >= 0 }?.toUInt()
                )
            }
        }

        return Pair(null, null)
    }

}
