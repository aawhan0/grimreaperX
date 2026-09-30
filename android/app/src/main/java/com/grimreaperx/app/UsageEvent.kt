package com.grimreaperx.app

import org.json.JSONObject
import java.time.Instant
import java.util.UUID

data class UsageEvent(
    val id: String = UUID.randomUUID().toString(),
    val deviceId: String,
    val appName: String,
    val category: String,
    val startTs: Instant,
    val endTs: Instant,
) {
    fun toContractFields(): Map<String, String> = linkedMapOf(
        "id" to id,
        "device_id" to deviceId,
        "app_name" to appName,
        "category" to category,
        "start_ts" to startTs.toString(),
        "end_ts" to endTs.toString(),
    )

    fun toJson(): JSONObject = JSONObject(toContractFields())
}

fun inferCategory(appName: String): String {
    val name = appName.lowercase()
    return when {
        listOf("chrome", "firefox", "edge", "browser").any(name::contains) -> "browser"
        listOf("spotify", "discord", "slack").any(name::contains) -> "entertainment"
        listOf("code", "visual studio", "sublime", "android studio").any(name::contains) -> "development"
        listOf("steam", "minecraft").any(name::contains) -> "gaming"
        else -> "general"
    }
}