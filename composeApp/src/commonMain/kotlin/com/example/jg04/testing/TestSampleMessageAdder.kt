package com.example.jg04.testing

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.launch
import uniffi.rust_api.DbClient

class AppBackgroundTicker(
    private val applicationScope: CoroutineScope,
    private val dbClient: DbClient,
    private val onTick: suspend (count: Int) -> Unit
) {
    private val tickerFlow = flow {
        var counter = 0
        while (true) {
            emit(counter)
            counter++
            delay(1000)
        }
    }.flowOn(Dispatchers.Default)

    fun start() {
        applicationScope.launch {
            tickerFlow.collect { count ->
                onTick(count)
            }
        }
    }
}