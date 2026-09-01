//package com.example.jg04.ui
//
//import com.example.jg04.data.ChatPageModel
//import com.example.jg04.data.HomePageModel
//
//class AppViewModel(
//    val dbPath: String,
//    private val savedStateHandle: SavedStateHandle
//) : ViewModel() {
//    val dbManager = UiDbManager.spawn(dbPath)
//
//    val db = dbManager.spawnClient()
//
//    val homePageModel = HomePageModel(db)
//
//    val navController = NavigationController(
//                initialScreen = Screen.ChatLists
//            )
//
//    fun getChatModel(chatHeader: UiChatHeader): ChatPageModel {
//        return ChatPageModel(db, chatHeader)
//    }
//    companion object {
//
//        val Factory: ViewModelProvider.Factory = viewModelFactory {
//
//        }
//    }
//}