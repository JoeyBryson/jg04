package com.example.jg04.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import uniffi.rust_api.UiContact



@Composable
fun NewChatContent(
    contacts: Map<String, UiContact>,
    onCreateChat: (List<UiContact>) -> Unit
) {

    val selectedContacts =
        remember {
            mutableStateListOf<UiContact>()
        }

    Column(
        modifier = Modifier.fillMaxSize()
    ) {

        LazyColumn(

            modifier = Modifier.weight(1f),

            contentPadding = PaddingValues(vertical = 8.dp)

        ) {

            items(
                items = contacts.entries.toList(),
                key = { it.key }
            ) { (_, contact) ->

                val isSelected =
                    contact in selectedContacts

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

                    Spacer(
                        modifier = Modifier.width(12.dp)
                    )

                    Text(
                        text = contact.name,
                        style = MaterialTheme.typography.bodyLarge
                    )
                }
            }
        }

        val canCreateChat =
            selectedContacts.size > 0

        Button(

            onClick = {
                onCreateChat(selectedContacts.toList())
            },

            enabled = canCreateChat,

            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)

        ) {

            Text("Create Chat")
        }
    }
}