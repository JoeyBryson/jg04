package com.example.jg04.ui

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import uniffi.rust_api.UiChat
import uniffi.rust_api.UiChatWithMessages
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.TopAppBar
import uniffi.rust_api.UiContact

@Composable
fun AppNavigation(
    currentScreen: Screen,
    chatList: List<UiChat>,
    chatsWithMessages: Map<String, UiChatWithMessages>,
    contacts: Map<String, UiContact>,
    onNavigateToChat: (String) -> Unit,
    onNavigateToShareProfile: () -> Unit,
    onNavigateToNewChat: () -> Unit,
    onNavigateToNewContact: () -> Unit,
    onNavigateToContacts: () -> Unit,
    onNavigateToSettings: () -> Unit,
    onBack: () -> Unit
) {

    AnimatedContent(
        targetState = currentScreen,
        transitionSpec = { fadeIn() togetherWith fadeOut() }
    ) { targetScreen ->

        when (targetScreen) {

            is Screen.ChatLists -> {

                ChatListScreen(
                    chats = chatList,
                    onChatClick = onNavigateToChat,
                    onShareProfileClick = onNavigateToShareProfile,
                    onNewChatClick = onNavigateToNewChat,
                    onNewContactClick = onNavigateToNewContact,
                    onContactsClick = onNavigateToContacts,
                    onSettingsClick = onNavigateToSettings
                )
            }

            is Screen.Chat -> {

                val fullChatData =
                    chatsWithMessages[targetScreen.topicId]

                if (fullChatData != null) {

                    ChatScreen(
                        chat = fullChatData,
                        onBackPress = onBack
                    )

                } else {

                    ErrorScreen("Chat data failed to load")
                }
            }

            is Screen.ShareProfile -> {
                ShareProfileScreen(onBack = onBack)
            }

            is Screen.NewChat -> {
                NewChatScreen(onBack = onBack,
                    contacts,
                    {})
            }

            is Screen.NewContact -> {
                NewContactScreen(onBack = onBack)
            }

            is Screen.Contacts -> {
                ContactsScreen(onBack = onBack)
            }

            is Screen.Settings -> {
                SettingsScreen(onBack = onBack)
            }
        }
    }
}

class NavigationController(initialScreen: Screen) {

    private val _stack =
        mutableStateListOf<Screen>(initialScreen)

    val currentScreen: Screen
        get() = _stack.last()

    fun navigateTo(screen: Screen) {
        _stack.add(screen)
    }

    fun pop(): Boolean {

        return if (_stack.size > 1) {

            _stack.removeAt(_stack.lastIndex)
            true

        } else {

            false
        }
    }
}

sealed interface Screen {

    data object ChatLists : Screen

    data class Chat(
        val topicId: String
    ) : Screen

    data object ShareProfile : Screen

    data object NewChat : Screen

    data object NewContact : Screen

    data object Contacts : Screen

    data object Settings : Screen
}

@Composable
fun ShareProfileScreen(
    onBack: () -> Unit
) {
    PlaceholderScreen(
        title = "Share Profile Screen",
        onBack = onBack,
        {}
    )
}

@Composable
fun NewChatScreen(
    onBack: () -> Unit,
    contacts: Map<String, UiContact>,
    onCreateChat: (List<UiContact>) -> Unit
) {

    PlaceholderScreen(
        title = "New Chat",
        onBack = onBack
    ) {

        NewChatContent(
            contacts = contacts,
            onCreateChat = onCreateChat
        )
    }
}

@Composable
fun NewContactScreen(
    onBack: () -> Unit
) {
    PlaceholderScreen(
        title = "New Contact Screen",
        onBack = onBack,
        {}
    )
}

@Composable
fun ContactsScreen(
    onBack: () -> Unit
) {
    PlaceholderScreen(
        title = "Contacts Screen",
        onBack = onBack,
        {}
    )
}

@Composable
fun SettingsScreen(
    onBack: () -> Unit
) {
    PlaceholderScreen(
        title = "Settings Screen",
        onBack = onBack,
        {}
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PlaceholderScreen(
    title: String,
    onBack: () -> Unit,
    content: @Composable () -> Unit
) {

    Scaffold(

        topBar = {

            TopAppBar(

                title = {
                    Text(title)
                },

                navigationIcon = {

                    IconButton(
                        onClick = onBack
                    ) {

                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = "Back"
                        )
                    }
                }
            )
        }

    ) { paddingValues ->

        Box(

            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)

        ) {

            content()
        }
    }
}