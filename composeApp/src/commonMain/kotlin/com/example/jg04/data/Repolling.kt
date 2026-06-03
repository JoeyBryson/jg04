package com.example.jg04.data
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue


class UiQuery<T>(
    private val query: () -> T
) {
    var state by mutableStateOf(query())
        private set

    fun refresh() {
        state = query()
    }
}