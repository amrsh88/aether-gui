package com.netrepublic.aether

import android.content.Context
import android.os.Build
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import java.io.BufferedReader
import java.io.File
import java.io.InputStreamReader

class AetherManager private constructor(private val context: Context) {

    companion object {
        private const val TAG = "AetherManager"
        private const val BINARY_DIR_NAME = "aether"
        
        @Volatile
        private var instance: AetherManager? = null

        fun getInstance(context: Context): AetherManager {
            return instance ?: synchronized(this) {
                instance ?: AetherManager(context.applicationContext).also { instance = it }
            }
        }

        private const val BINARY_NAME = "aether"
        private const val MAX_LOG_LINES = 500
        private const val FORCE_KILL_TIMEOUT_MS = 2000L
        private const val INITIAL_RECONNECT_DELAY_MS = 2_000L
        private const val MAX_RECONNECT_DELAY_MS = 30_000L

        /** Connection state keywords parsed from stdout — checked longest-first */
        private val STATE_KEYWORDS = linkedMapOf(
            "tunnel ready" to "connected",
            "connected" to "connected",
            "listening on" to "connected",
            "scanning" to "connecting",
            "trying" to "connecting",
            "error" to "error",
            "fatal" to "error",
            "failed" to "error",
            "refused" to "error",
            "denied" to "error",
        )
    }

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    private val processLock = Any()
    private var process: Process? = null

    private val _state = MutableStateFlow("idle")
    val state: StateFlow<String> = _state.asStateFlow()

    private val _logs = MutableStateFlow<List<String>>(emptyList())
    val logs: StateFlow<List<String>> = _logs.asStateFlow()

    private val _isRunning = MutableStateFlow(false)
    val isRunning: StateFlow<Boolean> = _isRunning.asStateFlow()

    // Auto-reconnect state
    private var lastArgs: List<String>? = null
    private var userRequestedStop = false
    private var reconnectDelay = INITIAL_RECONNECT_DELAY_MS
    private val _isReconnecting = MutableStateFlow(false)
    val isReconnecting: StateFlow<Boolean> = _isReconnecting.asStateFlow()
    private var reconnectJob: Job? = null

    private val logBuffer = mutableListOf<String>()

    // ==================================================================
    // Architecture detection
    // ==================================================================

    fun detectArchitecture(): String {
        val abi = Build.SUPPORTED_ABIS.firstOrNull() ?: return "arm64"
        return when {
            abi.contains("arm64") -> "arm64"
            abi.contains("arm") -> "armv7"
            abi.contains("x86_64") || abi.contains("amd64") -> "x86_64"
            abi.contains("x86") -> "x86_64"
            else -> "arm64"
        }
    }

    fun ensureBinaryExtracted(): File {
        val nativeDir = File(context.applicationInfo.nativeLibraryDir)
        val binaryFile = File(nativeDir, "libaether.so")

        appendLog("▸ Checking for binary at: ${binaryFile.absolutePath}")

        if (binaryFile.exists()) {
            // Ensure it's executable
            binaryFile.setExecutable(true, false)
            if (binaryFile.canExecute()) {
                return binaryFile
            } else {
                appendLog("▸ Warning: Binary exists but is not executable.")
            }
        }

        // Fallback for older devices or debugging
        val binDir = File(context.filesDir, "bin")
        val fallbackFile = File(binDir, BINARY_NAME)
        if (fallbackFile.exists() && fallbackFile.canExecute()) {
            return fallbackFile
        }

        throw IllegalStateException("Binary not found at ${binaryFile.absolutePath}. Ensure it is placed in jniLibs as libaether.so")
    }

    // ==================================================================
    // CLI arg builder
    // ==================================================================

    fun buildArgs(config: AetherConfig, tunFd: Int = -1): List<String> {
        val args = mutableListOf<String>()

        if (tunFd != -1) {
            args.addAll(listOf("--tun", tunFd.toString()))
        }

        when (config.protocol.lowercase()) {
            "masque" -> args.add("--masque")
            "wg" -> args.add("--wg")
            "gool" -> args.add("--gool")
        }

        when (config.scanMode.lowercase()) {
            "turbo" -> args.add("--turbo")
            "balanced" -> args.add("--balanced")
            "thorough" -> args.add("--thorough")
            "stealth" -> args.add("--stealth")
            "ironclad" -> args.add("--ironclad")
        }

        when (config.ipVersion.lowercase()) {
            "v4" -> args.add("-4")
            "v6" -> args.add("-6")
            "both" -> args.add("--dual")
        }

        if (config.noizeProfile.lowercase() != "off") {
            args.addAll(listOf("--noize", config.noizeProfile))
        }

        if (config.quickReconnect) {
            args.add("--quick-reconnect")
        }

        if (config.useH2) {
            args.add("--h2")
        }

        if (config.useFragment) {
            args.add("--fragment")
        }

        args.addAll(listOf("--bind", "127.0.0.1:${config.socksPort}"))
        args.addAll(listOf("--log-level", config.logLevel))

        return args
    }

    // ==================================================================
    // Process lifecycle
    // ==================================================================

