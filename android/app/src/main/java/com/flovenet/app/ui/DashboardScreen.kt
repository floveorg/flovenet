package com.flovenet.app.ui

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.flovenet.app.network.Profile

@Composable
fun DashboardScreen(
    profile: Profile?,
    onLogout: () -> Unit,
    onNavigateFeed: () -> Unit,
    onNavigateProfile: () -> Unit
) {
    Column(
        modifier = Modifier.fillMaxSize().padding(24.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Spacer(modifier = Modifier.height(24.dp))
        Text("Flovenet", fontSize = 28.sp, fontWeight = FontWeight.Bold)
        Text("Decentralized Compute Network", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Spacer(modifier = Modifier.height(32.dp))

        if (profile != null) {
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(20.dp)) {
                    Text(profile.displayName, fontSize = 20.sp, fontWeight = FontWeight.Bold)
                    Text("Peer ID: ${profile.peerId.take(20)}...", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Spacer(modifier = Modifier.height(8.dp))
                    Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceEvenly) {
                        StatItem("Posts", profile.postCount.toString())
                        StatItem("Followers", profile.followerCount.toString())
                        StatItem("Following", profile.followingCount.toString())
                    }
                }
            }
        } else {
            CircularProgressIndicator()
        }

        Spacer(modifier = Modifier.height(24.dp))

        Button(onClick = onNavigateFeed, modifier = Modifier.fillMaxWidth()) {
            Text("View Feed")
        }
        Spacer(modifier = Modifier.height(8.dp))
        Button(onClick = onNavigateProfile, modifier = Modifier.fillMaxWidth()) {
            Text("My Profile")
        }

        Spacer(modifier = Modifier.weight(1f))
        OutlinedButton(onClick = onLogout) {
            Text("Logout")
        }
    }
}

@Composable
private fun StatItem(label: String, value: String) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Text(value, fontSize = 20.sp, fontWeight = FontWeight.Bold)
        Text(label, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}
