package com.example.jg04.ui.screens

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable

@Composable
fun PlaceholderScreen(
    title: String,
    onBack: () -> Unit,
    content: @Composable () -> Unit
) {
    Column {
        Button(onClick = onBack) {
            Text("Back")
        }

        Text(title)

        content()
    }
}