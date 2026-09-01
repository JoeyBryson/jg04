package com.example.jg04.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewmodel.compose.viewModel
import com.example.jg04.ui.theme.ComposeTutorialTheme
import com.example.jg04.state.AppCore
import com.example.jg04.state.ProfileVM
import com.example.jg04.state.profileVMFactory
import com.example.jg04.testing.FpsCounter
import com.example.jg04.ui.screens.SetProfile
import kotlinx.coroutines.delay
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.sp
import kotlin.math.roundToInt
import androidx.compose.foundation.layout.offset
import androidx.compose.runtime.mutableFloatStateOf

@Composable
fun MainComposable() {

    val profileVM: ProfileVM = viewModel(factory = profileVMFactory)

    val profileExists by profileVM.profileExists.collectAsState()

    LaunchedEffect(profileExists) {
        if (profileExists) {
            AppCore.startNetworking()
        }
    }

    ComposeTutorialTheme {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.background)
                .windowInsetsPadding(WindowInsets.safeDrawing)
        ) {

            if (profileExists) {
                Navigator()
            } else {
                SetProfile(
                    onCreateProfileButtonPressed = {
                        AppCore.setProfile()
                        profileVM.profileIsSet()
                    }
                )
            }
            NwCoreStatusOverlay()
        }
    }
}
@Composable
fun NwCoreStatusOverlay() {
    var initialized by remember {
        mutableStateOf(AppCore.isNwCoreInitialized())
    }

    var offsetX by remember { mutableFloatStateOf(0f) }
    var offsetY by remember { mutableFloatStateOf(0f) }

    LaunchedEffect(Unit) {
        while (true) {
            initialized = AppCore.isNwCoreInitialized()
            delay(1000)
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp)
    ) {
        Surface(
            modifier = Modifier
                .align(Alignment.TopEnd)
                .offset {
                    IntOffset(
                        offsetX.roundToInt(),
                        offsetY.roundToInt()
                    )
                }
                .pointerInput(Unit) {
                    detectDragGestures { change, dragAmount ->
                        change.consume()

                        offsetX += dragAmount.x
                        offsetY += dragAmount.y
                    }
                },
            shape = MaterialTheme.shapes.medium,
            color = MaterialTheme.colorScheme.surface.copy(alpha = 0.6f),
            tonalElevation = 4.dp
        ) {
            Text(
                text = if (initialized) {
                    "NwCore: INITIALIZED"
                } else {
                    "NwCore: NOT INITIALIZED"
                },
                color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.7f),
                fontSize = 11.sp,
                modifier = Modifier.padding(
                    horizontal = 8.dp,
                    vertical = 4.dp
                )
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