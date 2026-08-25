package com.netrepublic.aether.ui

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Send
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.netrepublic.aether.AetherConfig
import kotlinx.coroutines.launch

// ═══════════════════════════════════════════
// Modern Aesthetics
// ═══════════════════════════════════════════
private val DeepSpace = Color(0xFF030509)
private val CardGlass = Color(0xCC0F121D)
private val NeonIndigo = Color(0xFF6366F1)
private val NeonCyan = Color(0xFF22D3EE)
private val SuccessGreen = Color(0xFF10B981)
private val ErrorRed = Color(0xFFF43F5E)
private val GlassBorder = Color(0x33FFFFFF)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AetherScreen(
    viewModel: AetherViewModel,
    onRequestVpnPermission: () -> Unit = {}
) {
    val config by viewModel.config.collectAsState()
    val connectionState by viewModel.connectionState.collectAsState()
    val logs by viewModel.logs.collectAsState()
    var settingsExpanded by remember { mutableStateOf(false) }
    val uriHandler = LocalUriHandler.current

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(DeepSpace)
    ) {
        // Dynamic Ambient Orbs
        AnimatedAmbientOrbs()

        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(horizontal = 24.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            contentPadding = PaddingValues(top = 60.dp, bottom = 140.dp)
        ) {
            // High-Tech Header
            item {
                HeaderSection()
                Spacer(Modifier.height(40.dp))
            }

            // Central Power Button
            item {
                ModernPowerButton(
                    state = connectionState,
                    onClick = {
                        if (connectionState != "connecting" && connectionState != "connected") {
                            if (config.connectionMode == "vpn") {
                                onRequestVpnPermission()
                            } else {
                                viewModel.connect()
                            }
                        } else {
                            viewModel.disconnect()
                        }
                    }
                )
                Spacer(Modifier.height(40.dp))
            }

            // Real-time Status Hub
            item {
                StatusHub(connectionState, config)
                Spacer(Modifier.height(24.dp))
            }

            // Dynamic Settings Control
            item {
                GlassSettingsPanel(
                    expanded = settingsExpanded,
                    onToggle = { settingsExpanded = !settingsExpanded },
                    config = config,
                    viewModel = viewModel
                )
                Spacer(Modifier.height(24.dp))
            }

            // Console Stream
            item {
                CyberConsole(logs = logs)
                Spacer(Modifier.height(32.dp))
            }
        }

        // Floating Action Bar (Social & Info)
        SocialNavBar(uriHandler)
    }
}

@Composable
private fun HeaderSection() {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Box(
            modifier = Modifier
                .size(64.dp)
                .background(
                    brush = Brush.linearGradient(listOf(NeonIndigo, NeonCyan)),
                    shape = RoundedCornerShape(18.dp)
                )
                .padding(12.dp),
            contentAlignment = Alignment.Center
        ) {
            Icon(Icons.Default.Security, contentDescription = null, tint = Color.White, modifier = Modifier.size(32.dp))
        }
        Spacer(Modifier.height(16.dp))
        Text(
            text = "NET REPUBLIC",
            style = MaterialTheme.typography.labelLarge,
            color = NeonCyan,
            fontWeight = FontWeight.ExtraBold,
            letterSpacing = 4.sp
        )
        Text(
            text = "Aether Android",
            style = MaterialTheme.typography.titleLarge,
            color = Color.White.copy(alpha = 0.7f),
            fontWeight = FontWeight.Light,
            letterSpacing = (-0.5).sp
        )
    }
}