    /**
     * Start the Aether binary with the given CLI args.
     * Saves args for auto-reconnect on unexpected disconnection.
     */
    fun start(args: List<String>, tunFd: Int = -1) {
        synchronized(processLock) {
            if (_isRunning.value) return

            stopInternal()

            userRequestedStop = false
            lastArgs = args
            reconnectDelay = INITIAL_RECONNECT_DELAY_MS
            _isReconnecting.value = false

            _state.value = "connecting"
            _isRunning.value = true
            clearLogs()

            val binaryFile = try {
                ensureBinaryExtracted()
            } catch (e: Exception) {
                _isRunning.value = false
                _state.value = "error"
                appendLog("▸ Binary extraction failed: ${e.message}")
                return
            }

            val command = listOf(binaryFile.absolutePath) + args
            appendLog("▸ Starting: ${command.joinToString(" ")}")

            // Debug: Check binary help for TUN support
            scope.launch {
                try {
                    val helpFile = File(context.filesDir, "aether_help.txt")
                    val p = ProcessBuilder(binaryFile.absolutePath, "--help").start()
                    p.inputStream.use { input ->
                        helpFile.outputStream().use { output ->
                            input.copyTo(output)
                        }
                    }
                } catch (_: Exception) {}
            }

            val workingDir = File(context.filesDir, BINARY_DIR_NAME)
            workingDir.mkdirs()

            val pb = ProcessBuilder(command)
                .directory(workingDir)
                .redirectErrorStream(false)
                .apply {
                    environment()["HOME"] = context.filesDir.absolutePath
                    if (tunFd != -1) {
                        // Pass the FD to the child process if supported
                    }
                }

            try {
                val proc = pb.start()
                process = proc
                _state.value = "connecting"

                // Read stdout
                scope.launch {
                    try {
                        BufferedReader(InputStreamReader(proc.inputStream)).use { reader ->
                            var line: String?
                            while (reader.readLine().also { line = it } != null) {
                                val text = line ?: continue
                                appendLog(text)
                                parseConnectionState(text)
                            }
                        }
                    } catch (_: Exception) { }
                }

                // Read stderr
                scope.launch {
                    try {
                        BufferedReader(InputStreamReader(proc.errorStream)).use { reader ->
                            var line: String?
                            while (reader.readLine().also { line = it } != null) {
                                val text = line ?: continue
                                appendLog("[ERR] $text")
                                parseConnectionState(text)
                            }
                        }
                    } catch (_: Exception) { }
                }

                // Monitor exit → auto-reconnect
                scope.launch {
                    val exitCode = proc.waitFor()
                    synchronized(processLock) {
                        process = null
                        _isRunning.value = false
                        appendLog("▸ Process exited with code $exitCode")
                    }

                    // Auto-reconnect if not user-initiated
                    if (!userRequestedStop) {
                        _state.value = "reconnecting"
                        scheduleReconnect()
                    } else {
                        _state.value = "disconnected"
                        _isReconnecting.value = false
                    }
                }

            } catch (e: Exception) {
                _isRunning.value = false
                _state.value = "error"
                appendLog("▸ Error: ${e.message}")
            }
        }
    }

    /** Convenience: build args from config, then start. */
    fun start(config: AetherConfig) {
        start(buildArgs(config))
    }

    /**
     * Stop the running Aether process gracefully.
     * Marks as user-initiated so auto-reconnect won't trigger.
     */
    fun stop() {
        synchronized(processLock) {
            userRequestedStop = true
            reconnectJob?.cancel()
            reconnectJob = null
            _isReconnecting.value = false
            stopInternal()
        }
    }

    private fun stopInternal() {
        _state.value = "disconnected"
        _isRunning.value = false
        
        val proc = process ?: return

        appendLog("▸ Stopping process...")

        try {
            proc.destroy() // SIGTERM
        } catch (_: Exception) { }

        // Force kill after timeout
        scope.launch {
            delay(FORCE_KILL_TIMEOUT_MS)
            synchronized(processLock) {
                val p = process
                if (p != null && p.isAlive) {
                    try {
                        p.destroyForcibly() // SIGKILL
                    } catch (_: Exception) { }
                }
                process = null
                _isRunning.value = false
            }
        }
    }

    // ==================================================================
    // Auto-reconnect
    // ==================================================================

    private fun scheduleReconnect() {
        reconnectJob?.cancel()
        _isReconnecting.value = true
        appendLog("▸ Reconnecting in ${reconnectDelay / 1000}s...")

        reconnectJob = scope.launch {
            delay(reconnectDelay)

            // Exponential backoff: 2s → 4s → 8s → 16s → 30s (cap)
            reconnectDelay = (reconnectDelay * 2).coerceAtMost(MAX_RECONNECT_DELAY_MS)

            val savedArgs = lastArgs
            if (savedArgs != null && !userRequestedStop) {
                appendLog("▸ Auto-reconnect attempt...")
                start(savedArgs)
            }
        }
    }

    // ==================================================================
    // Output parsing
    // ==================================================================

    private fun parseConnectionState(line: String) {
        val lower = line.lowercase()
        for ((keyword, stateValue) in STATE_KEYWORDS) {
            if (lower.contains(keyword)) {
                _state.value = stateValue
                // Reset reconnect delay on successful connection
                if (stateValue == "connected") {
                    reconnectDelay = INITIAL_RECONNECT_DELAY_MS
                    _isReconnecting.value = false
                }
                return
            }
        }
    }

    private fun appendLog(line: String) {
        synchronized(logBuffer) {
            logBuffer.add(line)
            if (logBuffer.size > MAX_LOG_LINES) {
                logBuffer.removeAt(0)
            }
            _logs.value = logBuffer.toList()
        }
    }

    private fun clearLogs() {
        synchronized(logBuffer) {
            logBuffer.clear()
            _logs.value = emptyList()
        }
    }

    fun cleanup() {
        stop()
        scope.cancel()
    }

    /**
     * Check if the SOCKS5 proxy is actually listening on the given port.
     */
    suspend fun isSocks5Ready(port: Int): Boolean {
        return try {
            val socket = java.net.Socket()
            socket.connect(java.net.InetSocketAddress("127.0.0.1", port), 500)
            socket.close()
            true
        } catch (e: Exception) {
            false
        }
    }
}