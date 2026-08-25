package com.netrepublic.aether.ui.theme

import androidx.compose.ui.graphics.Color

// ── Core Brand Palette ──────────────────────────────────────────────────
val DarkNavy = Color(0xFF0A0E1A)       // Primary background
val DarkNavyLight = Color(0xFF111627)   // Elevated surfaces / cards
val DarkNavySurface = Color(0xFF161C2E) // Dialog / bottom-sheet background
val DarkNavyVariant = Color(0xFF1E2538) // Borders, dividers, subtle fills

val Blue = Color(0xFF3B82F6)            // Primary accent
val BlueLight = Color(0xFF60A5FA)       // Lighter accent / highlight
val BlueDark = Color(0xFF2563EB)        // Pressed / selected states

val Cyan = Color(0xFF06B6D4)            // Secondary accent
val CyanLight = Color(0xFF22D3EE)       // Lighter secondary
val CyanDark = Color(0xFF0891B2)        // Pressed secondary

// ── Semantic / Status Colors ────────────────────────────────────────────
val Success = Color(0xFF22C55E)         // Connected / healthy
val Warning = Color(0xFFFBBF24)         // Warning state
val Error = Color(0xFFEF4444)           // Error / disconnect
val Info = Color(0xFF38BDF8)            // Informational

// ── Text Colors ─────────────────────────────────────────────────────────
val TextPrimary = Color(0xFFE2E8F0)     // Main body text
val TextSecondary = Color(0xFF94A3B8)   // Secondary / muted text
val TextTertiary = Color(0xFF64748B)    // Disabled / placeholder text
val TextOnAccent = Color(0xFFFFFFFF)    // Text rendered on accent buttons

// ── Glassmorphism ───────────────────────────────────────────────────────
val GlassBackground = Color(0x33FFFFFF) // White at 20% – translucent panels
val GlassBorder = Color(0x22FFFFFF)     // White at 13% – subtle border
val GlassHighlight = Color(0x0DFFFFFF)  // White at 5%  – inner glow

// ── Material 3 Color Scheme Mapping (Dark) ─────────────────────────────
val md_theme_dark_primary = Blue
val md_theme_dark_onPrimary = TextOnAccent
val md_theme_dark_primaryContainer = BlueDark
val md_theme_dark_onPrimaryContainer = Color(0xFFD4E4FF)

val md_theme_dark_secondary = Cyan
val md_theme_dark_onSecondary = TextOnAccent
val md_theme_dark_secondaryContainer = CyanDark
val md_theme_dark_onSecondaryContainer = Color(0xFFD5FAFF)

val md_theme_dark_tertiary = BlueLight
val md_theme_dark_onTertiary = TextOnAccent
val md_theme_dark_tertiaryContainer = Color(0xFF1A2744)
val md_theme_dark_onTertiaryContainer = Color(0xFFC4D7FF)

val md_theme_dark_background = DarkNavy
val md_theme_dark_onBackground = TextPrimary

val md_theme_dark_surface = DarkNavyLight
val md_theme_dark_onSurface = TextPrimary
val md_theme_dark_surfaceVariant = DarkNavyVariant
val md_theme_dark_onSurfaceVariant = TextSecondary

val md_theme_dark_error = Error
val md_theme_dark_onError = TextOnAccent
val md_theme_dark_errorContainer = Color(0xFF7F1D1D)
val md_theme_dark_onErrorContainer = Color(0xFFFECACA)

val md_theme_dark_outline = DarkNavyVariant
val md_theme_dark_outlineVariant = GlassBorder

val md_theme_dark_inverseSurface = DarkNavySurface
val md_theme_dark_inverseOnSurface = TextPrimary
val md_theme_dark_inversePrimary = BlueLight
