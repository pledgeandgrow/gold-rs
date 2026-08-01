//! Goal 150: Deploy pipeline.
//!
//! `rye deploy` command — build and deploy to various targets:
//! static hosting (Netlify, Vercel), edge (Cloudflare), or self-hosted.

use std::path::PathBuf;

/// Deployment target.
#[derive(Debug, Clone, PartialEq)]
pub enum DeployTarget {
    /// Netlify static hosting.
    Netlify,
    /// Vercel.
    Vercel,
    /// Cloudflare Pages.
    CloudflarePages,
    /// Cloudflare Workers.
    CloudflareWorkers,
    /// Self-hosted (copy files to server).
    SelfHosted,
    /// GitHub Pages.
    GithubPages,
    /// Custom deploy command.
    Custom(String),
}

impl DeployTarget {
    /// Parse from a string.
    pub fn from_str(s: &str) -> Self {
        match s {
            "netlify" => Self::Netlify,
            "vercel" => Self::Vercel,
            "cloudflare" | "cf-pages" => Self::CloudflarePages,
            "cf-workers" => Self::CloudflareWorkers,
            "self" | "self-hosted" => Self::SelfHosted,
            "github" | "gh-pages" => Self::GithubPages,
            other => Self::Custom(other.to_string()),
        }
    }

    /// Convert to string.
    pub fn as_str(&self) -> String {
        match self {
            Self::Netlify => "netlify".to_string(),
            Self::Vercel => "vercel".to_string(),
            Self::CloudflarePages => "cloudflare-pages".to_string(),
            Self::CloudflareWorkers => "cloudflare-workers".to_string(),
            Self::SelfHosted => "self-hosted".to_string(),
            Self::GithubPages => "github-pages".to_string(),
            Self::Custom(s) => s.clone(),
        }
    }
}

/// Deploy configuration.
#[derive(Debug, Clone)]
pub struct DeployConfig {
    /// Target platform.
    pub target: DeployTarget,
    /// Build output directory.
    pub output_dir: PathBuf,
    /// Whether to build before deploying.
    pub build_first: bool,
    /// Environment (production, staging, preview).
    pub environment: DeployEnvironment,
    /// Site name or project name.
    pub site_name: Option<String>,
    /// Custom deploy command (for Custom target).
    pub custom_command: Option<String>,
    /// Environment variables to set.
    pub env_vars: Vec<(String, String)>,
}

impl Default for DeployConfig {
    fn default() -> Self {
        Self {
            target: DeployTarget::Netlify,
            output_dir: PathBuf::from("dist"),
            build_first: true,
            environment: DeployEnvironment::Production,
            site_name: None,
            custom_command: None,
            env_vars: Vec::new(),
        }
    }
}

/// Deployment environment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeployEnvironment {
    /// Production.
    Production,
    /// Staging.
    Staging,
    /// Preview/PR deploy.
    Preview,
}

impl DeployEnvironment {
    /// Convert to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            DeployEnvironment::Production => "production",
            DeployEnvironment::Staging => "staging",
            DeployEnvironment::Preview => "preview",
        }
    }
}

/// Deploy result.
#[derive(Debug, Clone)]
pub struct DeployResult {
    /// Whether the deploy succeeded.
    pub success: bool,
    /// URL of the deployed site.
    pub url: Option<String>,
    /// Deploy logs.
    pub logs: String,
    /// Deploy duration in seconds.
    pub duration_secs: u64,
}

/// Generate the deploy command for the target platform.
pub fn deploy_command(config: &DeployConfig) -> String {
    match &config.target {
        DeployTarget::Netlify => {
            let mut cmd = "npx netlify deploy".to_string();
            cmd.push_str(&format!(" --dir={}", config.output_dir.display()));
            if config.environment == DeployEnvironment::Production {
                cmd.push_str(" --prod");
            }
            cmd
        }
        DeployTarget::Vercel => {
            let mut cmd = "npx vercel".to_string();
            if config.environment == DeployEnvironment::Production {
                cmd.push_str(" --prod");
            }
            cmd
        }
        DeployTarget::CloudflarePages => {
            format!("npx wrangler pages deploy {}", config.output_dir.display())
        }
        DeployTarget::CloudflareWorkers => {
            "npx wrangler deploy".to_string()
        }
        DeployTarget::SelfHosted => {
            format!("rsync -avz {}/ user@server:/var/www/html/", config.output_dir.display())
        }
        DeployTarget::GithubPages => {
            format!("npx gh-pages -d {}", config.output_dir.display())
        }
        DeployTarget::Custom(cmd) => cmd.clone(),
    }
}

/// Generate a Netlify configuration file (netlify.toml).
pub fn netlify_toml(config: &DeployConfig) -> String {
    format!(
        r#"[build]
  publish = "{output}"
  command = "rye build --release"

[[redirects]]
  from = "/*"
  to = "/index.html"
  status = 200

[[headers]]
  for = "/wasm/*"
  [headers.values]
    Content-Type = "application/wasm"
    Cache-Control = "public, max-age=31536000, immutable"

[[headers]]
  for = "/*.js"
  [headers.values]
    Cache-Control = "public, max-age=31536000, immutable"
"#,
        output = config.output_dir.display(),
    )
}

/// Generate a Vercel configuration file (vercel.json).
pub fn vercel_json(config: &DeployConfig) -> String {
    format!(
        r#"{{
  "buildCommand": "rye build --release",
  "outputDirectory": "{output}",
  "rewrites": [
    {{ "source": "/(.*)", "destination": "/index.html" }}
  ],
  "headers": [
    {{
      "source": "/wasm/(.*)",
      "headers": [
        {{ "key": "Content-Type", "value": "application/wasm" }},
        {{ "key": "Cache-Control", "value": "public, max-age=31536000, immutable" }}
      ]
    }}
  ]
}}"#,
        output = config.output_dir.display(),
    )
}

