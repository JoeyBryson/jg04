package com.example.jg04.ui.screens

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.Checkbox
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.example.jg04.state.AppCore
import com.example.jg04.ui.icons.arrowBackIcon
import uniffi.rust_api.UiContact
import androidx.compose.material3.OutlinedTextField
import uniffi.rust_api.UiChatHeader

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NewChatScreen(
    contacts: Map<String, UiContact>,
    onBackPress: () -> Unit,
    onCreateChat: (String) -> Unit
) {
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(text = "New Chat") },
                navigationIcon = {
                    IconButton(onClick = onBackPress) {
                        Icon(
                            imageVector = arrowBackIcon,
                            contentDescription = "Back"
                        )
                    }
                }
            )
        }
    ) { paddingValues ->
        NewChatContent(
            contacts = contacts,
            onCreateChat = onCreateChat,
            modifier = Modifier.padding(paddingValues)
        )
    }
}

@Composable
fun NewChatContent(
    contacts: Map<String, UiContact>,
    onCreateChat: (String) -> Unit,
    modifier: Modifier = Modifier
) {
    val selectedContacts = remember { mutableStateListOf<UiContact>() }
    var chatName by rememberSaveable { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }

    Column(
        modifier = modifier.fillMaxSize()
    ) {
        OutlinedTextField(
            value = chatName,
            onValueChange = {
                chatName = it
                error = null
            },
            label = { Text("Chat name (optional)") },
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        )

        LazyColumn(
            modifier = Modifier.weight(1f),
            contentPadding = PaddingValues(vertical = 8.dp)
        ) {
            items(
                items = contacts.entries.toList(),
                key = { it.key }
            ) { (_, contact) ->

                val isSelected = contact in selectedContacts

                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable {
                            if (isSelected) {
                                selectedContacts.remove(contact)
                            } else {
                                selectedContacts.add(contact)
                            }
                        }
                        .padding(
                            horizontal = 16.dp,
                            vertical = 12.dp
                        ),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Checkbox(
                        checked = isSelected,
                        onCheckedChange = { checked ->
                            if (checked) {
                                if (contact !in selectedContacts) {
                                    selectedContacts.add(contact)
                                }
                            } else {
                                selectedContacts.remove(contact)
                            }
                        }
                    )

                    Spacer(modifier = Modifier.width(12.dp))

                    Text(
                        text = contact.name,
                        style = MaterialTheme.typography.bodyLarge
                    )
                }
            }
        }

        val canCreateChat = selectedContacts.isNotEmpty()

        Button(
            onClick = {
                try {
                    val client = AppCore.dbManager.spawnClient()

                    val name = chatName
                        .trim()
                        .takeIf { it.isNotEmpty() }

                    val topic_id = client.addChatUi(
                        contacts = selectedContacts.toList(),
                        name = name
                    )

                    AppCore.nwCore.inviteChatMembers(
                        UiChatHeader(name,
                            selectedContacts.toList(),
                            topic_id,
                            null
                            )
                    )

                    onCreateChat(topic_id)
                } catch (e: Exception) {
                    error = e.toString()
                }
            },
            enabled = canCreateChat,
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            Text("Create Chat")
        }

        error?.let {
            Text(
                text = it,
                color = MaterialTheme.colorScheme.error,
                modifier = Modifier.padding(
                    horizontal = 16.dp,
                    vertical = 8.dp
                )
            )
        }
    }
}