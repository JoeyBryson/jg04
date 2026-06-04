package com.example.jg04.ui

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import com.example.jg04.ui.theme.ComposeTutorialTheme
import com.example.jg04.data.AppModel
import com.example.jg04.ui.ChatScreen


@Composable
fun App(model: AppModel) {
    ComposeTutorialTheme {
        val topicId = remember { "0202020202020202020202020202020202020202020202020202020202020202" }

        val allChatData by model.chatData.collectAsState()
        val currentChat = allChatData[topicId]

        Surface(modifier = Modifier.fillMaxSize()) {
            if (currentChat != null) {
                ChatScreen(currentChat)
            } else {
                CircularProgressIndicator()
            }
        }
    }
}