package com.netrepublic.aether

import android.app.Notification
import android.app.NotificationManager
import android.content.Intent
import android.content.pm.ServiceInfo
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import android.util.Log
import com.wgtunnel.hevtunnel.TProxyService
import java.io.Closeable
import java.io.File
import java.io.FileOutputStream
import java.net.InetAddress
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

/**
 * Advanced VPN service using hev-socks5-tunnel for system-wide routing.
 */
class AetherVpnService : VpnService(), Closeable {

    companion object {
        const val CHANNEL_ID = com.netrepublic.aether.CHANNEL_ID
        const val NOTIFICATION_ID = 1338
        const val ACTION_STOP = "com.netrepublic.aether.VPN_STOP"
        const val EXTRA_SOCKS_PORT = "extra_socks_port"
        private const val TAG = "AetherVpn"
    }

    private var vpnInterface: ParcelFileDescriptor? = null
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private var vpnJob: Job? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_STOP) {
            stopVpn()
            return START_NOT_STICKY
        }

        val socksPort = intent?.getIntExtra(EXTRA_SOCKS_PORT, 1819) ?: 1819
        
        val notification = buildNotification()
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            startForeground(NOTIFICATION_ID, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE)
        } else {
            startForeground(NOTIFICATION_ID, notification)
        }

        startVpn(socksPort)
        return START_STICKY
    }

    private fun startVpn(socksPort: Int) {
        vpnJob?.cancel()
        vpnJob = scope.launch {
            Log.d(TAG, "[AETHER-VPN] Starting VPN")
            try {
                // 1. Wait for SOCKS5 to be ready
                val manager = AetherManager.getInstance(this@AetherVpnService)
                Log.d(TAG, "[AETHER-VPN] Waiting for SOCKS5 on port $socksPort")
                
                var attempts = 0
                while (!manager.isSocks5Ready(socksPort) && attempts < 20) {
                    delay(500)
                    attempts++
                }
                
                if (!manager.isSocks5Ready(socksPort)) {
                    Log.e(TAG, "[AETHER-VPN] SOCKS5 not ready after timeout")
                    stopSelf()
                    return@launch
                }
                Log.d(TAG, "[AETHER-VPN] SOCKS5 ready on 127.0.0.1:$socksPort")

                // 2. Build TUN interface
                val builder = Builder()
                    .setSession("Net Republic Tunnel")
                    .setMtu(1500)
                    .addAddress("10.0.0.2", 32)
                    .addRoute("0.0.0.0", 0)
                    .addDnsServer("1.1.1.1")
                    .addDnsServer("8.8.8.8")
                    
                // Exclude Aether App itself to prevent routing loops
                try {
                    builder.addDisallowedApplication(packageName)
                } catch (e: Exception) {
                    Log.w(TAG, "Could not add disallowed application", e)
                }

                vpnInterface = builder.establish()
                
                if (vpnInterface == null) {
                    Log.e(TAG, "[AETHER-VPN] Failed to establish TUN interface")
                    stopSelf()
                    return@launch
                }
                
                val tunFd = vpnInterface!!.fd
                Log.d(TAG, "[AETHER-VPN] TUN created, fd = $tunFd")

                // 3. Prepare hev-socks5-tunnel config
                val configFile = File(cacheDir, "tunnel.yml")
                val configContent = """
                    tunnel:
                      name: tun0
                      mtu: 1500
                      ipv4: 10.0.0.2
                    
                    socks5:
                      port: $socksPort
                      address: 127.0.0.1
                      udp: true
                """.trimIndent()
                
                FileOutputStream(configFile).use { it.write(configContent.toByteArray()) }

                // 4. Start native tunnel
                Log.d(TAG, "[AETHER-VPN] Starting hev-socks5-tunnel")
                TProxyService.TProxyStartService(configFile.absolutePath, tunFd)
                
                Log.d(TAG, "[AETHER-VPN] VPN CONNECTED")
                
            } catch (e: Exception) {
                Log.e(TAG, "[AETHER-VPN] VPN Start Error", e)
                stopSelf()
            }
        }
    }

    private fun stopVpn() {
        Log.d(TAG, "[AETHER-VPN] Stopping tunnel")
        vpnJob?.cancel()
        try {
            TProxyService.TProxyStopService()
        } catch (e: Exception) {
            Log.e(TAG, "Error stopping tunnel", e)
        }
        close()
        Log.d(TAG, "[AETHER-VPN] VPN DISCONNECTED")
        stopForeground(STOP_FOREGROUND_REMOVE)
        stopSelf()
    }

    override fun onDestroy() {
        stopVpn()
        super.onDestroy()
    }

    override fun close() {
        try {
            vpnInterface?.close()
        } catch (_: Exception) {}
        vpnInterface = null
    }

    private fun buildNotification(): Notification {
        return Notification.Builder(this, CHANNEL_ID)
            .setContentTitle("Net Republic VPN")
            .setContentText("System-wide protection active")
            .setSmallIcon(android.R.drawable.ic_lock_lock)
            .setOngoing(true)
            .build()
    }
}
