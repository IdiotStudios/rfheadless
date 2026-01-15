/// Very small layout primitives for Phase 1 prototype

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_width_accounts_for_box_model() {
        let lb = LayoutBox {
            rect: Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 100,
            },
            box_model: BoxModel {
                margin: 10,
                border: 2,
                padding: 5,
            },
        };
        assert_eq!(lb.content_width(), 183);
    }
}
