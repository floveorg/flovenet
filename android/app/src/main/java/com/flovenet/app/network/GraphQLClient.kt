package com.flovenet.app.network

import com.google.gson.Gson
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.net.HttpURLConnection
import java.net.URL

data class AuthPayload(
    val token: String,
    val profile: Profile
)

data class Profile(
    val peerId: String,
    val displayName: String,
    val bio: String? = null,
    val avatarCid: String? = null,
    val followerCount: Int = 0,
    val followingCount: Int = 0,
    val postCount: Int = 0,
    val reputation: Double? = null
)

data class Post(
    val cid: String,
    val content: String,
    val media: String? = null,
    val timestamp: String
)

data class FeedItem(
    val post: Post,
    val author: Profile,
    val score: Double? = null
)

data class GraphQLResponse<T>(
    val data: T? = null,
    val errors: List<GraphQLError>? = null
)

data class GraphQLError(val message: String)

class GraphQLClient(private val baseUrl: String = "http://10.0.2.2:8080/graphql") {
    private val gson = Gson()

    suspend fun <T> query(
        query: String,
        variables: Map<String, Any> = emptyMap(),
        token: String? = null
    ): T? = withContext(Dispatchers.IO) {
        val url = URL(baseUrl)
        val conn = url.openConnection() as HttpURLConnection
        conn.requestMethod = "POST"
        conn.setRequestProperty("Content-Type", "application/json")
        if (token != null) {
            conn.setRequestProperty("Authorization", "Bearer $token")
        }
        conn.doOutput = true

        val body = mapOf("query" to query, "variables" to variables)
        conn.outputStream.write(gson.toJson(body).toByteArray())

        val response = conn.inputStream.bufferedReader().readText()
        conn.disconnect()

        @Suppress("UNCHECKED_CAST")
        val result = gson.fromJson(response, Map::class.java)
        if (result.containsKey("errors")) {
            throw Exception((result["errors"] as List<Map<String, Any>>).first()["message"] as String)
        }
        result["data"] as? T
    }

    suspend fun register(email: String, password: String, displayName: String): AuthPayload? {
        val query = """mutation {
            register(email: "$email", password: "$password", displayName: "$displayName") {
                token
                profile { peerId displayName bio followerCount followingCount postCount }
            }
        }"""
        @Suppress("UNCHECKED_CAST")
        val data = query<Map<String, Any>>(query)
        return data?.let { parseAuthPayload(it["register"] as Map<String, Any>) }
    }

    suspend fun login(email: String, password: String): AuthPayload? {
        val query = """mutation {
            login(email: "$email", password: "$password") {
                token
                profile { peerId displayName bio followerCount followingCount postCount }
            }
        }"""
        @Suppress("UNCHECKED_CAST")
        val data = query<Map<String, Any>>(query)
        return data?.let { parseAuthPayload(it["login"] as Map<String, Any>) }
    }

    suspend fun getFeed(limit: Int = 50, offset: Int = 0): List<FeedItem>? {
        val query = """query {
            feed(limit: $limit, offset: $offset) {
                post { cid content media timestamp }
                author { peerId displayName }
                score
            }
        }"""
        @Suppress("UNCHECKED_CAST")
        val data = query<Map<String, Any>>(query)
        return data?.let { parseFeed(it["feed"] as List<Map<String, Any>>) }
    }

    suspend fun createPost(content: String, token: String): Post? {
        val query = """mutation {
            createPost(content: "$content") {
                cid content timestamp
            }
        }"""
        @Suppress("UNCHECKED_CAST")
        val data = query<Map<String, Any>>(query, token = token)
        return data?.let { gson.fromJson(gson.toJson(it["createPost"]), Post::class.java) }
    }

    private fun parseAuthPayload(map: Map<String, Any>): AuthPayload {
        val token = map["token"] as String
        val profile = gson.fromJson(gson.toJson(map["profile"]), Profile::class.java)
        return AuthPayload(token, profile)
    }

    private fun parseFeed(items: List<Map<String, Any>>): List<FeedItem> {
        return items.map { gson.fromJson(gson.toJson(it), FeedItem::class.java) }
    }
}
