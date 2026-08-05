//! AspectRatio — maintain width/height ratio.

use rye_core::Element;
use rye_core::template::Template;

#[derive(Debug, Clone)]
pub struct AspectRatioProps {
    pub ratio: f64, // width / height
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for AspectRatioProps {
    fn default() -> Self { Self { ratio: 1.0, class: None, style: None } }
}

impl AspectRatioProps {
    pub fn ratio(mut self, r: f64) -> Self { self.ratio = r; self }
    pub fn square() -> Self { Self { ratio: 1.0, class: None, style: None } }
    pub fn video() -> Self { Self { ratio: 16.0 / 9.0, class: None, style: None } }
    pub fn wide() -> Self { Self { ratio: 2.0, class: None, style: None } }
    pub fn portrait() -> Self { Self { ratio: 3.0 / 4.0, class: None, style: None } }
}

pub struct AspectRatio;

impl AspectRatio {
    pub fn render(props: AspectRatioProps) -> Element {
        let pct = (100.0 / props.ratio).min(999.0);
        let style = format!(
            "position:relative;width:100%;padding-top:{}%;{}",
            pct, props.style.as_deref().unwrap_or(""),
        );

        let inner = Template::new_element("div",
            vec![("style".to_string(), "position:absolute;top:0;left:0;right:0;bottom:0;".to_string()),
                 ("class".to_string(), "rye-aspect-ratio-inner".to_string())],
            Vec::new(), Vec::new());

        Element::Template(Template::new_element("div",
            vec![("style".to_string(), style),
                 ("class".to_string(), format!("rye-aspect-ratio {}", props.class.as_deref().unwrap_or("")))],
            Vec::new(), vec![inner]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aspect_ratio_default() {
        let p = AspectRatioProps::default();
        assert_eq!(p.ratio, 1.0);
    }

    #[test]
    fn test_aspect_ratio_presets() {
        assert_eq!(AspectRatioProps::square().ratio, 1.0);
        assert!((AspectRatioProps::video().ratio - 16.0/9.0).abs() < 0.001);
        assert_eq!(AspectRatioProps::wide().ratio, 2.0);
    }

    #[test]
    fn test_aspect_ratio_builder() {
        let p = AspectRatioProps::default().ratio(4.0 / 3.0);
        assert!((p.ratio - 4.0/3.0).abs() < 0.001);
    }

    #[test]
    fn test_aspect_ratio_render() {
        let el = AspectRatio::render(AspectRatioProps::video());
        assert!(matches!(el, Element::Template(_)));
    }
}
