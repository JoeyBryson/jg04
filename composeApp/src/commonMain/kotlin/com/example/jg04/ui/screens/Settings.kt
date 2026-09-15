package com.example.jg04.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.example.jg04.state.AppCore
import com.example.jg04.ui.icons.arrowBackIcon

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    onDataBaseReset: () -> Unit,
    onBackPress: () -> Unit
) {
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Settings") },
                navigationIcon = {
                    IconButton(onClick = onBackPress) {
                        Icon(
                            imageVector = arrowBackIcon,
                            contentDescription = "Back"
                        )
                    }
                }
            )
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .padding(paddingValues)
                .verticalScroll(rememberScrollState())
                .padding(24.dp),
            verticalArrangement = Arrangement.spacedBy(24.dp)
        ) {
            DeveloperOptions(onDataBaseReset)
        }
    }
}

@Composable
private fun DeveloperOptions(onDataBaseReset: () -> Unit,) {
    Column(
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Text("Developer options")

        HorizontalDivider()

        Button(
            onClick = {
                val client = AppCore.dbManager.spawnClient()
                client.resetDatabase()
                onDataBaseReset()
                      },
            modifier = Modifier
        ) {
            Text("Reset database")
        }
    }
}

