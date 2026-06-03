package com.example.jg04.ui

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import uniffi.rust_api.UiChatWithMessages // Replaced old class with your exact UniFFI models
import uniffi.rust_api.UiMessage
import uniffi.rust_api.UiSender
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter

fun formatTime(epochMillis: Long): String {
    val formatter = DateTimeFormatter.ofPattern("HH:mm")

    return Instant.ofEpochMilli(epochMillis)
        .atZone(ZoneId.systemDefault())
        .format(formatter)
}

@Composable
fun ChatScreen(chat: UiChatWithMessages) {
    LazyColumn {
        items(chat.messages) { message ->
            MessageRow(message)
        }
    }
}

@Composable
fun MessageRow(msg: UiMessage) {
    // Check if the sender pattern matches 'Me' to decide layout arrangement
    val arrangement = when (msg.sender) {
        is UiSender.Me -> Arrangement.End
        is UiSender.Other -> Arrangement.Start
    }

    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = arrangement
    ) {
        MessageCard(msg)
    }
}

@Composable
fun MessageCard(msg: UiMessage) {
    var isSelected by remember { mutableStateOf(false) }

    val surfaceColor by animateColorAsState(
        if (isSelected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.secondary,
    )

    Surface(
        shape = MaterialTheme.shapes.medium,
        shadowElevation = 1.dp,
        color = surfaceColor,
        modifier = Modifier
            .animateContentSize()
            .padding(1.dp)
            .clickable(enabled = true, onClick = { isSelected = !isSelected })
    ) {
        Column(modifier = Modifier.padding(all = 8.dp)) {

            when (val sender = msg.sender) {
                is UiSender.Other -> {
                    Text(
                        text = sender.v1.name,
                        color = MaterialTheme.colorScheme.tertiary,
                        style = MaterialTheme.typography.titleSmall
                    )
                    Spacer(modifier = Modifier.height(4.dp))
                }
                is UiSender.Me -> {}
            }

            Text(
                text = msg.content,
                style = MaterialTheme.typography.bodyMedium
            )

            Spacer(modifier = Modifier.height(4.dp))

            Text(
                text = formatTime(msg.sentAt),
                style = MaterialTheme.typography.bodyMedium
            )
        }
    }
}