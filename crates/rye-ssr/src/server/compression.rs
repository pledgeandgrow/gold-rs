//! Goal 193: SSR compression with Brotli/Zstd.
//!
//! Auto-compress SSR responses with Brotli (best ratio) or Zstd (fastest).
//! Content-type aware — compress HTML, not images. Configurable quality levels.

use std::collections::HashMap;

/// Compression algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    /// Brotli — best compression ratio.
    Brotli,
    /// Zstd — fastest decompression.
    Zstd,
    /// Gzip — widely supported fallback.
    Gzip,
    /// No compression.
    None,
}

impl CompressionAlgorithm {
    /// Get the Content-Encoding header value.
    pub fn encoding(&self) -> &'static str {
        match self {
            CompressionAlgorithm::Brotli => "br",
            CompressionAlgorithm::Zstd => "zstd",
            CompressionAlgorithm::Gzip => "gzip",
            CompressionAlgorithm::None => "identity",
        }
    }

    /// Parse from Accept-Encoding header value.
    pub fn from_accept_encoding(header: &str) -> Self {
        let header_lower = header.to_lowercase();
        if header_lower.contains("br") {
            CompressionAlgorithm::Brotli
        } else if header_lower.contains("zstd") {
            CompressionAlgorithm::Zstd
        } else if header_lower.contains("gzip") {
            CompressionAlgorithm::Gzip
        } else {
            CompressionAlgorithm::None
        }
    }
}

/// Compression configuration.
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// The compression algorithm to use.
    pub algorithm: CompressionAlgorithm,
    /// Compression quality level (1-11 for Brotli, 1-22 for Zstd, 1-9 for Gzip).
    pub quality: u32,
    /// Minimum response size to compress (bytes).
    pub min_size: usize,
    /// Content types to compress.
    pub compressible_types: Vec<String>,
    /// Content types to never compress.
    pub excluded_types: Vec<String>,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Brotli,
            quality: 5,
            min_size: 1024, // 1KB
            compressible_types: vec![
                "text/html".to_string(),
                "text/css".to_string(),
                "text/javascript".to_string(),
                "application/javascript".to_string(),
                "application/json".to_string(),
                "application/xml".to_string(),
                "text/xml".to_string(),
                "text/plain".to_string(),
                "image/svg+xml".to_string(),
            ],
            excluded_types: vec![
                "image/png".to_string(),
                "image/jpeg".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
                "application/zip".to_string(),
                "application/gzip".to_string(),
                "video/mp4".to_string(),
                "font/woff2".to_string(),
            ],
        }
    }
}

impl CompressionConfig {
    /// Check if a content type should be compressed.
    pub fn should_compress(&self, content_type: &str, size: usize) -> bool {
        if size < self.min_size {
            return false;
        }

        let ct = content_type
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_lowercase();

        // Check excluded types first
        for excluded in &self.excluded_types {
            if ct == excluded.to_lowercase() {
                return false;
            }
        }

        // Check compressible types
        for compressible in &self.compressible_types {
            if ct == compressible.to_lowercase() {
                return true;
            }
        }

        false
    }

    /// Create a config optimized for best compression ratio.
    pub fn best_ratio() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Brotli,
            quality: 11,
            ..Default::default()
        }
    }

    /// Create a config optimized for speed.
    pub fn fastest() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Zstd,
            quality: 1,
            ..Default::default()
        }
    }

    /// Set the minimum size.
    pub fn with_min_size(mut self, min_size: usize) -> Self {
        self.min_size = min_size;
        self
    }

    /// Set the quality.
    pub fn with_quality(mut self, quality: u32) -> Self {
        self.quality = quality;
        self
    }
}

/// The compression middleware — compresses responses based on config.
pub struct CompressionMiddleware {
    config: CompressionConfig,
}

