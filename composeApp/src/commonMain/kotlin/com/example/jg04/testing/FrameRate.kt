package com.example.jg04.testing

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import com.example.jg04.KotlinLogger
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import java.util.ArrayDeque

class FpsCounter {

    var fps by mutableIntStateOf(0)
        private set

    var jankCount by mutableIntStateOf(0)
        private set

    private var job: Job? = null

    private val frameTimes = ArrayDeque<Long>(120)

    fun start(scope: CoroutineScope) {
        if (job != null) return

        job = scope.launch {

            var lastFrameTime = 0L
            var lastActivityTime = 0L

            while (true) {
                withFrameNanos { frameTimeNanos ->

                    if (lastFrameTime != 0L) {
                        val delta = frameTimeNanos - lastFrameTime

                        val ms = delta / 1_000_000

                        // mark activity window
                        if (ms < 100) {
                            lastActivityTime = frameTimeNanos
                        }

                        // ignore idle periods (no false FPS warnings)
                        val idleMs = (frameTimeNanos - lastActivityTime) / 1_000_000
                        val isIdle = idleMs > 500

                        // record only when active
                        if (!isIdle) {
                            frameTimes.add(delta)

                            if (frameTimes.size > 120) {
                                frameTimes.removeFirst()
                            }

                            // jank detection
                            if (ms > 16) {
                                jankCount++
                            }

                            // compute smoothed FPS (moving average)
                            val avgDelta = frameTimes.average()
                            val smoothFps = (1_000_000_000.0 / avgDelta).toInt()

                            fps = smoothFps

                            // warning only on sustained degradation
                            if (frameTimes.size >= 60) {
                                val slowFrames = frameTimes.count { it / 1_000_000 > 16 }

                                val slowRatio = slowFrames.toFloat() / frameTimes.size

                                if (slowRatio > 0.2) {
                                    KotlinLogger.warn(
                                        "FrameRate",
                                        "Sustained jank detected: fps=$smoothFps, jankRatio=$slowRatio"
                                    )
                                }
                            }
                        }
                    }

                    lastFrameTime = frameTimeNanos
                }
            }
        }
    }

    fun stop() {
        job?.cancel()
        job = null
        frameTimes.clear()
    }
}