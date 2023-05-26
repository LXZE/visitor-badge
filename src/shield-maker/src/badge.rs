use std::fmt::Display;
use std::ops::Rem;
use ab_glyph::{Font, FontArc, Glyph, point, PxScale, ScaleFont};
use crate::{color, xml};
use crate::xml::Pusher;
use crate::plastic_style::Plastic;
use crate::flat_style::Flat;
use crate::flat_square_style::FlatSquare;

fn measure_line(font: FontArc, text: &str, scale: PxScale) -> (f32, f32) {
    let font = font.as_scaled(scale);

    let mut caret = point(0.0, font.ascent());
    let mut first_glyph: Option<Glyph> = None;
    let mut last_glyph: Option<Glyph> = None;
    for c in text.chars().filter(|c| !c.is_control()) {
        let mut glyph = font.scaled_glyph(c);
        if let Some(prev) = last_glyph.take() {
            caret.x += font.kern(prev.id, glyph.id);
        }
        glyph.position = caret;

        if first_glyph.is_none() {
            first_glyph = Some(glyph.clone());
        }
        last_glyph = Some(glyph.clone());
        caret.x += font.h_advance(glyph.id);
    }

    let height = font.ascent() - font.descent() + font.line_gap();
    let width = {
        let min_x = first_glyph.unwrap().position.x;
        let last_glyph = last_glyph.unwrap();
        let max_x = last_glyph.position.x + font.h_advance(last_glyph.id);
        (max_x - min_x).ceil()
    };

    (width, height)
}

const FONT_FAMILY: &str = "Verdana,Geneva,DejaVu Sans,sans-serif";
const FONT_SCALE_UP_FACTOR: f32 = 10.0;
const FONT_SCALE_DOWN_VALUE: &str = "scale(.1)";

const WIDTH_FONT_SCALE: f32 = 11.0;

/// Represents the desired style of a badge
#[derive(Copy, Clone)]
pub enum Style {
    /// Plastic generates a rounded, plastic-ish looking badge. This is the
    /// default style generated by shields.io, for instance.
    Plastic,

    /// Flat is just like Plastic, without gradients.
    Flat,

    /// FlatSquare contains no rounded corners nor gradients.
    FlatSquare,
}

/// Represents the desired font family of a badge
pub enum FontFamily {
    /// Uses a font family provided by this crate, comprised of Verdana, Geneva
    /// DejaVu Sans, and sans-serif.
    Default,

    /// Uses a provided string as the font family for rendering the badge.
    Custom(String),
}

impl FontFamily {
    fn string(&self) -> String {
        match self {
            FontFamily::Default => FONT_FAMILY.into(),
            FontFamily::Custom(val) => val.clone(),
        }
    }
}

/// Metadata represents all information required to build a badge.
pub struct Metadata<'a> {
    /// The desired badge style
    pub style: Style,

    /// The text to be shown on the badge's label (left side)
    pub label: &'a str,

    /// The message to be shown on the badge's message (right side)
    pub message: &'a str,

    /// A [FontArc](ab_glyph::FontArc) to be used for measuring the final size
    /// of a badge.
    pub font: FontArc,

    /// The [FontFamily](shield_maker::FontFamily) to be used when rendering this
    /// badge.
    pub font_family: FontFamily,

    /// The color for the badge's label background. When `None`, a default
    /// grayish tone is used. When provided, any CSS color may be used.
    pub label_color: Option<&'a str>,

    /// The color for the badge's message background. When `None`, a default
    /// greenish color is used. When provided, any CSS color may be used.
    pub color: Option<&'a str>,
}

pub(crate) struct GradientStop<'a> {
    pub(crate) offset: &'a str,
    pub(crate) stop_color: &'a str,
    pub(crate) stop_opacity: &'a str,
}

impl GradientStop<'_> {
    pub(crate) fn into_attributes(self, of: &mut xml::Node) {
        of.add_attrs(&[
            ("offset", self.offset),
            ("stop-color", self.stop_color),
            ("stop-opacity", self.stop_opacity),
        ]);
    }
}

