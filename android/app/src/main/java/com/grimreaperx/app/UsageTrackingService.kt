package com.grimreaperx.app

import android.Manifest
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.app.usage.UsageEvents
import android.app.usage.UsageStatsManager
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationManagerCompat
import java.io.File
import java.time.Instant
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import java.util.UUID

class UsageTrackingService : Service() {
    private val executor = Executors.newSingleThreadScheduledExecutor()
    private val trackingChannelId = "grimreaperx_tracking"
    private val threatChannelId = "grimreaperx_threats"
    private var currentPackage: String? = null
    private var currentAppName: String? = null
    private var currentCategory: String? = null
    private var sessionStartedAt = 0L
    private var lastDeliveredTier = EscalationTier.NORMAL

    override fun onCreate() {
        super.onCreate()
        createChannels()
        startForeground(TRACKING_NOTIFICATION_ID, trackingNotification("Preparing usage tracking"))
        executor.scheduleAtFixedRate({
            try {
                pollUsage()
            } catch (error: Exception) {
                Log.e(TAG, "Usage tracking poll failed", error)
            }
        }, 0, POLL_INTERVAL_SECONDS, TimeUnit.SECONDS)
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        updateTrackingNotification(
            if (UsageAccess.hasAccess(this)) "Monitoring foreground app usage" else "Usage Access is required",
        )
        return START_STICKY
    }

    private fun pollUsage() {
        if (!UsageAccess.hasAccess(this)) {
            closeSession(System.currentTimeMillis())
            updateTrackingNotification("Usage Access is required")
            return
        }

        val now = System.currentTimeMillis()
        val packageName = foregroundPackage(now)
        if (packageName == null) {
            closeSession(now)
            return
        }

        if (packageName != currentPackage) {
            closeSession(now)
            currentPackage = packageName
            currentAppName = applicationLabel(packageName)
            currentCategory = inferCategory(currentAppName.orEmpty())
            sessionStartedAt = now
            lastDeliveredTier = EscalationTier.NORMAL
        }

        evaluateSession(now)
    }

    private fun foregroundPackage(now: Long): String? {
        val manager = getSystemService(Context.USAGE_STATS_SERVICE) as UsageStatsManager
        val events = manager.queryEvents(now - EVENT_LOOKBACK_MILLIS, now)
        val event = UsageEvents.Event()
        var activePackage: String? = null

        while (events.hasNextEvent()) {
            events.getNextEvent(event)
            val eventPackage = event.packageName ?: continue
            val isForeground = event.eventType == UsageEvents.Event.ACTIVITY_RESUMED ||
                event.eventType == UsageEvents.Event.MOVE_TO_FOREGROUND
            val isBackground = event.eventType == UsageEvents.Event.ACTIVITY_PAUSED ||
                event.eventType == UsageEvents.Event.MOVE_TO_BACKGROUND

            if (isForeground) {
                activePackage = eventPackage
            } else if (isBackground && activePackage == eventPackage) {
                activePackage = null
            } else if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P &&
                (event.eventType == UsageEvents.Event.KEYGUARD_SHOWN ||
                    event.eventType == UsageEvents.Event.SCREEN_NON_INTERACTIVE)) {
                activePackage = null
            }
        }

