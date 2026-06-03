package com.example.jg04.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import uniffi.rust_api.UiChat

@Composable
fun ChatListScreen(chats: List<UiChat>) {

}

@Composable
fun ChatHeader(chat: UiChat) {
    Column {
        Text(
            text = chat.name ?: chat
                .members
                .joinToString(", ")
                { it.name }
        )
        Spacer(modifier = Modifier.height(4.dp))

    }
}