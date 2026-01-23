/// Very small layout primitives for Phase 1 prototype

use crate::Viewport;
use scraper::{Html, Selector};

#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxModel {
    pub margin: u32,
    pub border: u32,
    pub padding: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutBox {
    pub rect: Rect,
    pub box_model: BoxModel,
}

impl LayoutBox {
    pub fn content_width(&self) -> u32 {
        let total = self.box_model.margin + self.box_model.border + self.box_model.padding;
        self.rect.width.saturating_sub(total)
    }
}

/// A layout node couples a `LayoutBox` with rendered text and element type.
/// For Phase 1 we keep this simple: title (heading) and paragraph boxes only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementType {
    Title,
    Paragraph,
    Other,
}

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub lb: LayoutBox,
    pub text: String,
    pub elem_type: ElementType,
    pub scale: usize,
}

/// Compute a basic block layout for the provided HTML document and viewport.
/// - Stacks blocks vertically with simple margins/padding
/// - Title (h1 or <title>) rendered at scale=2, paragraphs at scale=1
pub fn layout_document(document: &Html, viewport: Viewport) -> Vec<LayoutNode> {
    let mut y = 8u32; // top padding
    let page_width = viewport.width;
    let mut nodes = Vec::new();

    // Title: prefer <h1> then <title>
    let h1_sel = Selector::parse("h1").unwrap();
    let title_text = if let Some(h1) = document.select(&h1_sel).next() {
        h1.text().collect::<String>()
    } else {
        let tsel = Selector::parse("title").unwrap();
        document
            .select(&tsel)
            .next()
            .map(|n| n.text().collect::<String>())
            .unwrap_or_default()
    };

    if !title_text.trim().is_empty() {
        let padding = 8u32;
        let box_h = (8 * 2) as u32 + padding * 2; // scaled text height + padding
        let lb = LayoutBox {
            rect: Rect {
                x: 8,
                y: y as i32,
                width: page_width.saturating_sub(16),
                height: box_h,
            },
            box_model: BoxModel {
                margin: 8,
                border: 0,
                padding,
            },
        };
        nodes.push(LayoutNode {
            lb,
            text: title_text.trim().to_string(),
            elem_type: ElementType::Title,
            scale: 2,
        });
        y += box_h + 8; // margin
    }

    // Paragraphs: collect first N paragraphs
    let p_sel = Selector::parse("p").unwrap();
    for p in document.select(&p_sel) {
        let txt = p.text().collect::<String>();
        let padding = 6u32;
        // estimate lines: char width 8px
        let content_w = page_width.saturating_sub(16) - padding * 2;
        let chars_per_line = if content_w >= 8 { (content_w / 8) as usize } else { 1 };
        // wrap
        let mut lines = Vec::new();
        let mut cur = String::new();
        for word in txt.split_whitespace() {
            if cur.len() + word.len() + 1 > chars_per_line && !cur.is_empty() {
                lines.push(cur);
                cur = word.to_string();
            } else {
                if !cur.is_empty() { cur.push(' '); }
                cur.push_str(word);
            }
        }
        if !cur.is_empty() { lines.push(cur); }
        let text = lines.join("\n");
        let lines_count = (text.lines().count() as u32).max(1);
        let box_h = lines_count * 8 + padding * 2;

        let lb = LayoutBox {
            rect: Rect {
                x: 8,
                y: y as i32,
                width: page_width.saturating_sub(16),
                height: box_h,
            },
            box_model: BoxModel {
                margin: 6,
                border: 0,
                padding,
            },
        };
        nodes.push(LayoutNode {
            lb,
            text: text.trim().to_string(),
            elem_type: ElementType::Paragraph,
            scale: 1,
        });
        y += box_h + 6;
        // Stop if running out of vertical space
        if y >= viewport.height { break; }
    }

    nodes
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn layout_document_places_title_and_paragraphs() {
        let html = "<html><head><title>Test Title</title></head><body><h1>Heading</h1><p>Hello world</p><p>More text</p></body></html>";
        let doc = Html::parse_document(html);
        let v = crate::Viewport { width: 200, height: 200 };
        let nodes = layout_document(&doc, v);
        assert!(!nodes.is_empty());
        assert_eq!(nodes[0].elem_type, ElementType::Title);
        assert_eq!(nodes[1].elem_type, ElementType::Paragraph);
        assert!(nodes[1].lb.rect.width > 0);
    }
}