fn round_up_to_odd(val: f32) -> f32 {
    if val.rem(2.0) as i32 == 0 {
        val + 1.0
    } else {
        val
    }.round()
}

fn preferred_width_of(text: &str, font: FontArc, scale: PxScale) -> f32 {
    let (w, _) = measure_line(font, text, scale);
    let val = round_up_to_odd(w);
    val * 1.0345
}

fn colors_for_background(color_str: &str) -> Option<(&str, &str)> {
    const BRIGHTNESS_THRESHOLD: f32 = 0.69;
    let parsed_color = match color::color_by_name(Some(color_str)) {
        Some(c) => c,
        None => return None,
    };

    if color::brightness(parsed_color) <= BRIGHTNESS_THRESHOLD {
        return Some(("#fff", "#010101"));
    }

    Some(("#333", "#ccc"))
}

pub(crate) trait Badger {
    fn vertical_margin(&self) -> f32;
    fn height(&self) -> f32;
    fn shadow(&self) -> bool;
    fn render(&self, parent: &Renderer) -> Vec<xml::Node>;
}

/// Renderer implements all mechanisms required to turn a provided badge
/// [Metadata](Metadata) into its SVG representation.
pub struct Renderer<'a> {
    horizontal_padding: f32,

    label_margin: f32,
    message_margin: f32,
    label_width: f32,
    message_width: f32,

    left_width: f32,
    right_width: f32,
    font_family: String,

    width: f32,
    label_color: css_color_parser::Color,
    color: css_color_parser::Color,
    label: &'a str,
    message: &'a str,
    accessible_text: String,

    style: Box<dyn Badger>,
}