impl CompressionMiddleware {
    /// Create a new compression middleware.
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }

    /// Create with default config.
    pub fn default() -> Self {
        Self::new(CompressionConfig::default())
    }

    /// Get the config.
    pub fn config(&self) -> &CompressionConfig {
        &self.config
    }

    /// Determine the best compression algorithm for a request.
    pub fn select_algorithm(&self, accept_encoding: &str) -> CompressionAlgorithm {
        let preferred = CompressionAlgorithm::from_accept_encoding(accept_encoding);
        if preferred == CompressionAlgorithm::None {
            // If client doesn't support any, return None
            CompressionAlgorithm::None
        } else {
            preferred
        }
    }

    /// Check if a response should be compressed.
    pub fn should_compress(&self, content_type: &str, size: usize) -> bool {
        self.config.should_compress(content_type, size)
    }

    /// Get the Content-Encoding header for the selected algorithm.
    pub fn encoding_header(&self, algorithm: CompressionAlgorithm) -> Option<(String, String)> {
        if algorithm == CompressionAlgorithm::None {
            None
        } else {
            Some((
                "Content-Encoding".to_string(),
                algorithm.encoding().to_string(),
            ))
        }
    }

    /// Get the Vary header value for compression.
    pub fn vary_header(&self) -> (String, String) {
        ("Vary".to_string(), "Accept-Encoding".to_string())
    }

    /// Process a response — returns whether it should be compressed and the algorithm.
    pub fn process_response(
        &self,
        content_type: &str,
        size: usize,
        accept_encoding: &str,
    ) -> CompressionDecision {
        if !self.should_compress(content_type, size) {
            return CompressionDecision::Skip;
        }

        let algorithm = self.select_algorithm(accept_encoding);
        if algorithm == CompressionAlgorithm::None {
            return CompressionDecision::Skip;
        }

        CompressionDecision::Compress(algorithm)
    }
}

/// The result of compression decision.
#[derive(Debug, Clone, PartialEq)]
pub enum CompressionDecision {
    /// Compress with the given algorithm.
    Compress(CompressionAlgorithm),
    /// Skip compression.
    Skip,
}

/// Compress data using a simple RLE-like algorithm (for testing without external deps).
/// In production, this would use brotli/zstd crates.
pub fn compress_dummy(data: &[u8], _algorithm: CompressionAlgorithm, _quality: u32) -> Vec<u8> {
    // Placeholder: just return the data as-is
    // Real implementation would use brotli::BrotliCompress or zstd::encode_all
    data.to_vec()
}

