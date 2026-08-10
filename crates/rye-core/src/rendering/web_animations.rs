//! Goal 118: Web Animations API bridge.
//!
//! `use_web_animation()` hook that wraps the Web Animations API for
//! hardware-accelerated CSS animations. On native, maps to wgpu transforms.

/// Keyframe for Web Animations API.
#[derive(Debug, Clone)]
pub struct Keyframe {
    /// CSS properties at this keyframe.
    pub properties: Vec<(String, String)>,
    /// Offset (0.0 to 1.0). None means auto-distribute.
    pub offset: Option<f32>,
    /// Easing function for this keyframe.
    pub easing: Option<String>,
}

impl Keyframe {
    /// Create a keyframe at the given offset.
    pub fn at(offset: f32) -> Self {
        Self {
            properties: Vec::new(),
            offset: Some(offset),
            easing: None,
        }
    }

    /// Add a CSS property.
    pub fn prop(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.push((name.into(), value.into()));
        self
    }

    /// Set easing.
    pub fn ease(mut self, easing: impl Into<String>) -> Self {
        self.easing = Some(easing.into());
        self
    }
}

/// Animation options.
#[derive(Debug, Clone)]
pub struct AnimationOptions {
    /// Duration in milliseconds (or "auto").
    pub duration: AnimationDuration,
    /// Number of iterations (or infinite).
    pub iterations: IterationCount,
    /// Direction.
    pub direction: AnimationDirection,
    /// Easing function.
    pub easing: String,
    /// Fill mode.
    pub fill: FillMode,
    /// Delay in milliseconds.
    pub delay: i32,
    /// End delay in milliseconds.
    pub end_delay: i32,
}

impl Default for AnimationOptions {
    fn default() -> Self {
        Self {
            duration: AnimationDuration::Ms(300),
            iterations: IterationCount::Count(1),
            direction: AnimationDirection::Normal,
            easing: "ease".to_string(),
            fill: FillMode::None,
            delay: 0,
            end_delay: 0,
        }
    }
}

/// Animation duration.
#[derive(Debug, Clone)]
pub enum AnimationDuration {
    /// Duration in milliseconds.
    Ms(u32),
    /// Auto duration.
    Auto,
}

/// Iteration count.
#[derive(Debug, Clone)]
pub enum IterationCount {
    /// Fixed number of iterations.
    Count(u32),
    /// Infinite iterations.
    Infinite,
}

/// Animation direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationDirection {
    /// Normal playback.
    Normal,
    /// Reverse playback.
    Reverse,
    /// Alternate normal/reverse.
    Alternate,
    /// Alternate reverse/normal.
    AlternateReverse,
}

impl AnimationDirection {
    /// Convert to CSS string.
    pub fn as_str(&self) -> &'static str {
        match self {
            AnimationDirection::Normal => "normal",
            AnimationDirection::Reverse => "reverse",
            AnimationDirection::Alternate => "alternate",
            AnimationDirection::AlternateReverse => "alternate-reverse",
        }
    }
}

/// Fill mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FillMode {
    /// No fill.
    None,
    /// Fill forwards (retain end state).
    Forwards,
    /// Fill backwards (apply start state during delay).
    Backwards,
    /// Fill both.
    Both,
}

impl FillMode {
    /// Convert to CSS string.
    pub fn as_str(&self) -> &'static str {
        match self {
            FillMode::None => "none",
            FillMode::Forwards => "forwards",
            FillMode::Backwards => "backwards",
            FillMode::Both => "both",
        }
    }
}

/// Generate the JS for Web Animations API.
pub fn web_animation_script() -> &'static str {
    r#"<script>
(function() {
  window.__rye_animate = function(elementId, keyframes, options) {
    var el = document.getElementById(elementId);
    if (!el) return null;

    var kf = keyframes.map(function(k) {
      var frame = {};
      k.properties.forEach(function(p) { frame[p[0]] = p[1]; });
      if (k.offset !== null) frame.offset = k.offset;
      if (k.easing) frame.easing = k.easing;
      return frame;
    });

    var opts = {
      duration: options.duration === 'auto' ? 'auto' : options.duration,
      iterations: options.iterations === Infinity ? Infinity : options.iterations,
      direction: options.direction,
      easing: options.easing || 'ease',
      fill: options.fill || 'none',
      delay: options.delay || 0,
      endDelay: options.endDelay || 0
    };

    return el.animate(kf, opts);
  };
})();
</script>"#
}

