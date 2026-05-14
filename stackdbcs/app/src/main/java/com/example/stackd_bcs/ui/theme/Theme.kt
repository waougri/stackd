package com.example.stackd_bcs.ui.theme

import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext

// Your specific palette
val DeepDarkBackground = Color(0xFF0F0E17) // Near black from icon
val SurfaceColor = Color(0xFF1B1A26)       // Slightly lighter for cards/sheets
val LaserPurple = Color(0xFFB57EDC)        // The vibrant purple from your icon
val LaserBright = Color(0xFFD8B4FE)        // Brighter purple for highlights

val White = Color(0xFFFFFFFF)


private val DarkColorScheme = darkColorScheme(
    primary = LaserPurple, onPrimary = White, primaryContainer = LaserPurple.copy(alpha = 0.3f),

    background = DeepDarkBackground, onBackground = White,

    surface = SurfaceColor, onSurface = White,

    secondary = LaserBright
)

private val LightColorScheme = lightColorScheme(
    primary = LaserPurple, // You can make the light theme white-based if you prefer
    background = White,
    onBackground = DeepDarkBackground,
    surface = White,
    onSurface = DeepDarkBackground
)

@Composable
fun StackdbcsTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    dynamicColor: Boolean = false,
    content: @Composable () -> Unit
) {
    val colorScheme = if (darkTheme) DarkColorScheme else LightColorScheme

    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography, // Ensure this exists
        content = content
    )
}