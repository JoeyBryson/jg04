package com.example.jg04.ui

import androidx.compose.animation.ContentTransform
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation3.runtime.NavEntry
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.ui.NavDisplay
import androidx.navigation3.runtime.rememberNavBackStack
import kotlinx.serialization.Serializable
import androidx.lifecycle.viewmodel.navigation3.rememberViewModelStoreNavEntryDecorator
import androidx.navigation3.runtime.*
import kotlinx.serialization.modules.polymorphic
import kotlinx.serialization.modules.subclass
import androidx.savedstate.serialization.SavedStateConfiguration
import com.example.jg04.KotlinLogger
import com.example.jg04.state.HomePageVM
import com.example.jg04.state.ChatPageVM
import com.example.jg04.state.NewChatPageVM
import com.example.jg04.state.HomePageVMFactory
import com.example.jg04.state.chatPageVMFactory
import com.example.jg04.state.NewChatPageVMFactory
import com.example.jg04.ui.screens.ChatListScreen
import com.example.jg04.ui.screens.ChatScreen
import com.example.jg04.ui.screens.NewChatContent
import com.example.jg04.ui.screens.NewChatScreen
import com.example.jg04.ui.screens.PlaceholderScreen
import com.example.jg04.ui.screens.ShareProfileScreen
import kotlinx.serialization.modules.SerializersModule
import androidx.compose.runtime.State


@Serializable data object Home : NavKey
@Serializable data object NewChat : NavKey
@Serializable data object ContactShare : NavKey
@Serializable data object AddContact : NavKey
@Serializable data object Settings : NavKey
@Serializable data object Contacts : NavKey
@Serializable data class Chat(val topicId: String) : NavKey



val screenKeyConfig = SavedStateConfiguration {
    serializersModule = SerializersModule {
        polymorphic(NavKey::class) {
            subclass(Home::class)
            subclass(NewChat::class)
            subclass(ContactShare::class)
            subclass(AddContact::class)
            subclass(Settings::class)
            subclass(Contacts::class)
            subclass(Chat::class)
        }
    }
}


@Composable
fun Navigator(profile_endppoint_id: String) {
    val backStack = rememberNavBackStack(
        configuration = screenKeyConfig,
        Home
    )

    val onBack: () -> Unit = {
        KotlinLogger.info("Navigation", "back button pressed")
        backStack.removeLastOrNull()
    }

    fun fadeContentTransform() = ContentTransform(
        targetContentEnter = fadeIn(tween(300)),
        initialContentExit = fadeOut(tween(300))
    )

    Surface(
        modifier = Modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background
    ) {
        NavDisplay(
            backStack = backStack,
            onBack = onBack,
            transitionSpec = {fadeContentTransform()},
            predictivePopTransitionSpec = {fadeContentTransform()},
            popTransitionSpec = {fadeContentTransform()},

            entryDecorators = listOf(
                rememberSaveableStateHolderNavEntryDecorator(),
                rememberViewModelStoreNavEntryDecorator()
            ),
            entryProvider = { key ->
                when (key) {
                    is Home -> NavEntry(key) {
                        val vm: HomePageVM = viewModel(factory = HomePageVMFactory)
                        val chatheaders by vm.chatHeaders.collectAsState()
                        val chatList by remember(chatheaders) {
                            derivedStateOf { chatheaders.values.toList() }
                        }

                        ChatListScreen(
                            chats = chatList,
                            onChatClick = { backStack.add(Chat(it)) },
                            onShareProfileClick = { backStack.add(ContactShare) },
                            onNewChatClick = { backStack.add(NewChat) },
                            onNewContactClick = { backStack.add(AddContact) },
                            onContactsClick = { backStack.add(Contacts) },
                            onSettingsClick = { backStack.add(Settings) }
                        )
                    }

                    is Chat -> NavEntry(key) {
                        val vm: ChatPageVM = viewModel(
                            factory = chatPageVMFactory(key.topicId)
                        )
                        val chatData by vm.chatData.collectAsState()

                        ChatScreen(
                            chatData,
                            onBackPress = onBack
                        )
                    }

                    is NewChat -> NavEntry(key) {
                        val vm: NewChatPageVM = viewModel(factory = NewChatPageVMFactory)
                        val contacts by vm.contacts.collectAsState()

                        NewChatScreen(
                            contacts,
                            onBackPress = onBack,
                            onCreateChat = {}
                        )
                    }

                    is ContactShare -> NavEntry(key) {
                        ShareProfileScreen(
                            endpointId = profile_endppoint_id,
                            onBackPress = onBack
                        )
                    }

                    else -> NavEntry(key) {
                        PlaceholderScreen(
                            title = "Placeholder Screen",
                            onBack = onBack,
                            {}
                        )
                    }
                }
            }
        )
    }
}