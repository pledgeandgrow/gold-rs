//! Carousel — image/content slider.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct CarouselSlide {
    pub image: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

impl CarouselSlide {
    pub fn new() -> Self {
        Self {
            image: None,
            title: None,
            description: None,
        }
    }
    pub fn image(mut self, i: impl Into<String>) -> Self {
        self.image = Some(i.into());
        self
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
}

impl Default for CarouselSlide {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CarouselProps {
    pub slides: Vec<CarouselSlide>,
    pub current: usize,
    pub show_arrows: bool,
    pub show_dots: bool,
    pub height: String,
    pub autoplay: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for CarouselProps {
    fn default() -> Self {
        Self {
            slides: Vec::new(),
            current: 0,
            show_arrows: true,
            show_dots: true,
            height: "300px".to_string(),
            autoplay: false,
            class: None,
            style: None,
        }
    }
}

impl CarouselProps {
    pub fn slides(mut self, s: Vec<CarouselSlide>) -> Self {
        self.slides = s;
        self
    }
    pub fn current(mut self, c: usize) -> Self {
        self.current = c;
        self
    }
    pub fn height(mut self, h: impl Into<String>) -> Self {
        self.height = h.into();
        self
    }
    pub fn autoplay(mut self, a: bool) -> Self {
        self.autoplay = a;
        self
    }
}

pub struct Carousel;

impl Carousel {
    pub fn render(props: CarouselProps) -> Element {
        if props.slides.is_empty() {
            return Element::None;
        }

        let current = props.current.min(props.slides.len() - 1);
        let slide = &props.slides[current];

        let container_style = format!(
            "position:relative;width:100%;height:{};border-radius:var(--rye-radius-lg);overflow:hidden;background:{};{}",
            props.height, vars::TEXT, props.style.as_deref().unwrap_or(""),
        );

        let mut children = Vec::new();

        // Slide content
        let mut slide_children = Vec::new();

        if let Some(img) = &slide.image {
            slide_children.push(Template::new_element(
                "img",
                vec![
                    ("src".to_string(), img.clone()),
                    (
                        "style".to_string(),
                        "width:100%;height:100%;object-fit:cover;".to_string(),
                    ),
                ],
                Vec::new(),
                Vec::new(),
            ));
        }

        if slide.title.is_some() || slide.description.is_some() {
            let overlay_style = "position:absolute;bottom:0;left:0;right:0;padding:20px;background:linear-gradient(transparent,rgba(0,0,0,0.7));color:var(--rye-bg);";

            let mut overlay_children = Vec::new();
            if let Some(title) = &slide.title {
                overlay_children.push(Template::new_element(
                    "div",
                    vec![(
                        "style".to_string(),
                        "font-size:20px;font-weight:600;margin-bottom:4px;".to_string(),
                    )],
                    Vec::new(),
                    vec![Template::text(title)],
                ));
            }
            if let Some(desc) = &slide.description {
                overlay_children.push(Template::new_element(
                    "div",
                    vec![(
                        "style".to_string(),
                        "font-size:14px;opacity:0.9;".to_string(),
                    )],
                    Vec::new(),
                    vec![Template::text(desc)],
                ));
            }

            slide_children.push(Template::new_element(
                "div",
                vec![("style".to_string(), overlay_style.to_string())],
                Vec::new(),
                overlay_children,
            ));
        }

        children.push(Template::new_element(
            "div",
            vec![
                (
                    "style".to_string(),
                    "width:100%;height:100%;position:relative;".to_string(),
                ),
                ("class".to_string(), "rye-carousel-slide".to_string()),
            ],
            Vec::new(),
            slide_children,
        ));

        // Arrows
        if props.show_arrows && props.slides.len() > 1 {
            let arrow_style = "position:absolute;top:50%;transform:translateY(-50%);width:40px;height:40px;border-radius:50%;background:rgba(255,255,255,0.8);border:none;cursor:pointer;font-size:var(--rye-font-size-xl);color:var(--rye-text);display:flex;align-items:center;justify-content:center;";
            children.push(Template::new_element(
                "button",
                vec![
                    ("style".to_string(), format!("{}left:12px;", arrow_style)),
                    ("class".to_string(), "rye-carousel-prev".to_string()),
                ],
                Vec::new(),
                vec![Template::text("‹")],
            ));
            children.push(Template::new_element(
                "button",
                vec![
                    ("style".to_string(), format!("{}right:12px;", arrow_style)),
                    ("class".to_string(), "rye-carousel-next".to_string()),
                ],
                Vec::new(),
                vec![Template::text("›")],
            ));
        }

        // Dots
        if props.show_dots && props.slides.len() > 1 {
            let dots: Vec<Template> = (0..props.slides.len()).map(|i| {
                let dot_color = if i == current { vars::BG } else { "rgba(255,255,255,0.4)" };
                let dot_w = if i == current { "24px" } else { "8px" };
                Template::new_element("button",
                    vec![("style".to_string(), format!("width:{};height:8px;border-radius:4px;background:{};border:none;cursor:pointer;transition:all 0.2s;", dot_w, dot_color)),
                         ("class".to_string(), "rye-carousel-dot".to_string())],
                    Vec::new(), Vec::new())
            }).collect();

            children.push(Template::new_element("div",
                vec![("style".to_string(), "position:absolute;bottom:12px;left:50%;transform:translateX(-50%);display:flex;gap:6px;".to_string()),
                     ("class".to_string(), "rye-carousel-dots".to_string())],
                Vec::new(), dots));
        }

        Element::Template(Template::new_element(
            "div",
            vec![
                ("style".to_string(), container_style),
                (
                    "class".to_string(),
                    format!("rye-carousel {}", props.class.as_deref().unwrap_or("")),
                ),
            ],
            Vec::new(),
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_carousel_slide_builder() {
        let s = CarouselSlide::new()
            .image("/photo.jpg")
            .title("Sunset")
            .description("Beautiful evening");
        assert_eq!(s.image.as_deref(), Some("/photo.jpg"));
        assert_eq!(s.title.as_deref(), Some("Sunset"));
    }

    #[test]
    fn test_carousel_props_builder() {
        let p = CarouselProps::default()
            .slides(vec![
                CarouselSlide::new().title("A"),
                CarouselSlide::new().title("B"),
            ])
            .current(1)
            .height("400px")
            .autoplay(true);
        assert_eq!(p.slides.len(), 2);
        assert_eq!(p.current, 1);
        assert!(p.autoplay);
    }

    #[test]
    fn test_carousel_render_empty() {
        let el = Carousel::render(CarouselProps::default());
        assert!(matches!(el, Element::None));
    }

    #[test]
    fn test_carousel_render() {
        let el = Carousel::render(
            CarouselProps::default()
                .slides(vec![
                    CarouselSlide::new().title("First").image("/1.jpg"),
                    CarouselSlide::new().title("Second").image("/2.jpg"),
                ])
                .current(0),
        );
        assert!(matches!(el, Element::Template(_)));
    }
}
