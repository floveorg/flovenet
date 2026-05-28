package com.flovenet.app.ui

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.flovenet.app.network.FeedItem

@Composable
fun FeedScreen(
    items: List<FeedItem>,
    onPost: (String) -> Unit,
    isLoading: Boolean
) {
    var newContent by remember { mutableStateOf("") }

    Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
        Text("Feed", fontSize = 24.sp, fontWeight = FontWeight.Bold)
        Spacer(modifier = Modifier.height(16.dp))

        OutlinedTextField(
            value = newContent,
            onValueChange = { newContent = it.take(280) },
            placeholder = { Text("What's happening in the network?") },
            modifier = Modifier.fillMaxWidth(),
            minLines = 2, maxLines = 4
        )
        Spacer(modifier = Modifier.height(8.dp))
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text("${newContent.length}/280", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            Button(
                onClick = { onPost(newContent); newContent = "" },
                enabled = newContent.isNotBlank() && newContent.length <= 280
            ) { Text("Post") }
        }

        Spacer(modifier = Modifier.height(16.dp))

        if (isLoading) {
            CircularProgressIndicator(modifier = Modifier.align(Alignment.CenterHorizontally))
        } else if (items.isEmpty()) {
            Text("No posts yet. Be the first!", modifier = Modifier.align(Alignment.CenterHorizontally),
                color = MaterialTheme.colorScheme.onSurfaceVariant)
        } else {
            LazyColumn {
                items(items) { item ->
                    Card(modifier = Modifier.fillMaxWidth().padding(vertical = 6.dp)) {
                        Column(modifier = Modifier.padding(16.dp)) {
                            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                                Text(item.author.displayName, fontWeight = FontWeight.Bold, fontSize = 14.sp)
                                Text(item.post.timestamp.take(10), fontSize = 11.sp,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                            Spacer(modifier = Modifier.height(8.dp))
                            Text(item.post.content, fontSize = 14.sp)
                        }
                    }
                }
            }
        }
    }
}
