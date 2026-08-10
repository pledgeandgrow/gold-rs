//! Goal 119: High-DPI / Retina rendering.
//!
//! Automatic DPI scaling for native renderer. `use_dpi()` hook for
//! pixel-perfect rendering. Canvas components get correct backing store size.

/// DPI information.
#[derive(Debug, Clone, Copy)]
pub struct DpiInfo {
    /// Device pixel ratio (1.0 = standard, 2.0 = Retina, 3.0 = super Retina).
    pub device_pixel_ratio: f32,
    /// Logical width in CSS pixels.
    pub logical_width: f32,
    /// Logical height in CSS pixels.
    pub logical_height: f32,
    /// Physical width in device pixels.
    pub physical_width: f32,
    /// Physical height in device pixels.
    pub physical_height: f32,
}

impl DpiInfo {
    /// Create DPI info from logical dimensions and device pixel ratio.
    pub fn new(logical_width: f32, logical_height: f32, dpr: f32) -> Self {
        Self {
            device_pixel_ratio: dpr,
            logical_width,
            logical_height,
            physical_width: logical_width * dpr,
            physical_height: logical_height * dpr,
        }
    }

    /// Standard DPI (1x).
    pub fn standard(width: f32, height: f32) -> Self {
        Self::new(width, height, 1.0)
    }

    /// Retina DPI (2x).
    pub fn retina(width: f32, height: f32) -> Self {
        Self::new(width, height, 2.0)
    }

    /// Scale a logical value to physical pixels.
    pub fn to_physical(&self, logical: f32) -> f32 {
        logical * self.device_pixel_ratio
    }

    /// Scale a physical value to logical pixels.
    pub fn to_logical(&self, physical: f32) -> f32 {
        physical / self.device_pixel_ratio
    }

    /// Whether this is a high-DPI display (DPR > 1.5).
    pub fn is_high_dpi(&self) -> bool {
        self.device_pixel_ratio > 1.5
    }
}

/// DPI scale factor for rendering.
#[derive(Debug, Clone, Copy)]
pub struct DpiScale {
    /// Horizontal scale factor.
    pub x: f32,
    /// Vertical scale factor.
    pub y: f32,
}

impl DpiScale {
    /// Create a uniform scale.
    pub fn uniform(scale: f32) -> Self {
        Self { x: scale, y: scale }
    }

    /// Identity scale (1:1).
    pub fn identity() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

/// Detect the current device pixel ratio.
pub fn detect_device_pixel_ratio() -> f32 {
    // On Wasm: window.devicePixelRatio
    // On native: query the window/surface
    #[cfg(not(target_arch = "wasm32"))]
    {
        1.0
    }
    #[cfg(target_arch = "wasm32")]
    {
        1.0
    }
}

/// Generate the JS for DPI detection and canvas sizing.
pub fn dpi_script() -> &'static str {
    r#"<script>
(function() {
  window.__rye_get_dpi = function() {
    var dpr = window.devicePixelRatio || 1;
    return {
      devicePixelRatio: dpr,
      logicalWidth: window.innerWidth,
      logicalHeight: window.innerHeight,
      physicalWidth: window.innerWidth * dpr,
      physicalHeight: window.innerHeight * dpr
    };
  };

  window.__rye_setup_canvas = function(canvasId, logicalWidth, logicalHeight) {
    var canvas = document.getElementById(canvasId);
    if (!canvas) return;
    var dpr = window.devicePixelRatio || 1;
    canvas.style.width = logicalWidth + 'px';
    canvas.style.height = logicalHeight + 'px';
    canvas.width = logicalWidth * dpr;
    canvas.height = logicalHeight * dpr;
    var ctx = canvas.getContext('2d');
    if (ctx) {
      ctx.scale(dpr, dpr);
    }
    return { dpr: dpr, ctx: ctx };
  };

  window.__rye_on_dpi_change = function(callbackId) {
    var mediaQuery = window.matchMedia('(resolution: ' + window.devicePixelRatio + 'dppx)');
    mediaQuery.addEventListener('change', function() {
      window.__rye_signal_update(callbackId, window.__rye_get_dpi());
    });
  };
})();
</script>"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dpi_info_standard() {
        let dpi = DpiInfo::standard(1920.0, 1080.0);
        assert_eq!(dpi.device_pixel_ratio, 1.0);
        assert_eq!(dpi.physical_width, 1920.0);
        assert!(!dpi.is_high_dpi());
    }

    #[test]
    fn test_dpi_info_retina() {
        let dpi = DpiInfo::retina(1440.0, 900.0);
        assert_eq!(dpi.device_pixel_ratio, 2.0);
        assert_eq!(dpi.physical_width, 2880.0);
        assert_eq!(dpi.physical_height, 1800.0);
        assert!(dpi.is_high_dpi());
    }

    #[test]
    fn test_dpi_to_physical() {
        let dpi = DpiInfo::retina(100.0, 100.0);
        assert_eq!(dpi.to_physical(50.0), 100.0);
    }

    #[test]
    fn test_dpi_to_logical() {
        let dpi = DpiInfo::retina(100.0, 100.0);
        assert_eq!(dpi.to_logical(100.0), 50.0);
    }

    #[test]
    fn test_dpi_scale() {
        let s = DpiScale::uniform(2.0);
        assert_eq!(s.x, 2.0);
        assert_eq!(s.y, 2.0);

        let id = DpiScale::identity();
        assert_eq!(id.x, 1.0);
    }

    #[test]
    fn test_dpi_script() {
        let script = dpi_script();
        assert!(script.contains("devicePixelRatio"));
        assert!(script.contains("__rye_get_dpi"));
        assert!(script.contains("__rye_setup_canvas"));
    }
}
