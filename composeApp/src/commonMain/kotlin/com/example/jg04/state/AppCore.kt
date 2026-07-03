package com.example.jg04.state

import uniffi.rust_api.UiDbManagerUniffiObject


object AppCore {
    lateinit var provider: DbManagerProvider

    val manager: UiDbManagerUniffiObject
        get() = provider.dbManager
}

interface DbManagerProvider {
    val dbManager: UiDbManagerUniffiObject
}

