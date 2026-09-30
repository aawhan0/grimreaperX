package com.grimreaperx.app

import org.junit.Assert.assertEquals
import org.junit.Test

class UsageRuleEngineTest {
    @Test
    fun matchesSharedEscalationBoundaries() {
        val cases = listOf(
            Triple(19L, EscalationTier.NORMAL, 20L),
            Triple(20L, EscalationTier.MILD, 20L),
            Triple(24L, EscalationTier.MILD, 20L),
            Triple(25L, EscalationTier.SERIOUS, 20L),
            Triple(49L, EscalationTier.SERIOUS, 20L),
            Triple(50L, EscalationTier.UNHINGED, 20L),
        )

        cases.forEach { (durationMinutes, expected, thresholdMinutes) ->
            assertEquals(
                expected,
                UsageRuleEngine.evaluate(durationMinutes * 60, thresholdMinutes),
            )
        }
    }
}