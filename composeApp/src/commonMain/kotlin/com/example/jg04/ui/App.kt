package com.example.jg04.ui

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.runtime.Composable
import androidx.compose.material3.Surface
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import uniffi.rust_api.UiDbClient
import com.example.jg04.data.*




@Composable
fun App(dB: UiDbClient) {
    _root_ide_package_.com.example.jg04.ui.theme.ComposeTutorialTheme {

        val topicId = ByteArray(32) { 2 }

        val chatQuery = UiQuery {
            dB.getChatWithMessages(topicId)
        }

        Surface(modifier = Modifier.fillMaxSize()) {
            ChatScreen(chatQuery.state)
        }
    }
}
