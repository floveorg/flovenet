//! Flovenet Core — librería compartida multiplataforma.
//!
//! # Uso desde Android (JNI)
//! ```kotlin
//! class NativeBridge {
//!     external fun init(dataDir: String): Boolean
//!     external fun getPeerId(): String
//!     external fun getResources(): String  // JSON
//!     external fun getPlatform(): String
//! }
//! ```

#![cfg_attr(target_os = "android", allow(dead_code))]

pub use resource_manager::{
    default_cache_dir, default_data_dir, gpu::GpuSlot, NodeResources, Platform,
};

/// Core Flovenet node — wrapping identity, resource detection, and data dirs.
pub struct FlovenetNode {
    pub peer_id: String,
    pub resources: NodeResources,
    pub platform: Platform,
    pub data_dir: std::path::PathBuf,
}

impl FlovenetNode {
    /// Create a new Flovenet node, detecting local resources and generating keys.
    pub async fn new() -> anyhow::Result<Self> {
        let peer_id = format!("flovenet-{}", uuid::Uuid::new_v4());
        let resources = NodeResources::detect();
        let platform = Platform::detect();
        let data_dir = default_data_dir();

        Ok(Self {
            peer_id,
            resources,
            platform,
            data_dir,
        })
    }

    /// Detect and return current node resources.
    pub fn detect_resources(&self) -> NodeResources {
        NodeResources::detect()
    }

    /// Return the peer ID as a string.
    pub fn peer_id_str(&self) -> String {
        self.peer_id.clone()
    }

    /// Return platform identifier string.
    pub fn platform_str(&self) -> String {
        self.platform.as_str().to_string()
    }

    /// Return resources as JSON.
    pub fn resources_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.resources).unwrap_or_default()
    }
}

// ─── Android JNI Bridge ───────────────────────────────────────

#[cfg(target_os = "android")]
pub mod android {
    use super::*;
    use jni::objects::{JClass, JString};
    use jni::sys::{jboolean, jstring, JNI_FALSE, JNI_TRUE};
    use jni::JNIEnv;
    use std::sync::OnceLock;

    static NODE: OnceLock<FlovenetNode> = OnceLock::new();

    #[no_mangle]
    pub extern "system" fn Java_com_flovenet_app_NativeBridge_init(
        mut env: JNIEnv,
        _class: JClass,
        data_dir: JString,
    ) -> jboolean {
        let dir: String = match env.get_string(&data_dir) {
            Ok(s) => s.into(),
            Err(_) => return JNI_FALSE,
        };
        std::env::set_var("FLOVENET_PLATFORM", "android");
        std::env::set_var("FLOVENET_DATA_DIR", &dir);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let node = rt.block_on(FlovenetNode::new());
        match node {
            Ok(node) => {
                let _ = NODE.set(node);
                JNI_TRUE
            }
            Err(e) => {
                tracing::error!("Failed to init FlovenetNode: {e}");
                JNI_FALSE
            }
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_com_flovenet_app_NativeBridge_getPeerId(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        let peer_id = NODE.get().map(|n| n.peer_id_str()).unwrap_or_default();
        env.new_string(peer_id)
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut())
    }

    #[no_mangle]
    pub extern "system" fn Java_com_flovenet_app_NativeBridge_getResources(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        let json = NODE
            .get()
            .map(|n| n.resources_json().to_string())
            .unwrap_or_else(|| "{}".to_string());
        env.new_string(json)
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut())
    }

    #[no_mangle]
    pub extern "system" fn Java_com_flovenet_app_NativeBridge_getPlatform(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        let platform = NODE
            .get()
            .map(|n| n.platform_str())
            .unwrap_or_else(|| "android".to_string());
        env.new_string(platform)
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut())
    }
}
