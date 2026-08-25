# Aether binary extraction
-keep class com.netrepublic.aether.AetherManager { *; }

# Keep Compose
-dontwarn androidx.compose.**

# Keep DataStore
-keep class * extends androidx.datastore.preferences.protobuf.GeneratedMessageLite { *; }