@Composable
private fun ModernPowerButton(state: String, onClick: () -> Unit) {
    val isConnected = state == "connected"
    val isConnecting = state == "connecting" || state == "reconnecting"
    val isError = state == "error"

    val infiniteTransition = rememberInfiniteTransition(label = "pulse")
    val pulseAlpha by infiniteTransition.animateFloat(
        initialValue = 0.3f,
        targetValue = 0.1f,
        animationSpec = infiniteRepeatable(tween(1500), RepeatMode.Reverse), label = ""
    )
    
    val buttonScale by animateFloatAsState(
        targetValue = if (isConnecting) 1.05f else 1f,
        animationSpec = spring(dampingRatio = 0.5f, stiffness = 400f), label = ""
    )

    Box(contentAlignment = Alignment.Center) {
        // Outer Halo
        if (isConnected || isConnecting) {
            Box(
                modifier = Modifier
                    .size(180.dp)
                    .blur(20.dp)
                    .background(
                        color = (if (isConnected) SuccessGreen else NeonIndigo).copy(alpha = pulseAlpha),
                        shape = CircleShape
                    )
            )
        }

        Surface(
            onClick = onClick,
            modifier = Modifier
                .size(140.dp)
                .scale(buttonScale),
            shape = CircleShape,
            color = when {
                isConnected -> SuccessGreen
                isError -> ErrorRed
                else -> Color.White.copy(alpha = 0.05f)
            },
            border = BorderStroke(
                1.dp,
                if (isConnected || isError) Color.Transparent else GlassBorder
            ),
            shadowElevation = if (isConnected) 40.dp else 0.dp,
            tonalElevation = 8.dp
        ) {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.Center
            ) {
                Icon(
                    imageVector = when {
                        isConnected -> Icons.Default.Shield
                        isError -> Icons.Default.ErrorOutline
                        isConnecting -> Icons.Default.Sync
                        else -> Icons.Default.PowerSettingsNew
                    },
                    contentDescription = null,
                    modifier = Modifier.size(48.dp),
                    tint = if (isConnected || isError) Color.White else NeonIndigo
                )
                Spacer(Modifier.height(8.dp))
                Text(
                    text = when {
                        state == "reconnecting" -> "RETRYING..."
                        isConnecting -> "LINKING..."
                        isConnected -> "PROTECTED"
                        isError -> "FAILED"
                        else -> "CONNECT"
                    },
                    style = MaterialTheme.typography.labelLarge,
                    fontWeight = FontWeight.Black,
                    color = if (isConnected || isError) Color.White else Color.White.copy(alpha = 0.8f)
                )
            }
        }
    }
}

@Composable
private fun StatusHub(state: String, config: AetherConfig) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        StatusTile(
            label = "STATUS",
            value = state.replaceFirstChar { it.uppercase() },
            icon = if (state == "connected") Icons.Default.CheckCircle else Icons.Default.Circle,
            color = when (state) {
                "connected" -> SuccessGreen
                "connecting" -> NeonIndigo
                "error" -> ErrorRed
                else -> Color.White.copy(alpha = 0.4f)
            },
            modifier = Modifier.weight(1f)
        )
        StatusTile(
            label = "MODE",
            value = config.connectionMode.uppercase(),
            icon = if (config.connectionMode == "vpn") Icons.Default.VpnLock else Icons.Default.SettingsEthernet,
            color = NeonCyan,
            modifier = Modifier.weight(1f)
        )
    }
}

@Composable
private fun StatusTile(label: String, value: String, icon: ImageVector, color: Color, modifier: Modifier) {
    Surface(
        modifier = modifier,
        color = CardGlass,
        shape = RoundedCornerShape(24.dp),
        border = BorderStroke(1.dp, GlassBorder)
    ) {
        Row(
            modifier = Modifier.padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .background(color.copy(alpha = 0.1f), CircleShape),
                contentAlignment = Alignment.Center
            ) {
                Icon(icon, contentDescription = null, tint = color, modifier = Modifier.size(20.dp))
            }
            Spacer(Modifier.width(12.dp))
            Column {
                Text(label, style = MaterialTheme.typography.labelSmall, color = Color.White.copy(alpha = 0.4f))
                Text(value, style = MaterialTheme.typography.bodyMedium, color = Color.White, fontWeight = FontWeight.Bold)
            }
        }
    }
}

