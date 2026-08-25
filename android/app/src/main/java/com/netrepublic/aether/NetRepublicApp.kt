package com.netrepublic.aether

import android.app.Application
import android.app.NotificationChannel
import android.app.NotificationManager

internal const val CHANNEL_ID = "aether_service"
internal const val CHANNEL_NAME = "Net Republic Service"
internal const val CHANNEL_DESC = "Keeps Net Republic VPN tunnel running"

class NetRepublicApp : Application() {

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    private fun createNotificationChannel() {
        val channel = NotificationChannel(
            CHANNEL_ID,
            CHANNEL_NAME,
            NotificationManager.IMPORTANCE_LOW
        ).apply {
            description = CHANNEL_DESC
            setShowBadge(false)
        }
        val nm = getSystemService(NotificationManager::class.java)
        nm.createNotificationChannel(channel)
    }
}
