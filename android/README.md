# Android

The Aether client for Android — Kotlin + Jetpack Compose, wrapping the same
[Aether](https://github.com/CluvexStudio/Aether) core as the desktop app.

## Download

[`app/release/Aether-by-Net-Republic-universal.apk`](app/release/Aether-by-Net-Republic-universal.apk)
— universal build (arm64-v8a, armeabi-v7a, x86_64), ~23 MB.

## Build from source

Requirements: Android Studio (Ladybug or newer), JDK 17, Android SDK 35.

```bash
# one-time: clone the Aether core next to the project for the native tunnel lib
git clone --depth 1 https://github.com/CluvexStudio/Aether.git ../core

./gradlew assembleRelease
# -> app/build/outputs/apk/release/
```

Or just open the `android/` folder in Android Studio and press Run.

## Project layout

```
app/src/main/java/com/netrepublic/aether/
├── AetherVpnService.kt     VPN service: TUN device + routing
├── Tun2SocksProxy.kt       userspace TUN → SOCKS5 forwarding
├── AetherManager.kt        connection state machine
└── ui/                     Compose screens
```

## Notes

- `minSdk 26` (Android 8.0+)
- The prebuilt APK in `app/release/` is what ships; builds from source are for
  development
- `local.properties` is intentionally not committed — Android Studio generates it
