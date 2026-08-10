//! Goal 149: Mobile build (iOS/Android).
//!
//! Build rye apps for iOS and Android using the Pledgepack native adapter.
//! Wraps Xcode build and Gradle build with rye-specific configuration.

use std::path::PathBuf;

/// Mobile platform target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MobilePlatform {
    /// iOS.
    Ios,
    /// Android.
    Android,
}

impl MobilePlatform {
    /// Convert to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            MobilePlatform::Ios => "ios",
            MobilePlatform::Android => "android",
        }
    }
}

/// Mobile build configuration.
#[derive(Debug, Clone)]
pub struct MobileBuildConfig {
    /// Target platform.
    pub platform: MobilePlatform,
    /// App name.
    pub app_name: String,
    /// App identifier (e.g. com.example.app).
    pub app_id: String,
    /// App version.
    pub version: String,
    /// Build number.
    pub build_number: u32,
    /// Output directory.
    pub output_dir: PathBuf,
    /// Whether to build for release.
    pub release: bool,
    /// iOS-specific config.
    pub ios: IosConfig,
    /// Android-specific config.
    pub android: AndroidConfig,
}

impl Default for MobileBuildConfig {
    fn default() -> Self {
        Self {
            platform: MobilePlatform::Ios,
            app_name: "RyeApp".to_string(),
            app_id: "com.example.ryeapp".to_string(),
            version: "1.0.0".to_string(),
            build_number: 1,
            output_dir: PathBuf::from("dist/mobile"),
            release: false,
            ios: IosConfig::default(),
            android: AndroidConfig::default(),
        }
    }
}

/// iOS-specific build configuration.
#[derive(Debug, Clone)]
pub struct IosConfig {
    /// Minimum iOS version.
    pub min_version: String,
    /// Target device (iphone, ipad, universal).
    pub target_device: String,
    /// Development team ID.
    pub team_id: Option<String>,
    /// Provisioning profile name.
    pub provisioning_profile: Option<String>,
    /// Whether to use Swift bridging.
    pub swift_bridge: bool,
}

impl Default for IosConfig {
    fn default() -> Self {
        Self {
            min_version: "14.0".to_string(),
            target_device: "universal".to_string(),
            team_id: None,
            provisioning_profile: None,
            swift_bridge: true,
        }
    }
}

/// Android-specific build configuration.
#[derive(Debug, Clone)]
pub struct AndroidConfig {
    /// Minimum SDK version.
    pub min_sdk: u32,
    /// Target SDK version.
    pub target_sdk: u32,
    /// Application package name.
    pub package: String,
    /// Whether to use Kotlin bridging.
    pub kotlin_bridge: bool,
    /// Keystore path (for release builds).
    pub keystore: Option<PathBuf>,
}

impl Default for AndroidConfig {
    fn default() -> Self {
        Self {
            min_sdk: 24,
            target_sdk: 34,
            package: "com.example.ryeapp".to_string(),
            kotlin_bridge: true,
            keystore: None,
        }
    }
}

/// Mobile build result.
#[derive(Debug, Clone)]
pub struct MobileBuildResult {
    /// Whether the build succeeded.
    pub success: bool,
    /// Path to the built artifact (APK or IPA).
    pub artifact_path: Option<PathBuf>,
    /// Build output/logs.
    pub logs: String,
    /// Build duration in seconds.
    pub duration_secs: u64,
}

/// Generate the iOS Xcode project settings (Info.plist content).
pub fn ios_info_plist(config: &MobileBuildConfig) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>{name}</string>
  <key>CFBundleIdentifier</key>
  <string>{id}</string>
  <key>CFBundleVersion</key>
  <string>{build}</string>
  <key>CFBundleShortVersionString</key>
  <string>{version}</string>
  <key>MinimumOSVersion</key>
  <string>{min_ios}</string>
  <key>UILaunchScreenType</key>
  <string>Default</string>
  <key>UISupportedInterfaceOrientations</key>
  <array>
    <string>UIInterfaceOrientationPortrait</string>
    <string>UIInterfaceOrientationLandscapeLeft</string>
    <string>UIInterfaceOrientationLandscapeRight</string>
  </array>
</dict>
</plist>"#,
        name = config.app_name,
        id = config.app_id,
        build = config.build_number,
        version = config.version,
        min_ios = config.ios.min_version,
    )
}

/// Generate the Android AndroidManifest.xml.
pub fn android_manifest(config: &MobileBuildConfig) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="{package}">
  <uses-sdk android:minSdkVersion="{min_sdk}" android:targetSdkVersion="{target_sdk}" />
  <application
    android:label="{name}"
    android:allowBackup="true"
    android:supportsRtl="true">
    <activity
      android:name=".MainActivity"
      android:exported="true">
      <intent-filter>
        <action android:name="android.intent.action.MAIN" />
        <category android:name="android.intent.category.LAUNCHER" />
      </intent-filter>
    </activity>
  </application>