/// Generate a fade-in animation.
pub fn fade_in(duration_ms: u32) -> (Vec<Keyframe>, AnimationOptions) {
    let keyframes = vec![
        Keyframe::at(0.0).prop("opacity", "0"),
        Keyframe::at(1.0).prop("opacity", "1"),
    ];
    let options = AnimationOptions {
        duration: AnimationDuration::Ms(duration_ms),
        fill: FillMode::Forwards,
        ..Default::default()
    };
    (keyframes, options)
}

/// Generate a slide-in animation.
pub fn slide_in(
    direction: &str,
    distance: f32,
    duration_ms: u32,
) -> (Vec<Keyframe>, AnimationOptions) {
    let transform_start = match direction {
        "left" => format!("translateX(-{}px)", distance),
        "right" => format!("translateX({}px)", distance),
        "up" => format!("translateY({}px)", distance),
        "down" => format!("translateY(-{}px)", distance),
        _ => format!("translateX(-{}px)", distance),
    };

    let keyframes = vec![
        Keyframe::at(0.0)
            .prop("transform", transform_start)
            .prop("opacity", "0"),
        Keyframe::at(1.0)
            .prop("transform", "translateX(0)")
            .prop("opacity", "1"),
    ];
    let options = AnimationOptions {
        duration: AnimationDuration::Ms(duration_ms),
        fill: FillMode::Forwards,
        easing: "ease-out".to_string(),
        ..Default::default()
    };
    (keyframes, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyframe() {
        let kf = Keyframe::at(0.5)
            .prop("opacity", "0.5")
            .prop("transform", "scale(1.1)")
            .ease("ease-in-out");
        assert_eq!(kf.offset, Some(0.5));
        assert_eq!(kf.properties.len(), 2);
        assert_eq!(kf.easing, Some("ease-in-out".to_string()));
    }

    #[test]
    fn test_animation_options_default() {
        let opts = AnimationOptions::default();
        assert!(matches!(opts.duration, AnimationDuration::Ms(300)));
        assert!(matches!(opts.iterations, IterationCount::Count(1)));
        assert_eq!(opts.direction, AnimationDirection::Normal);
        assert_eq!(opts.fill, FillMode::None);
    }

    #[test]
    fn test_animation_direction() {
        assert_eq!(AnimationDirection::Normal.as_str(), "normal");
        assert_eq!(AnimationDirection::Reverse.as_str(), "reverse");
        assert_eq!(AnimationDirection::Alternate.as_str(), "alternate");
        assert_eq!(
            AnimationDirection::AlternateReverse.as_str(),
            "alternate-reverse"
        );
    }

    #[test]
    fn test_fill_mode() {
        assert_eq!(FillMode::None.as_str(), "none");
        assert_eq!(FillMode::Forwards.as_str(), "forwards");
        assert_eq!(FillMode::Both.as_str(), "both");
    }

    #[test]
    fn test_fade_in() {
        let (kf, opts) = fade_in(500);
        assert_eq!(kf.len(), 2);
        assert!(matches!(opts.duration, AnimationDuration::Ms(500)));
        assert_eq!(opts.fill, FillMode::Forwards);
    }

    #[test]
    fn test_slide_in() {
        let (kf, opts) = slide_in("left", 100.0, 300);
        assert_eq!(kf.len(), 2);
        assert!(kf[0]
            .properties
            .iter()
            .any(|(k, v)| k == "transform" && v.contains("translateX(-100px)")));
        assert_eq!(opts.easing, "ease-out");
    }

    #[test]
    fn test_web_animation_script() {
        let script = web_animation_script();
        assert!(script.contains("el.animate"));
        assert!(script.contains("__rye_animate"));
    }
}