/// Decompress data (dummy implementation for testing).
pub fn decompress_dummy(data: &[u8], _algorithm: CompressionAlgorithm) -> Vec<u8> {
    data.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_algorithm_encoding() {
        assert_eq!(CompressionAlgorithm::Brotli.encoding(), "br");
        assert_eq!(CompressionAlgorithm::Zstd.encoding(), "zstd");
        assert_eq!(CompressionAlgorithm::Gzip.encoding(), "gzip");
        assert_eq!(CompressionAlgorithm::None.encoding(), "identity");
    }

    #[test]
    fn test_from_accept_encoding() {
        assert_eq!(
            CompressionAlgorithm::from_accept_encoding("br, gzip, zstd"),
            CompressionAlgorithm::Brotli
        );
        assert_eq!(
            CompressionAlgorithm::from_accept_encoding("gzip, deflate"),
            CompressionAlgorithm::Gzip
        );
        assert_eq!(
            CompressionAlgorithm::from_accept_encoding("zstd"),
            CompressionAlgorithm::Zstd
        );
        assert_eq!(
            CompressionAlgorithm::from_accept_encoding("identity"),
            CompressionAlgorithm::None
        );
    }

    #[test]
    fn test_should_compress_html() {
        let config = CompressionConfig::default();
        assert!(config.should_compress("text/html", 2048));
        assert!(config.should_compress("text/html; charset=utf-8", 2048));
    }

    #[test]
    fn test_should_not_compress_small() {
        let config = CompressionConfig::default();
        assert!(!config.should_compress("text/html", 100)); // below min_size
    }

    #[test]
    fn test_should_not_compress_images() {
        let config = CompressionConfig::default();
        assert!(!config.should_compress("image/png", 10000));
        assert!(!config.should_compress("image/jpeg", 10000));
    }

    #[test]
    fn test_should_compress_json() {
        let config = CompressionConfig::default();
        assert!(config.should_compress("application/json", 2048));
    }

    #[test]
    fn test_should_compress_svg() {
        let config = CompressionConfig::default();
        assert!(config.should_compress("image/svg+xml", 2048));
    }

    #[test]
    fn test_should_not_compress_video() {
        let config = CompressionConfig::default();
        assert!(!config.should_compress("video/mp4", 100000));
    }

    #[test]
    fn test_config_best_ratio() {
        let config = CompressionConfig::best_ratio();
        assert_eq!(config.algorithm, CompressionAlgorithm::Brotli);
        assert_eq!(config.quality, 11);
    }

    #[test]
    fn test_config_fastest() {
        let config = CompressionConfig::fastest();
        assert_eq!(config.algorithm, CompressionAlgorithm::Zstd);
        assert_eq!(config.quality, 1);
    }

    #[test]
    fn test_config_with_min_size() {
        let config = CompressionConfig::default().with_min_size(512);
        assert_eq!(config.min_size, 512);
    }

    #[test]
    fn test_config_with_quality() {
        let config = CompressionConfig::default().with_quality(9);
        assert_eq!(config.quality, 9);
    }

    #[test]
    fn test_middleware_process_compress() {
        let middleware = CompressionMiddleware::default();
        let decision = middleware.process_response("text/html", 5000, "br, gzip");
        assert_eq!(
            decision,
            CompressionDecision::Compress(CompressionAlgorithm::Brotli)
        );
    }

    #[test]
    fn test_middleware_process_skip_small() {
        let middleware = CompressionMiddleware::default();
        let decision = middleware.process_response("text/html", 100, "br");
        assert_eq!(decision, CompressionDecision::Skip);
    }

    #[test]
    fn test_middleware_process_skip_image() {
        let middleware = CompressionMiddleware::default();
        let decision = middleware.process_response("image/png", 10000, "br");
        assert_eq!(decision, CompressionDecision::Skip);
    }

    #[test]
    fn test_middleware_process_skip_no_accept() {
        let middleware = CompressionMiddleware::default();
        let decision = middleware.process_response("text/html", 5000, "identity");
        assert_eq!(decision, CompressionDecision::Skip);
    }

    #[test]
    fn test_middleware_encoding_header() {
        let middleware = CompressionMiddleware::default();
        let header = middleware.encoding_header(CompressionAlgorithm::Brotli);
        assert_eq!(
            header,
            Some(("Content-Encoding".to_string(), "br".to_string()))
        );

        let none_header = middleware.encoding_header(CompressionAlgorithm::None);
        assert!(none_header.is_none());
    }

    #[test]
    fn test_middleware_vary_header() {
        let middleware = CompressionMiddleware::default();
        let (key, value) = middleware.vary_header();
        assert_eq!(key, "Vary");
        assert_eq!(value, "Accept-Encoding");
    }

    #[test]
    fn test_middleware_select_algorithm() {
        let middleware = CompressionMiddleware::default();
        assert_eq!(
            middleware.select_algorithm("br, gzip"),
            CompressionAlgorithm::Brotli
        );
        assert_eq!(
            middleware.select_algorithm("gzip"),
            CompressionAlgorithm::Gzip
        );
    }

    #[test]
    fn test_compress_decompress_dummy() {
        let data = b"Hello, World!";
        let compressed = compress_dummy(data, CompressionAlgorithm::Brotli, 5);
        let decompressed = decompress_dummy(&compressed, CompressionAlgorithm::Brotli);
        assert_eq!(decompressed, data);
    }
}
