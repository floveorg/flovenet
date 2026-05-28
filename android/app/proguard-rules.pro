# Flovenet Android App
# ProGuard rules for release builds

-keep class com.flovenet.app.NativeBridge { *; }

# Keep Gson serialization classes
-keepclassmembers class com.flovenet.app.network.** { *; }
-keep class com.flovenet.app.network.** { *; }
