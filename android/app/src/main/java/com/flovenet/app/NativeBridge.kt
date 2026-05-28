package com.flovenet.app

object NativeBridge {
    init {
        System.loadLibrary("flovenet_core")
    }

    external fun init(dataDir: String): Boolean
    external fun getPeerId(): String
    external fun getResources(): String
    external fun getPlatform(): String
}
