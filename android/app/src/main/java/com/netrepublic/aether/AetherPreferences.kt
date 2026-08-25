package com.netrepublic.aether

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.intPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.first as firstExtension

// ── DataStore instance (top-level extension on Context) ─────────────────
private val Context.aetherDataStore: DataStore<Preferences> by preferencesDataStore(
    name = "aether_settings",
)

/**
 * Immutable snapshot of every tunable setting in Aether.
 *
 * All defaults match the values required by the Aether CLI unless
 * documented otherwise.
 */
data class AetherConfig(
    val protocol: String = "masque",
    val scanMode: String = "balanced",
    val ipVersion: String = "v4",
    val noizeProfile: String = "balanced",
    val useH2: Boolean = false,
    val useFragment: Boolean = false,
    val quickReconnect: Boolean = true,
    val connectionMode: String = "proxy",
    val socksPort: Int = 1819,
    val logLevel: String = "info",
)

/**
 * Provides a persistent, observable bridge between [AetherConfig] and
 * Android DataStore Preferences.
 *
 * Usage:
 * ```kotlin
 * val prefs = AetherPreferences(context)
 *
 * // One-shot read
 * val config = prefs.config.first()
 *
 * // Reactive observation
 * prefs.observeConfig().collect { config -> … }
 *
 * // Persist a change
 * prefs.saveProtocol("wg")
 * ```
 */
class AetherPreferences(private val context: Context) {

    // ── Preference Keys ──────────────────────────────────────────────
    private object Keys {
        val PROTOCOL = stringPreferencesKey("protocol")
        val SCAN_MODE = stringPreferencesKey("scan_mode")
        val IP_VERSION = stringPreferencesKey("ip_version")
        val NOIZE_PROFILE = stringPreferencesKey("noize_profile")
        val USE_H2 = booleanPreferencesKey("use_h2")
        val USE_FRAGMENT = booleanPreferencesKey("use_fragment")
        val QUICK_RECONNECT = booleanPreferencesKey("quick_reconnect")
        val CONNECTION_MODE = stringPreferencesKey("connection_mode")
        val SOCKS_PORT = intPreferencesKey("socks_port")
        val LOG_LEVEL = stringPreferencesKey("log_level")
    }

    private val ds = context.aetherDataStore

    // ── Read ─────────────────────────────────────────────────────────

    /**
     * Current config snapshot as a cold [Flow]. Completes once with the
     * current values (including defaults for any keys not yet written).
     */
    fun configFlow(): Flow<AetherConfig> = ds.data.map { prefs ->
        prefs.toAetherConfig()
    }

    /**
     * Convenience: first emission only.
     */
    suspend fun config(): AetherConfig = configFlow().firstExtension()

    /**
     * Alias used by Compose collectAsState / produceState helpers.
     */
    fun observeConfig(): Flow<AetherConfig> = configFlow()

    // ── Write helpers (single-key) ───────────────────────────────────

    suspend fun saveProtocol(value: String) {
        require(value in VALID_PROTOCOLS) { "Invalid protocol: $value" }
        ds.edit { it[Keys.PROTOCOL] = value }
    }

    suspend fun saveScanMode(value: String) {
        require(value in VALID_SCAN_MODES) { "Invalid scan mode: $value" }
        ds.edit { it[Keys.SCAN_MODE] = value }
    }

    suspend fun saveIpVersion(value: String) {
        require(value in VALID_IP_VERSIONS) { "Invalid IP version: $value" }
        ds.edit { it[Keys.IP_VERSION] = value }
    }

    suspend fun saveNoizeProfile(value: String) {
        require(value in VALID_NOIZE_PROFILES) { "Invalid noize profile: $value" }
        ds.edit { it[Keys.NOIZE_PROFILE] = value }
    }

    suspend fun saveUseH2(value: Boolean) {
        ds.edit { it[Keys.USE_H2] = value }
    }

    suspend fun saveUseFragment(value: Boolean) {
        ds.edit { it[Keys.USE_FRAGMENT] = value }
    }

    suspend fun saveQuickReconnect(value: Boolean) {
        ds.edit { it[Keys.QUICK_RECONNECT] = value }
    }

    suspend fun saveConnectionMode(value: String) {
        require(value in VALID_CONNECTION_MODES) { "Invalid connection mode: $value" }
        ds.edit { it[Keys.CONNECTION_MODE] = value }
    }

    suspend fun saveSocksPort(value: Int) {
        require(value in PORT_RANGE) { "Port must be in $PORT_RANGE, got $value" }
        ds.edit { it[Keys.SOCKS_PORT] = value }
    }

    suspend fun saveLogLevel(value: String) {
        require(value in VALID_LOG_LEVELS) { "Invalid log level: $value" }
        ds.edit { it[Keys.LOG_LEVEL] = value }
    }

