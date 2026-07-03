package com.example.jg04.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.example.jg04.ui.theme.ComposeTutorialTheme
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.unit.dp

import com.example.jg04.testing.FpsCounter

@Composable
fun MainComposable() {
    ComposeTutorialTheme {
        Box {
            Navigator()

            FpsOverlay(
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .padding(8.dp)
            )
        }
    }
}

@Composable
fun FpsOverlay(
    modifier: Modifier = Modifier
) {
    val fpsCounter = remember { FpsCounter() }

    LaunchedEffect(Unit) {
        fpsCounter.start(this)
    }

    DisposableEffect(Unit) {
        onDispose {
            fpsCounter.stop()
        }
    }

    Surface(
        modifier = modifier,
        tonalElevation = 4.dp
    ) {
        Text(
            "FPS: ${fpsCounter.fps}",
            modifier = Modifier.padding(8.dp)
        )
    }
}