@Composable
private fun GlassSettingsPanel(
    expanded: Boolean,
    onToggle: () -> Unit,
    config: AetherConfig,
    viewModel: AetherViewModel
) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        color = CardGlass,
        shape = RoundedCornerShape(28.dp),
        border = BorderStroke(1.dp, GlassBorder)
    ) {
        Column {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable { onToggle() }
                    .padding(20.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(Icons.Default.Tune, contentDescription = null, tint = NeonCyan, modifier = Modifier.size(20.dp))
                Spacer(Modifier.width(12.dp))
                Text(
                    "OPTIMIZATION CONTROL",
                    style = MaterialTheme.typography.labelMedium,
                    color = Color.White.copy(alpha = 0.8f),
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 1.sp
                )
                Spacer(Modifier.weight(1f))
                Icon(
                    if (expanded) Icons.Default.KeyboardArrowUp else Icons.Default.KeyboardArrowDown,
                    contentDescription = null,
                    tint = Color.White.copy(alpha = 0.5f)
                )
            }
            
            AnimatedVisibility(
                visible = expanded,
                enter = expandVertically() + fadeIn(),
                exit = shrinkVertically() + fadeOut()
            ) {
                Column(modifier = Modifier.padding(horizontal = 20.dp, vertical = 8.dp)) {
                    HorizontalDivider(color = GlassBorder, modifier = Modifier.padding(bottom = 16.dp))
                    
                    ControlSelector(
                        title = "Network Mode",
                        options = listOf("proxy" to "Proxy", "vpn" to "VPN"),
                        selected = config.connectionMode,
                        onSelect = { viewModel.updateConfig(config.copy(connectionMode = it)) }
                    )
                    Spacer(Modifier.height(24.dp))
                    
                    ControlSelector(
                        title = "Transport Protocol",
                        options = listOf("masque" to "Masque", "wg" to "WG", "gool" to "Gool"),
                        selected = config.protocol,
                        onSelect = { viewModel.updateConfig(config.copy(protocol = it)) }
                    )
                    Spacer(Modifier.height(24.dp))

                    ControlSelector(
                        title = "Scan Profile",
                        options = listOf("turbo" to "Turbo", "balanced" to "Bal.", "thorough" to "Thor.", "stealth" to "Stl.", "ironclad" to "Iron."),
                        selected = config.scanMode,
                        onSelect = { viewModel.updateConfig(config.copy(scanMode = it)) }
                    )
                    Spacer(Modifier.height(24.dp))

                    ControlSelector(
                        title = "Noise Masking",
                        options = listOf("off" to "Off", "light" to "Light", "balanced" to "Bal.", "aggressive" to "Aggr."),
                        selected = config.noizeProfile,
                        onSelect = { viewModel.updateConfig(config.copy(noizeProfile = it)) }
                    )
                    Spacer(Modifier.height(24.dp))
                    
                    Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        CyberToggle("HTTP/2", config.useH2, { viewModel.updateConfig(config.copy(useH2 = it)) }, Modifier.weight(1f))
                        CyberToggle("TLS Frag", config.useFragment, { viewModel.updateConfig(config.copy(useFragment = it)) }, Modifier.weight(1f))
                    }
                    Spacer(Modifier.height(12.dp))
                    CyberToggle("Quick Reconnect", config.quickReconnect, { viewModel.updateConfig(config.copy(quickReconnect = it)) })
                    
                    Spacer(Modifier.height(24.dp))
                }
            }
        }
    }
}

@Composable
private fun ControlSelector(title: String, options: List<Pair<String, String>>, selected: String, onSelect: (String) -> Unit) {
    Column {
        Text(
            title.uppercase(),
            color = Color.White.copy(alpha = 0.4f),
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.ExtraBold,
            modifier = Modifier.padding(start = 4.dp, bottom = 12.dp)
        )
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .background(Color.Black.copy(alpha = 0.3f), RoundedCornerShape(16.dp))
                .padding(4.dp),
            horizontalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            options.forEach { (key, label) ->
                val isSelected = key == selected
                Box(
                    modifier = Modifier
                        .weight(1f)
                        .clip(RoundedCornerShape(12.dp))
                        .clickable { onSelect(key) }
                        .background(if (isSelected) NeonIndigo else Color.Transparent)
                        .padding(vertical = 8.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Text(
                        text = label,
                        color = if (isSelected) Color.White else Color.White.copy(alpha = 0.5f),
                        style = MaterialTheme.typography.labelMedium,
                        fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium
                    )
                }
            }
        }
    }
}

@Composable
private fun CyberToggle(label: String, checked: Boolean, onCheckedChange: (Boolean) -> Unit, modifier: Modifier = Modifier) {
    Surface(
        color = Color.White.copy(alpha = 0.05f),
        shape = RoundedCornerShape(16.dp),
        modifier = modifier
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 12.dp, vertical = 2.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(label, color = Color.White.copy(alpha = 0.8f), style = MaterialTheme.typography.labelMedium, fontWeight = FontWeight.Bold, modifier = Modifier.weight(1f))
            Switch(
                checked = checked,
                onCheckedChange = onCheckedChange,
                colors = SwitchDefaults.colors(
                    checkedThumbColor = Color.White,
                    checkedTrackColor = NeonIndigo,
                    uncheckedTrackColor = Color.White.copy(alpha = 0.1f)
                )
            )
        }
    }
}

