package com.example.jg04.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Chat
import androidx.compose.material.icons.filled.AccountCircle
import androidx.compose.material.icons.filled.AddComment
import androidx.compose.material.icons.filled.Badge
import androidx.compose.material.icons.filled.Chat
import androidx.compose.material.icons.filled.ModeComment
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.Notifications
import androidx.compose.material.icons.filled.People
import androidx.compose.material.icons.filled.PersonAdd
import androidx.compose.material.icons.filled.QrCode2
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Share
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import uniffi.rust_api.UiChat

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ChatListScreen(
    chats: List<UiChat>,
    onChatClick: (String) -> Unit,
    onShareProfileClick: () -> Unit,
    onNewChatClick: () -> Unit,
    onNewContactClick: () -> Unit,
    onContactsClick: () -> Unit,
    onSettingsClick: () -> Unit
) {

    Scaffold(

        modifier = Modifier.fillMaxSize(),

        contentWindowInsets = WindowInsets.safeDrawing,

        topBar = {
            ChatListTopBar(
                onShareProfileClick = onShareProfileClick,
                onNewChatClick = onNewChatClick,
                onNewContactClick = onNewContactClick,
                onContactsClick = onContactsClick,
                onSettingsClick = onSettingsClick
            )
        }

    ) { paddingValues ->

        LazyColumn(

            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues),

            contentPadding = PaddingValues(vertical = 4.dp)

        ) {

            items(chats) { chat ->

                ChatHeader(
                    chat = chat,
                    modifier = Modifier.clickable {
                        onChatClick(chat.topicId)
                    }
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ChatListTopBar(
    onShareProfileClick: () -> Unit,
    onNewChatClick: () -> Unit,
    onNewContactClick: () -> Unit,
    onContactsClick: () -> Unit,
    onSettingsClick: () -> Unit
) {

    TopAppBar(

        title = {
            Text("Chats")
        },

        actions = {

            IconButton(onClick = onShareProfileClick) {
                Icon(
                    imageVector = Icons.Default.Share,
                    contentDescription = "Share Profile"
                )
            }

            IconButton(onClick = onNewChatClick) {
                Icon(
                    imageVector = Icons.Default.AddComment,
                    contentDescription = "New Chat"
                )
            }

            IconButton(onClick = onNewContactClick) {
                Icon(
                    imageVector = Icons.Default.PersonAdd,
                    contentDescription = "New Contact"
                )
            }

            IconButton(onClick = onContactsClick) {
                Icon(
                    imageVector = Icons.Default.People,
                    contentDescription = "Contacts"
                )
            }

            IconButton(onClick = onSettingsClick) {
                Icon(
                    imageVector = Icons.Default.Settings,
                    contentDescription = "Settings"
                )
            }
        }
    )
}

@Composable
fun ChatHeader(
    chat: UiChat,
    modifier: Modifier = Modifier
) {

    Surface(

        color = MaterialTheme.colorScheme.surface,

        tonalElevation = 1.dp,

        modifier = Modifier
            .fillMaxWidth()
            .then(modifier)

    ) {

        Row(

            modifier = Modifier.padding(
                horizontal = 16.dp,
                vertical = 12.dp
            ),

            verticalAlignment = Alignment.CenterVertically

        ) {

            ChatAvatar(chat)

            Spacer(modifier = Modifier.width(12.dp))

            ChatHeaderContent(
                chat = chat,
                modifier = Modifier.weight(1f)
            )
        }
    }
}

@Composable
fun ChatAvatar(chat: UiChat) {

    Surface(

        shape = CircleShape,

        color = MaterialTheme.colorScheme.secondaryContainer,

        modifier = Modifier.size(48.dp)

    ) {

        Box(
            contentAlignment = Alignment.Center
        ) {

            Text(

                text = (
                        chat.name?.firstOrNull()
                            ?: chat.members.firstOrNull()?.name?.firstOrNull()
                            ?: '?'
                        ).toString().uppercase(),

                style = MaterialTheme.typography.titleMedium
            )
        }
    }
}

@Composable
fun ChatHeaderContent(
    chat: UiChat,
    modifier: Modifier = Modifier
) {

    Column(
        modifier = modifier
    ) {

        Text(

            text = chat.name
                ?: chat.members.joinToString(", ") { it.name },

            style = MaterialTheme.typography.titleMedium,

            maxLines = 1,

            overflow = TextOverflow.Ellipsis
        )

        Spacer(modifier = Modifier.height(4.dp))

        Text(

            text = chat.lastMessage?.content
                ?: "No messages yet",

            style = MaterialTheme.typography.bodyMedium,

            color = MaterialTheme.colorScheme.onSurfaceVariant,

            maxLines = 1,

            overflow = TextOverflow.Ellipsis
        )
    }
}