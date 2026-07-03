package com.example.jg04.testing

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

class FpsCounter {

    var fps by mutableIntStateOf(0)
        private set

    private var job: Job? = null

    fun start(scope: CoroutineScope) {
        if (job != null) return

        job = scope.launch {
            var lastFrameTime = 0L

            while (true) {
                withFrameNanos { frameTimeNanos ->
                    if (lastFrameTime != 0L) {
                        val delta = frameTimeNanos - lastFrameTime
                        fps = (1_000_000_000L / delta).toInt()
                    }
                    lastFrameTime = frameTimeNanos
                }
            }
        }
    }

    fun stop() {
        job?.cancel()
        job = null
    }
}