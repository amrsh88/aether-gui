package com.netrepublic.aether.ui

import android.app.Application
import android.content.Intent
import android.util.Log
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.netrepublic.aether.*
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch

class AetherViewModel(application: Application) : AndroidViewModel(application) {

    private val prefs = AetherPreferences(application)
    private val manager = AetherManager.getInstance(application)

    val config: StateFlow<AetherConfig> = prefs.configFlow()
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), AetherConfig())

    val connectionState: StateFlow<String> = manager.state
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), "idle")

    val logs: StateFlow<List<String>> = manager.logs
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), emptyList())

    fun connect() {
        val ctx = getApplication<Application>()

        viewModelScope.launch {
            try {
                // Fetch config first
                val cfg = config.value
                val args = manager.buildArgs(cfg)

                Log.d("AetherVM", "Initiating connection...")

                // Start foreground service first for notification
                AetherService.startService(ctx, "Linking...", "${cfg.protocol} · ${cfg.scanMode}")

                // Start VPN service if in VPN mode
                if (cfg.connectionMode == "vpn") {
                    val vpnIntent = Intent(ctx, AetherVpnService::class.java).apply {
                        putExtra(AetherVpnService.EXTRA_SOCKS_PORT, cfg.socksPort)
                    }
                    ctx.startForegroundService(vpnIntent)
                }

                // Finally start the binary (this sets state to 'connecting' immediately)
                manager.start(args)
            } catch (e: Exception) {
                Log.e("AetherVM", "Connect failed", e)
            }
        }
    }

    fun disconnect() {
        val ctx = getApplication<Application>()
        try {
            manager.stop()
            AetherService.stopService(ctx)
            
            // Stop VPN Service if active
            val vpnIntent = Intent(ctx, AetherVpnService::class.java).apply {
                action = AetherVpnService.ACTION_STOP
            }
            ctx.startService(vpnIntent)
            
        } catch (e: Exception) {
            Log.e("AetherVM", "Disconnect failed", e)
        }
    }

    fun updateConfig(newConfig: AetherConfig) {
        viewModelScope.launch {
            prefs.saveAll(newConfig)
        }
    }

    override fun onCleared() {
        super.onCleared()
        // We don't cleanup manager here because it's a singleton tied to App lifecycle
    }
}