    // ── Bulk write ───────────────────────────────────────────────────

    /**
     * Atomically replace every key in the DataStore with the values from
     * [config]. Keys not present in the data class keep their current
     * value (this never deletes previously-written keys).
     */
    suspend fun saveAll(config: AetherConfig) {
        ds.edit { prefs ->
            prefs[Keys.PROTOCOL] = config.protocol
            prefs[Keys.SCAN_MODE] = config.scanMode
            prefs[Keys.IP_VERSION] = config.ipVersion
            prefs[Keys.NOIZE_PROFILE] = config.noizeProfile
            prefs[Keys.USE_H2] = config.useH2
            prefs[Keys.USE_FRAGMENT] = config.useFragment
            prefs[Keys.QUICK_RECONNECT] = config.quickReconnect
            prefs[Keys.CONNECTION_MODE] = config.connectionMode
            prefs[Keys.SOCKS_PORT] = config.socksPort
            prefs[Keys.LOG_LEVEL] = config.logLevel
        }
    }

    /**
     * Wipe every Aether preference back to factory defaults.
     */
    suspend fun resetAll() {
        ds.edit { prefs ->
            prefs.remove(Keys.PROTOCOL)
            prefs.remove(Keys.SCAN_MODE)
            prefs.remove(Keys.IP_VERSION)
            prefs.remove(Keys.NOIZE_PROFILE)
            prefs.remove(Keys.USE_H2)
            prefs.remove(Keys.USE_FRAGMENT)
            prefs.remove(Keys.QUICK_RECONNECT)
            prefs.remove(Keys.CONNECTION_MODE)
            prefs.remove(Keys.SOCKS_PORT)
            prefs.remove(Keys.LOG_LEVEL)
        }
    }

    // ── Validation helpers ───────────────────────────────────────────

    /**
     * Validate a candidate config without writing it.
     * Returns a list of human-readable error messages; empty means valid.
     */
    fun validate(config: AetherConfig): List<String> {
        val errors = mutableListOf<String>()

        if (config.protocol !in VALID_PROTOCOLS)
            errors += "protocol must be one of $VALID_PROTOCOLS"
        if (config.scanMode !in VALID_SCAN_MODES)
            errors += "scanMode must be one of $VALID_SCAN_MODES"
        if (config.ipVersion !in VALID_IP_VERSIONS)
            errors += "ipVersion must be one of $VALID_IP_VERSIONS"
        if (config.noizeProfile !in VALID_NOIZE_PROFILES)
            errors += "noizeProfile must be one of $VALID_NOIZE_PROFILES"
        if (config.connectionMode !in VALID_CONNECTION_MODES)
            errors += "connectionMode must be one of $VALID_CONNECTION_MODES"
        if (config.socksPort !in PORT_RANGE)
            errors += "socksPort must be in $PORT_RANGE"
        if (config.logLevel !in VALID_LOG_LEVELS)
            errors += "logLevel must be one of $VALID_LOG_LEVELS"

        return errors
    }

    // ── Mapping ──────────────────────────────────────────────────────

    private fun Preferences.toAetherConfig(): AetherConfig = AetherConfig(
        protocol = this[Keys.PROTOCOL] ?: AetherConfig().protocol,
        scanMode = this[Keys.SCAN_MODE] ?: AetherConfig().scanMode,
        ipVersion = this[Keys.IP_VERSION] ?: AetherConfig().ipVersion,
        noizeProfile = this[Keys.NOIZE_PROFILE] ?: AetherConfig().noizeProfile,
        useH2 = this[Keys.USE_H2] ?: AetherConfig().useH2,
        useFragment = this[Keys.USE_FRAGMENT] ?: AetherConfig().useFragment,
        quickReconnect = this[Keys.QUICK_RECONNECT] ?: AetherConfig().quickReconnect,
        connectionMode = this[Keys.CONNECTION_MODE] ?: AetherConfig().connectionMode,
        socksPort = this[Keys.SOCKS_PORT] ?: AetherConfig().socksPort,
        logLevel = this[Keys.LOG_LEVEL] ?: AetherConfig().logLevel,
    )

    // ── Constants ────────────────────────────────────────────────────

    companion object {
        val VALID_PROTOCOLS = setOf("masque", "wg", "gool")
        val VALID_SCAN_MODES = setOf("turbo", "balanced", "thorough", "stealth", "ironclad")
        val VALID_IP_VERSIONS = setOf("v4", "v6", "both")
        val VALID_NOIZE_PROFILES = setOf("off", "light", "balanced", "aggressive")
        val VALID_CONNECTION_MODES = setOf("proxy", "vpn")
        val VALID_LOG_LEVELS = setOf("debug", "info", "warn", "error")
        val PORT_RANGE = 1024..65535
    }
}