        return activePackage?.takeUnless {
            it == packageName || it == "com.android.systemui"
        }
    }

    private fun evaluateSession(now: Long) {
        val appName = currentAppName ?: return
        val threshold = getSharedPreferences("tracking", MODE_PRIVATE)
            .getLong("threshold_minutes", DEFAULT_THRESHOLD_MINUTES)
        val elapsedSeconds = (now - sessionStartedAt).coerceAtLeast(0) / 1000
        val tier = UsageRuleEngine.evaluate(elapsedSeconds, threshold)

        if (tier.ordinal <= lastDeliveredTier.ordinal || tier == EscalationTier.NORMAL) return
        val message = UsageRuleEngine.chooseMessage(this, tier, appName) ?: return
        if (deliverThreat(tier, message)) lastDeliveredTier = tier
    }

    private fun deliverThreat(tier: EscalationTier, message: String): Boolean {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU &&
            checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) {
            return false
        }
        if (!NotificationManagerCompat.from(this).areNotificationsEnabled()) return false

        val openApp = PendingIntent.getActivity(
            this,
            0,
            Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        val notification = Notification.Builder(this, threatChannelId)
            .setContentTitle("grimreaperX · ${tier.key}")
            .setContentText(message)
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setCategory(Notification.CATEGORY_REMINDER)
            .setPriority(Notification.PRIORITY_HIGH)
            .setAutoCancel(true)
            .setContentIntent(openApp)
            .build()

        getSystemService(NotificationManager::class.java).notify(THREAT_NOTIFICATION_ID + tier.ordinal, notification)
        return true
    }

    private fun closeSession(endedAt: Long) {
        val appName = currentAppName
        val category = currentCategory
        if (appName != null && category != null && sessionStartedAt > 0) {
            val event = UsageEvent(
                deviceId = androidDeviceId(),
                appName = appName,
                category = category,
                startTs = Instant.ofEpochMilli(sessionStartedAt),
                endTs = Instant.ofEpochMilli(endedAt.coerceAtLeast(sessionStartedAt)),
            )
            try {
                File(filesDir, USAGE_EVENTS_FILE).appendText(event.toJson().toString() + "\n", Charsets.UTF_8)
            } catch (error: Exception) {
                Log.e(TAG, "Could not persist usage event", error)
            }
        }

        currentPackage = null
        currentAppName = null
        currentCategory = null
        sessionStartedAt = 0L
        lastDeliveredTier = EscalationTier.NORMAL
    }

    private fun applicationLabel(packageName: String): String = try {
        val appInfo = packageManager.getApplicationInfo(packageName, 0)
        packageManager.getApplicationLabel(appInfo).toString()
    } catch (_: PackageManager.NameNotFoundException) {
        packageName
    }

    private fun androidDeviceId(): String {
        val preferences = getSharedPreferences("tracking", MODE_PRIVATE)
        val savedId = preferences.getString("device_id", null)
        if (savedId != null) return savedId
        val deviceId = "android:${UUID.randomUUID()}"
        preferences.edit().putString("device_id", deviceId).apply()
        return deviceId
    }

    private fun createChannels() {
        val manager = getSystemService(NotificationManager::class.java)
        manager.createNotificationChannel(
            NotificationChannel(trackingChannelId, "grimreaperX tracking", NotificationManager.IMPORTANCE_LOW),
        )
        manager.createNotificationChannel(
            NotificationChannel(threatChannelId, "Screen-time alerts", NotificationManager.IMPORTANCE_HIGH),
        )
    }

    private fun trackingNotification(text: String): Notification = Notification.Builder(this, trackingChannelId)
        .setContentTitle("grimreaperX")
        .setContentText(text)
        .setSmallIcon(android.R.drawable.ic_dialog_info)
        .setOngoing(true)
        .build()

    private fun updateTrackingNotification(text: String) {
        getSystemService(NotificationManager::class.java)
            .notify(TRACKING_NOTIFICATION_ID, trackingNotification(text))
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        executor.shutdownNow()
        try {
            executor.awaitTermination(1, TimeUnit.SECONDS)
        } catch (_: InterruptedException) {
            Thread.currentThread().interrupt()
        }
        closeSession(System.currentTimeMillis())
        super.onDestroy()
    }

    companion object {
        private const val TAG = "grimreaperX"
        private const val POLL_INTERVAL_SECONDS = 5L
        private const val EVENT_LOOKBACK_MILLIS = 24 * 60 * 60 * 1000L
        private const val TRACKING_NOTIFICATION_ID = 1001
        private const val THREAT_NOTIFICATION_ID = 2000
        private const val DEFAULT_THRESHOLD_MINUTES = 20L
        private const val USAGE_EVENTS_FILE = "usage_events.jsonl"
    }
}