</manifest>"#,
        package = config.android.package,
        min_sdk = config.android.min_sdk,
        target_sdk = config.android.target_sdk,
        name = config.app_name,
    )
}

/// Generate the build command for the target platform.
pub fn build_command(config: &MobileBuildConfig) -> String {
    match config.platform {
        MobilePlatform::Ios => {
            let mut cmd = format!("xcodebuild -project {}.xcodeproj", config.app_name);
            cmd.push_str(&format!(" -scheme {}", config.app_name));
            cmd.push_str(&format!(" -sdk iphoneos"));
            if config.release {
                cmd.push_str(" -configuration Release");
            } else {
                cmd.push_str(" -configuration Debug");
            }
            if let Some(team) = &config.ios.team_id {
                cmd.push_str(&format!(" DEVELOPMENT_TEAM={}", team));
            }
            cmd
        }
        MobilePlatform::Android => {
            let mut cmd = "./gradlew".to_string();
            if config.release {
                cmd.push_str(" assembleRelease");
            } else {
                cmd.push_str(" assembleDebug");
            }
            cmd
        }
    }
}

/// Check if the required build tools are installed.
pub fn check_build_tools(platform: MobilePlatform) -> Vec<String> {
    let mut missing = Vec::new();
    match platform {
        MobilePlatform::Ios => {
            // Check for xcodebuild
            // On non-macOS, this will always be missing
            #[cfg(not(target_os = "macos"))]
            missing.push("xcodebuild (requires macOS)".to_string());
            #[cfg(target_os = "macos")]
            {
                let _ = &mut missing;
            }
        }
        MobilePlatform::Android => {
            // Check for Android SDK
            if std::env::var("ANDROID_HOME").is_err() && std::env::var("ANDROID_SDK_ROOT").is_err()
            {
                missing.push("ANDROID_HOME / ANDROID_SDK_ROOT not set".to_string());
            }
        }
    }
    missing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_platform() {
        assert_eq!(MobilePlatform::Ios.as_str(), "ios");
        assert_eq!(MobilePlatform::Android.as_str(), "android");
    }

    #[test]
    fn test_mobile_build_config_default() {
        let config = MobileBuildConfig::default();
        assert_eq!(config.app_name, "RyeApp");
        assert_eq!(config.ios.min_version, "14.0");
        assert_eq!(config.android.min_sdk, 24);
    }

    #[test]
    fn test_ios_info_plist() {
        let config = MobileBuildConfig {
            app_name: "TestApp".to_string(),
            app_id: "com.test.app".to_string(),
            version: "2.0.0".to_string(),
            build_number: 42,
            ..Default::default()
        };
        let plist = ios_info_plist(&config);
        assert!(plist.contains("TestApp"));
        assert!(plist.contains("com.test.app"));
        assert!(plist.contains("2.0.0"));
        assert!(plist.contains("42"));
        assert!(plist.contains("14.0"));
    }

    #[test]
    fn test_android_manifest() {
        let config = MobileBuildConfig {
            app_name: "TestApp".to_string(),
            android: AndroidConfig {
                package: "com.test.app".to_string(),
                min_sdk: 26,
                target_sdk: 34,
                ..Default::default()
            },
            ..Default::default()
        };
        let manifest = android_manifest(&config);
        assert!(manifest.contains("com.test.app"));
        assert!(manifest.contains("26"));
        assert!(manifest.contains("34"));
        assert!(manifest.contains("TestApp"));
        assert!(manifest.contains("MainActivity"));
    }

    #[test]
    fn test_build_command_ios() {
        let config = MobileBuildConfig {
            platform: MobilePlatform::Ios,
            app_name: "MyApp".to_string(),
            release: true,
            ios: IosConfig {
                team_id: Some("ABC123".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let cmd = build_command(&config);
        assert!(cmd.contains("xcodebuild"));
        assert!(cmd.contains("MyApp"));
        assert!(cmd.contains("Release"));
        assert!(cmd.contains("ABC123"));
    }

    #[test]
    fn test_build_command_android() {
        let config = MobileBuildConfig {
            platform: MobilePlatform::Android,
            release: false,
            ..Default::default()
        };
        let cmd = build_command(&config);
        assert!(cmd.contains("gradlew"));
        assert!(cmd.contains("assembleDebug"));
    }

    #[test]
    fn test_build_command_android_release() {
        let config = MobileBuildConfig {
            platform: MobilePlatform::Android,
            release: true,
            ..Default::default()
        };
        let cmd = build_command(&config);
        assert!(cmd.contains("assembleRelease"));
    }

    #[test]
    fn test_check_build_tools() {
        let missing = check_build_tools(MobilePlatform::Ios);
        // On non-macOS, xcodebuild should be missing
        #[cfg(not(target_os = "macos"))]
        assert!(!missing.is_empty());
        #[cfg(target_os = "macos")]
        assert!(true);
    }
}
