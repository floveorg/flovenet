package com.flovenet.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import com.flovenet.app.network.GraphQLClient
import com.flovenet.app.network.FeedItem
import com.flovenet.app.network.Profile
import com.flovenet.app.ui.DashboardScreen
import com.flovenet.app.ui.FeedScreen
import com.flovenet.app.ui.LoginScreen
import com.flovenet.app.ui.ProfileScreen
import kotlinx.coroutines.launch

enum class Screen { Login, Dashboard, Feed, Profile }

class MainActivity : ComponentActivity() {
    private val graphql = GraphQLClient()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        setContent {
            MaterialTheme(
                colorScheme = darkColorScheme(
                    primary = androidx.compose.ui.graphics.Color(0xFF6C5CE7),
                    secondary = androidx.compose.ui.graphics.Color(0xFF00CEC9),
                    background = androidx.compose.ui.graphics.Color(0xFF0F1117),
                    surface = androidx.compose.ui.graphics.Color(0xFF1A1D2E),
                    onPrimary = androidx.compose.ui.graphics.Color.White,
                    onBackground = androidx.compose.ui.graphics.Color(0xFFE4E6F0),
                    onSurface = androidx.compose.ui.graphics.Color(0xFFE4E6F0),
                    onSurfaceVariant = androidx.compose.ui.graphics.Color(0xFF8B8FA3),
                    error = androidx.compose.ui.graphics.Color(0xFFFF6B6B),
                )
            ) {
                FlovenetApp(graphql)
            }
        }
    }
}

@Composable
fun FlovenetApp(graphql: GraphQLClient) {
    var screen by remember { mutableStateOf(Screen.Login) }
    var token by remember { mutableStateOf<String?>(null) }
    var profile by remember { mutableStateOf<Profile?>(null) }
    var feedItems by remember { mutableStateOf<List<FeedItem>>(emptyList()) }
    var isLoading by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()

    Surface(modifier = Modifier.fillMaxSize()) {
        when (screen) {
            Screen.Login -> LoginScreen(
                onLogin = { email, password ->
                    isLoading = true; error = null
                    scope.launch {
                        try {
                            val result = graphql.login(email, password)
                            if (result != null) {
                                token = result.token
                                profile = result.profile
                                screen = Screen.Dashboard
                            } else error = "Invalid credentials"
                        } catch (e: Exception) { error = e.message }
                        isLoading = false
                    }
                },
                onNavigateRegister = {
                    scope.launch {
                        isLoading = true; error = null
                        try {
                            val result = graphql.register(
                                "android_${System.currentTimeMillis()}@flovenet.app",
                                "android123",
                                "Android_User"
                            )
                            if (result != null) {
                                token = result.token
                                profile = result.profile
                                screen = Screen.Dashboard
                            } else error = "Registration failed"
                        } catch (e: Exception) { error = e.message }
                        isLoading = false
                    }
                },
                isLoading = isLoading, error = error
            )

            Screen.Dashboard -> DashboardScreen(
                profile = profile,
                onLogout = { token = null; profile = null; screen = Screen.Login },
                onNavigateFeed = {
                    scope.launch {
                        isLoading = true
                        try {
                            feedItems = graphql.getFeed() ?: emptyList()
                        } catch (_: Exception) {}
                        isLoading = false
                        screen = Screen.Feed
                    }
                },
                onNavigateProfile = { screen = Screen.Profile }
            )

            Screen.Feed -> FeedScreen(
                items = feedItems,
                onPost = { content ->
                    scope.launch {
                        try {
                            graphql.createPost(content, token ?: return@launch)
                            feedItems = graphql.getFeed() ?: emptyList()
                        } catch (_: Exception) {}
                    }
                },
                isLoading = isLoading
            )

            Screen.Profile -> profile?.let {
                ProfileScreen(profile = it)
            }
        }
    }
}
