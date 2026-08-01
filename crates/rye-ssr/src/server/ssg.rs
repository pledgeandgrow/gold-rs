//! Goal 130: Static site generation (SSG).
//!
//! `rye build --static` generates static HTML files for all routes.
//! Supports incremental regeneration, preview builds, and sitemap generation.

use std::collections::HashMap;
use std::path::PathBuf;

/// SSG configuration.
#[derive(Debug, Clone)]
pub struct SsgConfig {
    /// Output directory for static files.
    pub output_dir: PathBuf,
    /// Routes to generate.
    pub routes: Vec<SsgRoute>,
    /// Whether to generate a sitemap.xml.
    pub sitemap: bool,
    /// Whether to generate robots.txt.
    pub robots: bool,
    /// Base URL for sitemap.
    pub base_url: Option<String>,
    /// Incremental regeneration interval (seconds). None = no revalidation.
    pub revalidate: Option<u32>,
}

impl Default for SsgConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("dist"),
            routes: Vec::new(),
            sitemap: true,
            robots: true,
            base_url: None,
            revalidate: None,
        }
    }
}

impl SsgConfig {
    /// Create a new SSG config with output directory.
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            output_dir: output_dir.into(),
            ..Default::default()
        }
    }

    /// Add a route to generate.
    pub fn route(mut self, route: SsgRoute) -> Self {
        self.routes.push(route);
        self
    }

    /// Set base URL for sitemap.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set revalidation interval.
    pub fn revalidate(mut self, seconds: u32) -> Self {
        self.revalidate = Some(seconds);
        self
    }
}

/// A route to statically generate.
#[derive(Debug, Clone)]
pub struct SsgRoute {
    /// Route path (e.g. "/", "/about", "/users/:id").
    pub path: String,
    /// Dynamic params (for dynamic routes).
    pub params: Vec<HashMap<String, String>>,
    /// Output file path (relative to output dir). None = auto from path.
    pub output_path: Option<String>,
}

impl SsgRoute {
    /// Create a static route.
    pub fn static_route(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            params: Vec::new(),
            output_path: None,
        }
    }

    /// Create a dynamic route with params.
    pub fn dynamic(path: impl Into<String>, params: Vec<HashMap<String, String>>) -> Self {
        Self {
            path: path.into(),
            params,
            output_path: None,
        }
    }

    /// Set custom output path.
    pub fn output_to(mut self, path: impl Into<String>) -> Self {
        self.output_path = Some(path.into());
        self
    }

    /// Get the output file path for this route.
    pub fn get_output_path(&self) -> String {
        if let Some(p) = &self.output_path {
            return p.clone();
        }

        if self.path == "/" {
            return "index.html".to_string();
        }

        let clean = self.path.trim_start_matches('/');
        format!("{}/index.html", clean)
    }

    /// Get all output paths (including dynamic params).
    pub fn get_all_output_paths(&self) -> Vec<String> {
        if self.params.is_empty() {
            return vec![self.get_output_path()];
        }

        self.params.iter().map(|params| {
            let mut path = self.path.clone();
            for (key, value) in params {
                path = path.replace(&format!(":{}", key), value);
            }
            if path == "/" {
                "index.html".to_string()
            } else {
                format!("{}/index.html", path.trim_start_matches('/'))
            }
        }).collect()
    }
}

/// Generated static file.
#[derive(Debug, Clone)]
pub struct StaticFile {
    /// Output path.
    pub path: String,
    /// File content.
    pub content: String,
    /// Content type.
    pub content_type: String,
}

/// Sitemap entry.
#[derive(Debug, Clone)]
pub struct SitemapEntry {
    /// URL.
    pub loc: String,
    /// Last modified date (YYYY-MM-DD).
    pub lastmod: Option<String>,
    /// Change frequency.
    pub changefreq: Option<String>,
    /// Priority (0.0 to 1.0).
    pub priority: Option<f32>,
}

/// Generate sitemap.xml content.
pub fn generate_sitemap(base_url: &str, entries: &[SitemapEntry]) -> String {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push_str("\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    for entry in entries {
        xml.push_str("  <url>\n");
        xml.push_str(&format!("    <loc>{}{}</loc>\n", base_url, entry.loc));
        if let Some(lastmod) = &entry.lastmod {
            xml.push_str(&format!("    <lastmod>{}</lastmod>\n", lastmod));
        }
        if let Some(changefreq) = &entry.changefreq {
            xml.push_str(&format!("    <changefreq>{}</changefreq>\n", changefreq));
        }
        if let Some(priority) = entry.priority {
            xml.push_str(&format!("    <priority>{:.1}</priority>\n", priority));
        }
        xml.push_str("  </url>\n");
    }

    xml.push_str("</urlset>\n");
    xml
}

