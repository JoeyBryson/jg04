package com.example.jg04.ui.screens

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isShiftPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import com.example.jg04.state.AppCore
import com.example.jg04.ui.icons.arrowBackIcon
import com.example.jg04.ui.icons.sendIcon
import com.example.jg04.ui.icons.doubleArrowDownIcon
import kotlinx.coroutines.launch
import uniffi.rust_api.UiChatData
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
fun ChatScreen(
    chat: UiChatData,
    onBackPress: () -> Unit
) {
    var messageText by rememberSaveable { mutableStateOf("") }
    var autoFollow by rememberSaveable { mutableStateOf(true) }

    val listState = rememberLazyListState()
    val coroutineScope = rememberCoroutineScope()

    val followThreshold = 1

    LaunchedEffect(chat.messages.size) {

        if (autoFollow && chat.messages.isNotEmpty()) {

            listState.scrollToItem(chat.messages.lastIndex)
        }
    }

    LaunchedEffect(listState,  chat.messages.size) {
        snapshotFlow {
            listState.layoutInfo.visibleItemsInfo.lastOrNull()?.index
        }.collect { lastVisible ->
            val lastIndex = chat.messages.lastIndex
            val newAutoFollow =
                lastVisible != null &&
                        lastVisible >= lastIndex - followThreshold

            autoFollow = newAutoFollow
        }
    }



    fun scrollToBottom() {

        autoFollow = true

        coroutineScope.launch {
            if (chat.messages.isNotEmpty()) {
                listState.scrollToItem(chat.messages.lastIndex)
            }
        }
    }

    fun sendMessage(messageText: String) {
        val chatId = chat.chat.topicId
        AppCore.nwCore.sendMessage(messageText, chatId)
    }

    Scaffold(
        topBar = {
            ChatTopBar(
                title = chat.chat.name ?: "Chat",
                onBackPress = onBackPress
            )
        },
        bottomBar = {
            MessageInputBar(
                messageText = messageText,
                onMessageTextChanged = { messageText = it },
                onSendClick = {
                    sendMessage(messageText)
                    messageText = ""
                }
            )
        }
    ) { paddingValues ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
        ) {
            MessageList(
                messages = chat.messages,
                listState = listState,
                modifier = Modifier.fillMaxSize()
            )

            if (!autoFollow) {
                FloatingActionButton(
                    onClick = ::scrollToBottom,
                    modifier = Modifier
                        .align(Alignment.BottomEnd)
                        .padding(16.dp)
                        .size(48.dp),
                    containerColor = MaterialTheme.colorScheme.primaryContainer,
                    contentColor = MaterialTheme.colorScheme.onPrimaryContainer
                ) {
                    Icon(
                        imageVector = doubleArrowDownIcon,
                        contentDescription = "Scroll to bottom"
                    )
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ChatTopBar(
    title: String,
    onBackPress: () -> Unit
) {
    TopAppBar(
        title = { Text(text = title) },
        navigationIcon = {
            IconButton(onClick = onBackPress) {
                Icon(imageVector = arrowBackIcon, contentDescription = "Back")
            }
        }
    )
}

@Composable
fun MessageList(
    messages: List<UiMessage>,
    listState: LazyListState,
    modifier: Modifier = Modifier
) {
    LazyColumn(
        state = listState,
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        items(messages) { message ->
            MessageRow(message)
        }
    }
}

@Composable
fun MessageInputBar(
    messageText: String,
    onMessageTextChanged: (String) -> Unit,
    onSendClick: () -> Unit
) {
    Surface(
        shadowElevation = 4.dp
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(8.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            MessageTextField(
                value = messageText,
                onValueChange = onMessageTextChanged,
                onSend = onSendClick,
                modifier = Modifier.weight(1f)
            )
            SendButton(
                enabled = messageText.isNotBlank(),
                onClick = onSendClick
            )
        }
    }
}

@Composable
fun MessageTextField(
    value: String,
    onValueChange: (String) -> Unit,
    onSend: () -> Unit,
    modifier: Modifier = Modifier
) {
    TextField(
        value = value,
        onValueChange = onValueChange,
        modifier = modifier.onPreviewKeyEvent { event ->
            if ((event.key == Key.Enter || event.key == Key.NumPadEnter) && event.type == KeyEventType.KeyDown) {
                if (!event.isShiftPressed) {
                    if (value.isNotBlank()) {
                        onSend()
                    }
                    true
                } else {
                    false
                }
            } else {
                false
            }
        },
        placeholder = { Text("Message") },
        maxLines = 4,
        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
        keyboardActions = KeyboardActions(onSend = { onSend() })
    )
}

@Composable
fun SendButton(
    enabled: Boolean,
    onClick: () -> Unit
) {
    IconButton(enabled = enabled, onClick = onClick) {
        Icon(imageVector = sendIcon, contentDescription = "Send Message")
    }
}

@Composable
fun MessageRow(msg: UiMessage) {
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
        targetValue = if (isSelected) {
            MaterialTheme.colorScheme.primaryContainer
        } else {
            MaterialTheme.colorScheme.secondaryContainer
        },
        label = "message_selection_color"
    )

    Surface(
        shape = MaterialTheme.shapes.medium,
        shadowElevation = 1.dp,
        color = surfaceColor,
        modifier = Modifier
            .animateContentSize()
            .padding(1.dp)
            .clickable { isSelected = !isSelected }
    ) {
        Column(modifier = Modifier.padding(8.dp)) {
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

            Text(text = msg.content, style = MaterialTheme.typography.bodyMedium)
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = formatTime(msg.sentAt),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}