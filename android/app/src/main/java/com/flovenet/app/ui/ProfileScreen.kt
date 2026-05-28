package com.flovenet.app.ui

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.flovenet.app.network.Profile

@Composable
fun ProfileScreen(profile: Profile) {
    Column(modifier = Modifier.fillMaxSize().padding(24.dp)) {
        Spacer(modifier = Modifier.height(24.dp))
        Text("Profile", fontSize = 24.sp, fontWeight = FontWeight.Bold)
        Spacer(modifier = Modifier.height(24.dp))

        Card(modifier = Modifier.fillMaxWidth()) {
            Column(modifier = Modifier.padding(20.dp)) {
                Text(profile.displayName, fontSize = 22.sp, fontWeight = FontWeight.Bold)
                Spacer(modifier = Modifier.height(4.dp))
                Text("Peer ID: ${profile.peerId}", fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant)
                if (!profile.bio.isNullOrBlank()) {
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(profile.bio, fontSize = 14.sp)
                }
            }
        }

        Spacer(modifier = Modifier.height(16.dp))

        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceEvenly) {
            Card(modifier = Modifier.weight(1f).padding(4.dp)) {
                Column(modifier = Modifier.padding(16.dp).fillMaxWidth()) {
                    Text(profile.followerCount.toString(), fontSize = 24.sp, fontWeight = FontWeight.Bold)
                    Text("Followers", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
            Card(modifier = Modifier.weight(1f).padding(4.dp)) {
                Column(modifier = Modifier.padding(16.dp).fillMaxWidth()) {
                    Text(profile.followingCount.toString(), fontSize = 24.sp, fontWeight = FontWeight.Bold)
                    Text("Following", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
            Card(modifier = Modifier.weight(1f).padding(4.dp)) {
                Column(modifier = Modifier.padding(16.dp).fillMaxWidth()) {
                    Text(profile.postCount.toString(), fontSize = 24.sp, fontWeight = FontWeight.Bold)
                    Text("Posts", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }
    }
}
