package com.example.jg04.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.example.jg04.data.AppModel
import com.example.jg04.ui.theme.ComposeTutorialTheme

@Composable
fun App(model: AppModel) {

    ComposeTutorialTheme {

        val navController =
            remember {
                NavigationController(
                    initialScreen = Screen.ChatLists
                )
            }

        val chats by model.chats.collectAsState()
        val chatsWithMessages by model.chatData.collectAsState()
        val contacts by model.contacts.collectAsState()

        Surface(
            modifier = Modifier.fillMaxSize()
        ) {

            val chatList by remember(chats) { derivedStateOf { chats.values.toList() } }

            if (chatList.isNotEmpty()) {

                AppNavigation(
                    currentScreen = navController.currentScreen,
                    chatList = chatList,
                    chatsWithMessages = chatsWithMessages,
                    contacts = contacts,
                    onNavigateToChat = { id ->
                        navController.navigateTo(
                            Screen.Chat(topicId = id)
                        )
                    },

                    onNavigateToShareProfile = {
                        navController.navigateTo(
                            Screen.ShareProfile
                        )
                    },

                    onNavigateToNewChat = {
                        navController.navigateTo(
                            Screen.NewChat
                        )
                    },

                    onNavigateToNewContact = {
                        navController.navigateTo(
                            Screen.NewContact
                        )
                    },

                    onNavigateToContacts = {
                        navController.navigateTo(
                            Screen.Contacts
                        )
                    },

                    onNavigateToSettings = {
                        navController.navigateTo(
                            Screen.Settings
                        )
                    },

                    onBack = {
                        navController.pop()
                    }
                )

            } else {

                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {

                    CircularProgressIndicator()
                }
            }
        }
    }
}