package com.example.jg04.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.unit.dp
import com.example.jg04.ui.icons.arrowBackIcon
import com.example.jg04.ui.icons.contentCopyIcon

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ShareProfileScreen(
    endpointId: String,
    onBackPress: () -> Unit
) {
    val clipboardManager = LocalClipboardManager.current

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Share Profile") },
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
                .fillMaxSize()
                .padding(paddingValues)
                .padding(horizontal = 24.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center
        ) {
            Text(
                text = "EndpointID:",
                modifier = Modifier.padding(bottom = 8.dp)
            )

            SelectionContainer {
                Box(
                    modifier = Modifier.width(IntrinsicSize.Min)
                ) {
                    OutlinedTextField(
                        value = endpointId,
                        onValueChange = {},
                        readOnly = true,
                        singleLine = false,
                        trailingIcon = {
                            IconButton(
                                onClick = {
                                    clipboardManager.setText(
                                        AnnotatedString(endpointId)
                                    )
                                }
                            ) {
                                Icon(
                                    imageVector = contentCopyIcon,
                                    contentDescription = "Copy EndpointID"
                                )
                            }
                        },
                        modifier = Modifier.fillMaxWidth()
                    )
                }
            }
        }
    }
}