@Composable
private fun CyberConsole(logs: List<String>) {
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(max = 220.dp),
        color = Color.Black.copy(alpha = 0.4f),
        shape = RoundedCornerShape(24.dp),
        border = BorderStroke(1.dp, GlassBorder)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Icon(Icons.Default.Terminal, contentDescription = null, tint = NeonCyan, modifier = Modifier.size(16.dp))
                Spacer(Modifier.width(8.dp))
                Text("NETWORK TELEMETRY", color = NeonCyan, style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.Black)
            }
            Spacer(Modifier.height(12.dp))
            val scrollState = rememberScrollState()
            
            // Auto-scroll logic
            LaunchedEffect(logs.size) {
                scrollState.animateScrollTo(scrollState.maxValue)
            }

            Column(modifier = Modifier.verticalScroll(scrollState)) {
                logs.takeLast(50).forEach { line ->
                    Text(
                        text = line,
                        color = when {
                            line.contains("connected", true) -> SuccessGreen
                            line.contains("error", true) || line.contains("failed", true) -> ErrorRed
                            line.contains("VPN", true) -> NeonCyan
                            else -> Color.White.copy(alpha = 0.5f)
                        },
                        fontSize = 9.sp,
                        fontFamily = FontFamily.Monospace,
                        lineHeight = 13.sp,
                        modifier = Modifier.padding(vertical = 1.dp)
                    )
                }
            }
        }
    }
}

@Composable
private fun AnimatedAmbientOrbs() {
    val infiniteTransition = rememberInfiniteTransition(label = "orbs")
    val orbit by infiniteTransition.animateFloat(
        initialValue = 0f,
        targetValue = 360f,
        animationSpec = infiniteRepeatable(tween(20000, easing = LinearEasing)), label = ""
    )

    Box(modifier = Modifier.fillMaxSize()) {
        Box(
            modifier = Modifier
                .size(400.dp)
                .align(Alignment.TopEnd)
                .offset(x = 100.dp, y = (-100).dp)
                .graphicsLayer(rotationZ = orbit)
                .blur(120.dp)
                .background(NeonIndigo.copy(alpha = 0.15f), CircleShape)
        )
        Box(
            modifier = Modifier
                .size(300.dp)
                .align(Alignment.BottomStart)
                .offset(x = (-80).dp, y = 80.dp)
                .graphicsLayer(rotationZ = -orbit)
                .blur(100.dp)
                .background(NeonCyan.copy(alpha = 0.1f), CircleShape)
        )
    }
}

@Composable
private fun SocialNavBar(uriHandler: androidx.compose.ui.platform.UriHandler) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp),
        contentAlignment = Alignment.BottomCenter
    ) {
        Surface(
            color = Color(0xEE0F121D),
            shape = RoundedCornerShape(32.dp),
            border = BorderStroke(1.dp, GlassBorder),
            modifier = Modifier
                .fillMaxWidth()
                .shadow(40.dp, ambientColor = NeonIndigo)
        ) {
            Row(
                modifier = Modifier.padding(8.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                IconButton(
                    onClick = { uriHandler.openUri("https://t.me/net_republic") },
                    modifier = Modifier
                        .clip(CircleShape)
                        .background(NeonIndigo.copy(alpha = 0.1f))
                ) {
                    Icon(Icons.AutoMirrored.Filled.Send, contentDescription = "Telegram", tint = NeonIndigo)
                }

                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text(
                        text = "CORE BY AMIRREZA",
                        style = MaterialTheme.typography.labelSmall,
                        color = Color.White,
                        fontWeight = FontWeight.Black,
                        letterSpacing = 1.sp
                    )
                    Text(
                        text = "VERSION 1.0.0 STABLE",
                        style = MaterialTheme.typography.labelSmall,
                        color = NeonCyan.copy(alpha = 0.7f),
                        fontSize = 7.sp,
                        letterSpacing = 2.sp
                    )
                }

                IconButton(
                    onClick = { uriHandler.openUri("https://github.com/CluvexStudio/Aether") },
                    modifier = Modifier
                        .clip(CircleShape)
                        .background(Color.White.copy(alpha = 0.05f))
                ) {
                    Icon(Icons.Default.Code, contentDescription = "GitHub", tint = Color.White)
                }
            }
        }
    }
}

fun Modifier.scale(scale: Float) = graphicsLayer(scaleX = scale, scaleY = scale)
