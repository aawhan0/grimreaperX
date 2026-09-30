package com.grimreaperx.app

import android.content.Context
import org.json.JSONObject
import kotlin.random.Random

enum class EscalationTier(val key: String) {
    NORMAL("normal"),
    MILD("mild"),
    SERIOUS("serious"),
    UNHINGED("unhinged"),
}

object UsageRuleEngine {
    fun evaluate(durationSeconds: Long, thresholdMinutes: Long): EscalationTier {
        val overThresholdMinutes = durationSeconds / 60 - thresholdMinutes
        return when {
            overThresholdMinutes < 0 -> EscalationTier.NORMAL
            overThresholdMinutes < 5 -> EscalationTier.MILD
            overThresholdMinutes < 30 -> EscalationTier.SERIOUS
            else -> EscalationTier.UNHINGED
        }
    }

    fun chooseMessage(context: Context, tier: EscalationTier, appName: String): String? {
        if (tier == EscalationTier.NORMAL) return null
        val templates = context.assets.open("threat_templates.json").bufferedReader().use {
            JSONObject(it.readText())
        }
        val messages = templates.getJSONArray(tier.key)
        if (messages.length() == 0) return null
        return messages.getString(Random.nextInt(messages.length())).replace("{app}", appName)
    }
}