/// Generate a Cloudflare Pages configuration (wrangler.toml).
pub fn wrangler_toml(config: &DeployConfig) -> String {
    let name = config.site_name.as_deref().unwrap_or("rye-app");
    format!(
        r#"name = "{name}"
pages_build_output_dir = "{output}"
compatibility_date = "2024-01-01"

[env.production]
name = "{name}"
"#,
        name = name,
        output = config.output_dir.display(),
    )
}

/// Pre-deploy checks.
pub fn pre_deploy_checks(config: &DeployConfig) -> Vec<String> {
    let mut warnings = Vec::new();

    // Check if output directory exists
    if !config.output_dir.exists() {
        warnings.push(format!(
            "Output directory '{}' does not exist — run 'rye build' first",
            config.output_dir.display()
        ));
    }

    // Check for required tools
    match &config.target {
        DeployTarget::Netlify => {
            warnings.push("Ensure 'netlify-cli' is installed: npm i -g netlify-cli".to_string());
        }
        DeployTarget::Vercel => {
            warnings.push("Ensure 'vercel' is installed: npm i -g vercel".to_string());
        }
        DeployTarget::CloudflarePages | DeployTarget::CloudflareWorkers => {
            warnings.push("Ensure 'wrangler' is installed: npm i -g wrangler".to_string());
        }
        DeployTarget::GithubPages => {
            warnings.push("Ensure 'gh-pages' is installed: npm i -g gh-pages".to_string());
        }
        _ => {}
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deploy_target_from_str() {
        assert_eq!(DeployTarget::from_str("netlify"), DeployTarget::Netlify);
        assert_eq!(DeployTarget::from_str("vercel"), DeployTarget::Vercel);
        assert_eq!(DeployTarget::from_str("cloudflare"), DeployTarget::CloudflarePages);
        assert_eq!(DeployTarget::from_str("cf-workers"), DeployTarget::CloudflareWorkers);
        assert_eq!(DeployTarget::from_str("self"), DeployTarget::SelfHosted);
        assert_eq!(DeployTarget::from_str("github"), DeployTarget::GithubPages);
        assert!(matches!(DeployTarget::from_str("custom-target"), DeployTarget::Custom(_)));
    }

    #[test]
    fn test_deploy_command_netlify() {
        let config = DeployConfig {
            target: DeployTarget::Netlify,
            output_dir: PathBuf::from("dist"),
            environment: DeployEnvironment::Production,
            ..Default::default()
        };
        let cmd = deploy_command(&config);
        assert!(cmd.contains("netlify deploy"));
        assert!(cmd.contains("--dir=dist"));
        assert!(cmd.contains("--prod"));
    }

    #[test]
    fn test_deploy_command_vercel() {
        let config = DeployConfig {
            target: DeployTarget::Vercel,
            environment: DeployEnvironment::Production,
            ..Default::default()
        };
        let cmd = deploy_command(&config);
        assert!(cmd.contains("vercel"));
        assert!(cmd.contains("--prod"));
    }

    #[test]
    fn test_deploy_command_cloudflare_pages() {
        let config = DeployConfig {
            target: DeployTarget::CloudflarePages,
            output_dir: PathBuf::from("dist"),
            ..Default::default()
        };
        let cmd = deploy_command(&config);
        assert!(cmd.contains("wrangler pages deploy"));
        assert!(cmd.contains("dist"));
    }

    #[test]
    fn test_deploy_command_custom() {
        let config = DeployConfig {
            target: DeployTarget::Custom("my-deploy-script.sh".to_string()),
            ..Default::default()
        };
        let cmd = deploy_command(&config);
        assert_eq!(cmd, "my-deploy-script.sh");
    }

    #[test]
    fn test_netlify_toml() {
        let config = DeployConfig {
            output_dir: PathBuf::from("dist"),
            ..Default::default()
        };
        let toml = netlify_toml(&config);
        assert!(toml.contains("[build]"));
        assert!(toml.contains("publish = \"dist\""));
        assert!(toml.contains("rye build"));
        assert!(toml.contains("application/wasm"));
    }

    #[test]
    fn test_vercel_json() {
        let config = DeployConfig {
            output_dir: PathBuf::from("dist"),
            ..Default::default()
        };
        let json = vercel_json(&config);
        assert!(json.contains("outputDirectory"));
        assert!(json.contains("dist"));
        assert!(json.contains("rewrites"));
    }

    #[test]
    fn test_wrangler_toml() {
        let config = DeployConfig {
            output_dir: PathBuf::from("dist"),
            site_name: Some("my-app".to_string()),
            ..Default::default()
        };
        let toml = wrangler_toml(&config);
        assert!(toml.contains("my-app"));
        assert!(toml.contains("pages_build_output_dir"));
    }

    #[test]
    fn test_pre_deploy_checks() {
        let config = DeployConfig {
            target: DeployTarget::Netlify,
            output_dir: PathBuf::from("nonexistent"),
            ..Default::default()
        };
        let warnings = pre_deploy_checks(&config);
        assert!(warnings.iter().any(|w| w.contains("does not exist")));
        assert!(warnings.iter().any(|w| w.contains("netlify-cli")));
    }

    #[test]
    fn test_deploy_environment() {
        assert_eq!(DeployEnvironment::Production.as_str(), "production");
        assert_eq!(DeployEnvironment::Staging.as_str(), "staging");
        assert_eq!(DeployEnvironment::Preview.as_str(), "preview");
    }
}
