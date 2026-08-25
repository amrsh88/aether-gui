package com.netrepublic.aether

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/**
 * Foreground service that keeps the Aether binary running and displays
 * a persistent notification with connection status.
 *
 * Usage:
 * ```
 * // Start:
 * AetherService.startService(context, "Aether Connected", "masque · balanced")
 *
 * // Stop:
 * AetherService.stopService(context)
 * ```
 */
class AetherService : Service() {

    companion object {
        private const val TAG = "AetherService"
        private const val NOTIFICATION_CHANNEL_ID = "aether_service"
        private const val NOTIFICATION_ID = 1001

        /** Intent extras */
        const val EXTRA_TITLE = "notification_title"
        const val EXTRA_SUBTITLE = "notification_subtitle"

        /** Actions */
        const val ACTION_START = "com.netrepublic.aether.START"
        const val ACTION_STOP = "com.netrepublic.aether.STOP"

        // ── Static state accessible from UI (ViewModel, Compose) ─────

        /** Current connection state: idle | connecting | connected | error | disconnected */
        private val _state = MutableStateFlow("idle")
        val state: StateFlow<String> = _state.asStateFlow()

        /** Rolling log lines from the aether binary */
        private val _logs = MutableStateFlow<List<String>>(emptyList())
        val logs: StateFlow<List<String>> = _logs.asStateFlow()

        /** Whether the service is active */
        private val _isRunning = MutableStateFlow(false)
        val isRunning: StateFlow<Boolean> = _isRunning.asStateFlow()

        /**
         * Start the foreground service with the given notification content.
         *
         * @param context Any context (application context is used internally).
         * @param title Notification title (e.g. "Aether Connected").
         * @param subtitle Notification subtitle (e.g. "masque · balanced").
         */
        fun startService(context: Context, title: String, subtitle: String) {
            val intent = Intent(context.applicationContext, AetherService::class.java).apply {
                action = ACTION_START
                putExtra(EXTRA_TITLE, title)
                putExtra(EXTRA_SUBTITLE, subtitle)
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.applicationContext.startForegroundService(intent)
            } else {
                context.applicationContext.startService(intent)
            }
        }

        /**
         * Stop the foreground service and the underlying aether process.
         */
        fun stopService(context: Context) {
            val intent = Intent(context.applicationContext, AetherService::class.java).apply {
                action = ACTION_STOP
            }
            context.applicationContext.startService(intent)
        }
    }

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private var stateJob: Job? = null

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
        observeState()
    }

    private fun observeState() {
        stateJob?.cancel()
        stateJob = scope.launch {
            val manager = AetherManager.getInstance(this@AetherService)
            val prefs = AetherPreferences(this@AetherService)
            
            manager.state.collect { state ->
                _state.value = state
                val config = prefs.config()
                val title = when(state) {
                    "connected" -> "Net Republic Protected"
                    "connecting" -> "Linking..."
                    "error" -> "Connection Error"
                    else -> "Net Republic"
                }
                val subtitle = "${config.protocol} · ${config.scanMode}"
                updateNotification(title, subtitle)
            }
        }
    }

    private fun updateNotification(title: String, subtitle: String) {
        if (!_isRunning.value) return
        val notification = buildNotification(title, subtitle)
        val nm = getSystemService(NotificationManager::class.java)
        nm.notify(NOTIFICATION_ID, notification)
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_STOP -> {
                _state.value = "disconnected"
                _isRunning.value = false
                stateJob?.cancel()
                stopForeground(STOP_FOREGROUND_REMOVE)
                stopSelf()
                return START_NOT_STICKY
            }

            ACTION_START -> {
                _isRunning.value = true
                val title = intent.getStringExtra(EXTRA_TITLE) ?: "Net Republic"
                val subtitle = intent.getStringExtra(EXTRA_SUBTITLE) ?: ""
                startForegroundWithNotification(title, subtitle)
                return START_STICKY
            }
        }

        return START_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        scope.cancel()
        super.onDestroy()
    }

    // ── Notification ─────────────────────────────────────────────────

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                getString(R.string.notification_channel_name),
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = CHANNEL_DESC
                setShowBadge(false)
            }
            getSystemService(NotificationManager::class.java)
                .createNotificationChannel(channel)
        }
    }

    /**
     * Builds and starts the foreground notification.
     */
    private fun startForegroundWithNotification(title: String, subtitle: String) {
        val notification = buildNotification(title, subtitle)

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            startForeground(
                NOTIFICATION_ID,
                notification,
                ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE
            )
        } else {
            startForeground(NOTIFICATION_ID, notification)
        }
    }

    private fun buildNotification(title: String, subtitle: String): Notification {
        // Tap notification → open MainActivity
        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            Intent(this, MainActivity::class.java).apply {
                flags = Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
            },
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        return NotificationCompat.Builder(this, NOTIFICATION_CHANNEL_ID)
            .setSmallIcon(android.R.drawable.ic_lock_lock)
            .setContentTitle(title)
            .setContentText(subtitle)
            .setOngoing(true)
            .setSilent(true)
            .setContentIntent(pendingIntent)
            .setForegroundServiceBehavior(NotificationCompat.FOREGROUND_SERVICE_IMMEDIATE)
            .build()
    }
}
