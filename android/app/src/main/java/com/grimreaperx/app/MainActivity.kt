package com.grimreaperx.app

import android.Manifest
import android.app.AppOpsManager
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import android.os.Process
import android.provider.Settings
import android.view.ViewGroup
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.ContextCompat

class MainActivity : AppCompatActivity() {
    private lateinit var statusView: TextView
    private lateinit var permissionButton: Button

    private val notificationPermissionRequest = registerForActivityResult(
        ActivityResultContracts.RequestPermission(),
    ) {
        renderStatus()
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(createContent())
        permissionButton.setOnClickListener(::onPermissionAction)
    }

    override fun onResume() {
        super.onResume()
        renderStatus()
        if (UsageAccess.hasAccess(this)) {
            ContextCompat.startForegroundService(this, Intent(this, UsageTrackingService::class.java))
            requestNotificationPermissionIfNeeded()
        }
    }

    private fun createContent(): LinearLayout {
        val container = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(dp(24), dp(32), dp(24), dp(24))
        }
        val title = TextView(this).apply {
            text = "grimreaperX"
            textSize = 26f
        }
        val explanation = TextView(this).apply {
            text = "Usage Access lets grimreaperX identify the foreground app and measure continuous sessions. Session events are stored locally on this device."
            textSize = 16f
            setPadding(0, dp(16), 0, dp(20))
        }
        statusView = TextView(this).apply {
            textSize = 14f
            setPadding(0, 0, 0, dp(20))
        }
        permissionButton = Button(this)
        container.addView(title, matchWidth())
        container.addView(explanation, matchWidth())
        container.addView(statusView, matchWidth())
        container.addView(permissionButton, matchWidth())
        return container
    }

    private fun renderStatus() {
        val hasUsageAccess = UsageAccess.hasAccess(this)
        val hasNotificationPermission = Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU ||
            ContextCompat.checkSelfPermission(this, Manifest.permission.POST_NOTIFICATIONS) == PackageManager.PERMISSION_GRANTED

        statusView.text = when {
            !hasUsageAccess -> "Usage Access is not enabled. Tracking has not started."
            !hasNotificationPermission -> "Usage Access is enabled. Allow notifications so escalation alerts can be delivered."
            else -> "Usage Access is enabled. Foreground tracking is active."
        }
        permissionButton.text = when {
            !hasUsageAccess -> "Grant Usage Access"
            !hasNotificationPermission -> "Open notification settings"
            else -> "Notification settings"
        }
    }

    private fun onPermissionAction(@Suppress("UNUSED_PARAMETER") view: android.view.View) {
        when {
            !UsageAccess.hasAccess(this) -> {
                startActivity(Intent(Settings.ACTION_USAGE_ACCESS_SETTINGS))
            }
            Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU &&
                ContextCompat.checkSelfPermission(this, Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED -> {
                notificationPermissionRequest.launch(Manifest.permission.POST_NOTIFICATIONS)
            }
            else -> openNotificationSettings()
        }
    }

    private fun requestNotificationPermissionIfNeeded() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU ||
            ContextCompat.checkSelfPermission(this, Manifest.permission.POST_NOTIFICATIONS) == PackageManager.PERMISSION_GRANTED) {
            return
        }

        val preferences = getSharedPreferences("permissions", MODE_PRIVATE)
        if (!preferences.getBoolean("notification_prompted", false)) {
            preferences.edit().putBoolean("notification_prompted", true).apply()
            notificationPermissionRequest.launch(Manifest.permission.POST_NOTIFICATIONS)
        }
    }

    private fun openNotificationSettings() {
        val intent = Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS)
            .putExtra(Settings.EXTRA_APP_PACKAGE, packageName)
        startActivity(intent)
    }

    private fun matchWidth() = LinearLayout.LayoutParams(
        ViewGroup.LayoutParams.MATCH_PARENT,
        ViewGroup.LayoutParams.WRAP_CONTENT,
    )

    private fun dp(value: Int) = (value * resources.displayMetrics.density).toInt()
}

object UsageAccess {
    @Suppress("DEPRECATION")
    fun hasAccess(context: Context): Boolean {
        val appOps = context.getSystemService(Context.APP_OPS_SERVICE) as AppOpsManager
        return appOps.checkOpNoThrow(
            AppOpsManager.OPSTR_GET_USAGE_STATS,
            Process.myUid(),
            context.packageName,
        ) == AppOpsManager.MODE_ALLOWED
    }
}