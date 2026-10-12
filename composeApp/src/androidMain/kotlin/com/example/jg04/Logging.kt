package com.example.jg04

import android.util.Log
import uniffi.rust_api.LogLevel

actual object PlatformLogger {

    actual fun logOut(level: LogLevel, line: String) {
        when (level) {
            LogLevel.ERROR -> Log.e("rust_api", line)
            LogLevel.WARN -> Log.w("rust_api", line)
            LogLevel.INFO -> Log.i("rust_api", line)
            LogLevel.DEBUG -> Log.d("rust_api", line)
            LogLevel.TRACE -> Log.v("rust_api", line)
        }
    }
}