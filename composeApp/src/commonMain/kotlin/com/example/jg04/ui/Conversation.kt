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
import com.example.jg04.data.UIChatWithMessages
import com.example.jg04.data.UIMessage
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
fun Conversation(chat: UIChatWithMessages) {
    LazyColumn {
        items(chat.messages) {message ->
            MessageRow(message)
        }
    }
}

@Composable
fun MessageRow(msg: UIMessage) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = if (msg.fromMe) Arrangement.End else Arrangement.Start
    ) {
        MessageCard(msg)
    }
}

@Composable
fun MessageCard(msg: UIMessage) {
    var isSelected by remember { mutableStateOf(false) }
    // surfaceColor will be updated gradually from one color to the other
    val surfaceColor by animateColorAsState(
        if (isSelected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.secondary,
    )

    Surface(
        shape = MaterialTheme.shapes.medium,
        shadowElevation = 1.dp,
        // surfaceColor color will be changing gradually from primary to surface
        color = surfaceColor,
        // animateContentSize will change the Surface size gradually
        modifier = Modifier.animateContentSize()
            .padding(1.dp)
            .clickable(true, onClick = {isSelected = !isSelected})
    ) {

        Column {
            msg.contact?.let { contact ->
                Row(modifier = Modifier.padding(all = 8.dp)) {
                    Text(
                        text = contact.name,
                        color = MaterialTheme.colorScheme.tertiary,
                        style = MaterialTheme.typography.titleSmall
                    )
                }
            }

            Spacer(modifier = Modifier.height(4.dp))

            Text(
                text = msg.content,
                modifier = Modifier.padding(all = 4.dp),
                style = MaterialTheme.typography.bodyMedium
            )

            Spacer(modifier = Modifier.height(4.dp))

            Text(
                text = formatTime(msg.sentAt),
                modifier = Modifier.padding(all = 4.dp),
                style = MaterialTheme.typography.bodyMedium
            )

        }
    }
}


