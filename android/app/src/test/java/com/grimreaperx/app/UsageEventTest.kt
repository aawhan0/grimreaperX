package com.grimreaperx.app

import org.junit.Assert.assertEquals
import org.junit.Test
import java.time.Instant

class UsageEventTest {
    @Test
    fun exposesTheSharedUsageEventFields() {
        val event = UsageEvent(
            id = "event-id",
            deviceId = "android:install-id",
            appName = "Chrome",
            category = "browser",
            startTs = Instant.parse("2026-09-30T10:00:00Z"),
            endTs = Instant.parse("2026-09-30T10:05:00Z"),
        )

        assertEquals(
            setOf("id", "device_id", "app_name", "category", "start_ts", "end_ts"),
            event.toContractFields().keys,
        )
        assertEquals("2026-09-30T10:00:00Z", event.toContractFields()["start_ts"])
    }
}