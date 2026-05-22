package com.example.jg04.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier

import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.ui.unit.dp
import androidx.compose.foundation.layout.height
import androidx.compose.material3.Surface
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.clickable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Arrangement
import uniffi.rust_api.DbEntrypoint
import com.example.jg04.data.*

@Composable
fun App(dB: DbEntrypoint) {
    _root_ide_package_.com.example.jg04.ui.theme.ComposeTutorialTheme {
        var sampleData by remember {
            mutableStateOf(BuildUIChatsWithMessages(dB))
        }
        Surface(modifier = Modifier.fillMaxSize()) {
            Conversation(sampleData.first())
        }
    }
}





//data class Message(val author: String, val body: String)

@Composable
fun Conversation(chat: UIChatWithMessages) {
    LazyColumn {
        items(chat.messages) {message ->
            MessageRow(message)
        }
    }
}

//public data class UIContact(
//    val name: String,
//    val endpointId: ByteArray
//)
//
//public data class UIChat(
//    val name: String?,
//    val members: List<UIContact>,
//    val topicId: ByteArray
//)
//
//public data class UIMessage(
//    val fromMe: Boolean,
//    val contact: UIContact?,
//    val content: String,
//    val sentAt: Long
//)
//
//public data class UIChatWithMessages(
//    val chat: UIChat,
//    val messages: List<UIMessage>
//)

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
        }
    }
}


