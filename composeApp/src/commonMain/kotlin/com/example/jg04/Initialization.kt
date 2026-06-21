package com.example.jg04

import com.example.jg04.data.AppModel
import com.example.jg04.data.ModelController
import com.example.jg04.data.initializeDbEventListener
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import uniffi.rust_api.UiDbManagerUniffiObject
import uniffi.rust_api.addSampleData
import uniffi.rust_api.resetDbForWal

data class AppDependencies(
    val dbPath: String,
    val scope: CoroutineScope
)

class AppRuntime(
    val model: AppModel,
    val controller: ModelController
)

fun initializeApp(
    deps: AppDependencies
): AppRuntime {

    resetDbForWal(deps.dbPath)

    val dbManager = UiDbManagerUniffiObject.spawn(deps.dbPath)
    val db = dbManager.getClient()

    val model = AppModel()
    val controller = ModelController(model, db)

    initializeDbEventListener(controller)

    addSampleData(deps.dbPath)

    deps.scope.launch {
        controller.loadInitialState()
    }

    return AppRuntime(
        model = model,
        controller = controller
    )
}