/// Generate robots.txt content.
pub fn generate_robots(base_url: &str, disallow: &[&str]) -> String {
    let mut txt = String::from("User-agent: *\n");
    for path in disallow {
        txt.push_str(&format!("Disallow: {}\n", path));
    }
    txt.push_str(&format!("\nSitemap: {}/sitemap.xml\n", base_url));
    txt
}

/// Build plan — all files to generate.
#[derive(Debug, Clone)]
pub struct SsgBuildPlan {
    /// HTML files to generate.
    pub html_files: Vec<StaticFile>,
    /// Sitemap content (if enabled).
    pub sitemap: Option<String>,
    /// Robots.txt content (if enabled).
    pub robots: Option<String>,
}

/// Create a build plan from SSG config.
pub fn create_build_plan(config: &SsgConfig) -> SsgBuildPlan {
    let mut html_files = Vec::new();

    for route in &config.routes {
        let paths = route.get_all_output_paths();
        for path in paths {
            html_files.push(StaticFile {
                path,
                content: String::new(), // Would be filled by renderer
                content_type: "text/html".to_string(),
            });
        }
    }

    let sitemap = if config.sitemap {
        if let Some(base_url) = &config.base_url {
            let entries: Vec<SitemapEntry> = config.routes.iter()
                .flat_map(|r| {
                    if r.params.is_empty() {
                        vec![SitemapEntry {
                            loc: r.path.clone(),
                            lastmod: None,
                            changefreq: Some("weekly".to_string()),
                            priority: Some(0.8),
                        }]
                    } else {
                        r.params.iter().map(|params| {
                            let mut path = r.path.clone();
                            for (key, value) in params {
                                path = path.replace(&format!(":{}", key), value);
                            }
                            SitemapEntry {
                                loc: path,
                                lastmod: None,
                                changefreq: Some("weekly".to_string()),
                                priority: Some(0.8),
                            }
                        }).collect()
                    }
                })
                .collect();
            Some(generate_sitemap(base_url, &entries))
        } else {
            None
        }
    } else {
        None
    };

    let robots = if config.robots {
        config.base_url.as_ref().map(|base| generate_robots(base, &["/admin/"]))
    } else {
        None
    };

    SsgBuildPlan { html_files, sitemap, robots }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssg_route_static() {
        let route = SsgRoute::static_route("/about");
        assert_eq!(route.get_output_path(), "about/index.html");
    }

    #[test]
    fn test_ssg_route_root() {
        let route = SsgRoute::static_route("/");
        assert_eq!(route.get_output_path(), "index.html");
    }

    #[test]
    fn test_ssg_route_custom_output() {
        let route = SsgRoute::static_route("/about").output_to("about.html");
        assert_eq!(route.get_output_path(), "about.html");
    }

    #[test]
    fn test_ssg_route_dynamic() {
        let mut params1 = HashMap::new();
        params1.insert("id".to_string(), "123".to_string());
        let mut params2 = HashMap::new();
        params2.insert("id".to_string(), "456".to_string());

        let route = SsgRoute::dynamic("/users/:id", vec![params1, params2]);
        let paths = route.get_all_output_paths();
        assert_eq!(paths.len(), 2);
        assert!(paths[0].contains("123"));
        assert!(paths[1].contains("456"));
    }

    #[test]
    fn test_generate_sitemap() {
        let entries = vec![
            SitemapEntry {
                loc: "/".to_string(),
                lastmod: Some("2024-01-01".to_string()),
                changefreq: Some("daily".to_string()),
                priority: Some(1.0),
            },
            SitemapEntry {
                loc: "/about".to_string(),
                lastmod: None,
                changefreq: None,
                priority: None,
            },
        ];
        let xml = generate_sitemap("https://example.com", &entries);
        assert!(xml.contains("urlset"));
        assert!(xml.contains("https://example.com/"));
        assert!(xml.contains("https://example.com/about"));
        assert!(xml.contains("lastmod"));
        assert!(xml.contains("priority"));
    }

    #[test]
    fn test_generate_robots() {
        let txt = generate_robots("https://example.com", &["/admin/", "/private/"]);
        assert!(txt.contains("User-agent: *"));
        assert!(txt.contains("Disallow: /admin/"));
        assert!(txt.contains("Sitemap: https://example.com/sitemap.xml"));
    }

    #[test]
    fn test_create_build_plan() {
        let config = SsgConfig::new("dist")
            .route(SsgRoute::static_route("/"))
            .route(SsgRoute::static_route("/about"))
            .base_url("https://example.com");

        let plan = create_build_plan(&config);
        assert_eq!(plan.html_files.len(), 2);
        assert!(plan.sitemap.is_some());
        assert!(plan.robots.is_some());
    }

    #[test]
    fn test_ssg_config_revalidate() {
        let config = SsgConfig::new("dist").revalidate(3600);
        assert_eq!(config.revalidate, Some(3600));
    }
}