impl Renderer<'_> {
    fn new<'a>(info: &'a Metadata<'a>) -> Renderer<'a> {
        let horizontal_padding = 5.0;

        let label_margin = 1.0;
        let scale = PxScale::from(WIDTH_FONT_SCALE);
        let label_width = preferred_width_of(info.label, info.font.clone(), scale);
        let left_width = label_width + 2.0 * horizontal_padding;

        let message_width = preferred_width_of(info.message, info.font.clone(), scale);
        let message_margin = left_width - 1.0;
        let right_width = message_width + 2.0 * horizontal_padding;
        let width = left_width + right_width;
        let label_color = color::color_by_name(info.label_color).unwrap_or_else(|| color::color_by_name(Some("#555")).unwrap());
        let color = color::color_by_name(info.color).unwrap_or_else(|| color::color_by_name(Some("#4c1")).unwrap());

        let accessible_text = format!("{}: {}", info.label, info.message);

        let styler: Box<dyn Badger> = match info.style {
            Style::Plastic => Box::new(Plastic {}),
            Style::Flat => Box::new(Flat {}),
            Style::FlatSquare => Box::new(FlatSquare {}),
        };

        Renderer {
            horizontal_padding,
            label_margin,
            message_margin,
            label_width,
            message_width,
            left_width,
            right_width,
            font_family: info.font_family.string(),
            width,
            label_color,
            color,
            label: info.label,
            message: info.message,
            accessible_text,
            style: styler,
        }
    }

    /// Render renders a given set of [Metadata] into its SVG representation.
    pub fn render(info: &Metadata) -> String {
        let mut render = Renderer::new(info);
        render.internal_render()
    }

    fn internal_render(&mut self) -> String {
        let title = xml::Node::with_name_and("title",
                                             |n| n.push_text(&self.accessible_text));

        let mut svg = xml::Node::with_attributes("svg", &[
            ("xmlns", "http://www.w3.org/2000/svg"),
            ("xmlns:xlink", "http://www.w3.org/1999/xlink"),
            ("width", &format!("{}", self.width)),
            ("height", &format!("{}", self.style.height())),
            ("role", "img"),
            ("aria-label", &self.accessible_text),
        ]);
        svg.push_node(title);
        svg.push_nodes(self.style.render(self));

        let mut doc = xml::Document::new();
        doc.push_node(svg);
        xml::Renderer::render(&doc)
    }

    fn make_text_element(&self, left_margin: f32, content: &str, color: &str, text_width: f32) -> Vec<xml::Node> {
        let (text_color, shadow_color) = colors_for_background(color).unwrap_or(("", ""));

        let x = FONT_SCALE_UP_FACTOR * (left_margin + 0.5 * text_width + self.horizontal_padding);
        let mut result = vec![];

        if self.style.shadow() {
            let shadow = xml::Node::with_name_and("text", |n| {
                n.add_attrs(&[
                    ("aria-hidden", "true"),
                    ("fill", shadow_color),
                    ("x", &format!("{}", x)),
                    ("y", &format!("{}", 150.0 + self.style.vertical_margin())),
                    ("fill-opacity", ".3"),
                    ("transform", FONT_SCALE_DOWN_VALUE),
                    ("textLength", &format!("{}", FONT_SCALE_UP_FACTOR * text_width)),
                ]);
                n.push_text(content);
            });
            result.push(shadow);
        }

        result.push(xml::Node::with_name_and("text", |n| {
            n.add_attrs(&[
                ("fill", text_color),
                ("x", &format!("{}", x)),
                ("y", &format!("{}", 140.0 + self.style.vertical_margin())),
                ("transform", FONT_SCALE_DOWN_VALUE),
                ("textLength", &format!("{}", FONT_SCALE_UP_FACTOR * text_width)),
            ]);
            n.push_text(content);
        }));

        result
    }

    fn make_label_element(&self) -> Vec<xml::Node> {
        self.make_text_element(self.label_margin, self.label, &color::color_to_string(self.label_color), self.label_width)
    }

    fn make_message_element(&self) -> Vec<xml::Node> {
        self.make_text_element(self.message_margin, self.message, &color::color_to_string(self.color), self.message_width)
    }

    pub(crate) fn make_clip_path_element(&self, radius: f32) -> xml::Node {
        xml::Node::with_name_and("clipPath", |n| {
            n.add_attr("id", "r");
            n.push_node_named("rect", |n| {
                n.add_attrs(&[
                    ("fill", "#fff"),
                    ("width", &format!("{}", self.width)),
                    ("height", &format!("{}", self.style.height())),
                    ("rx", &format!("{}", radius)),
                ]);
            });
        })
    }

    pub(crate) fn make_background_group_element<V: Display + ?Sized>(&self, with_gradient: bool, attributes: &[(&str, &V)]) -> xml::Node {
        xml::Node::with_name_and("g", |n| {
            n.add_attrs(attributes);
            let height = format!("{}", self.style.height());
            let left_width = format!("{}", self.left_width);

            // left rect
            n.push_node_named("rect", |r| {
                r.add_attrs(&[
                    ("width", &left_width),
                    ("height", &height),
                    ("fill", &color::color_to_string(self.label_color)),
                ]);
            });

            // right rect
            n.push_node_named("rect", |r| {
                r.add_attrs(&[
                    ("x", &left_width),
                    ("width", &format!("{}", self.right_width)),
                    ("height", &height),
                    ("fill", &color::color_to_string(self.color)),
                ]);
            });

            if with_gradient {
                n.push_node_named("rect", |r| {
                    r.add_attrs(&[
                        ("fill", "url(#s)"),
                        ("width", &format!("{}", self.width)),
                        ("height", &height),
                    ]);
                })
            }
        })
    }

    pub(crate) fn make_foreground_group_element(&self) -> xml::Node {
        xml::Node::with_name_and("g", |n| {
            n.push_nodes(self.make_label_element());
            n.push_nodes(self.make_message_element());
            n.add_attrs(&[
                ("fill", "#fff"),
                ("text-anchor", "middle"),
                ("font-family", &self.font_family),
                ("text-rendering", "geometricPrecision"),
                ("font-size", "110"),
            ]);
        })
    }
}
