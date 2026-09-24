package com.example.jg04.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import com.example.jg04.state.AppCore
import com.example.jg04.ui.icons.arrowBackIcon
import qrscanner.CameraLens
import qrscanner.OverlayShape
import qrscanner.QrScanner
import uniffi.rust_api.UiContact

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AddContactScreen(
    onBackPress: () -> Unit
) {
    var name by remember { mutableStateOf("") }
    var endpointId by remember { mutableStateOf("") }

    var scanning by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var success by remember { mutableStateOf(false) }

    val canAdd = name.isNotBlank() && endpointId.isNotBlank()

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Add Contact") },
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

        if (scanning) {
            QrScanner(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(paddingValues),
                flashlightOn = false,
                cameraLens = CameraLens.Back,
                openImagePicker = false,
                onCompletion = { result ->
                    endpointId = result
                    scanning = false
                    error = null
                    success = false
                },
                imagePickerHandler = {},
                onFailure = {
                    error = it.ifEmpty { "Invalid QR code" }
                },
                overlayShape = OverlayShape.Square
            )
        } else {
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(paddingValues)
                    .padding(horizontal = 24.dp),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Column(
                    modifier = Modifier
                        .weight(1f)
                        .fillMaxWidth()
                        .padding(top = 32.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.Top
                ) {
                    OutlinedTextField(
                        value = name,
                        onValueChange = {
                            name = it
                            error = null
                            success = false
                        },
                        label = { Text("Name") },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth()
                    )

                    Spacer(modifier = Modifier.height(32.dp))

                    OutlinedTextField(
                        value = endpointId,
                        onValueChange = {
                            endpointId = it
                            error = null
                            success = false
                        },
                        label = { Text("Endpoint ID") },
                        singleLine = false,
                        modifier = Modifier.fillMaxWidth()
                    )

                    Spacer(modifier = Modifier.height(16.dp))

                    Button(
                        onClick = {
                            scanning = true
                            error = null
                            success = false
                        },
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Text("Scan from QR code")
                    }

                    if (error != null) {
                        Text(
                            text = error!!,
                            color = Color.Red,
                            modifier = Modifier.padding(top = 12.dp)
                        )
                    }

                    if (success) {
                        Text(
                            text = "Contact successfully added!",
                            modifier = Modifier.padding(top = 12.dp)
                        )
                    }
                }

                Button(
                    onClick = {
                        val contact = UiContact(name, endpointId)
                        val client = AppCore.dbManager.spawnClient()

                        try {
                            client.addUiContact(contact)
                            success = true
                            error = null
                        } catch (e: Exception) {
                            success = false
                            error = e.message ?: "Failed to add contact"
                        }
                    },
                    enabled = canAdd,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(bottom = 24.dp)
                ) {
                    Text("Add Contact")
                }
            }
        }
    }
}
