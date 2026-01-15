/// Very small paint command set for Phase 1

#[derive(Debug, Clone, PartialEq)]
pub enum PaintCommand {
    SolidRect {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        rgba: (u8, u8, u8, u8),
    },
    Text {
        x: i32,
        y: i32,
        text: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_command_debug() {
        let cmd = PaintCommand::SolidRect {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
            rgba: (255, 0, 0, 255),
        };
        match cmd {
            PaintCommand::SolidRect { width, .. } => assert_eq!(width, 10),
            _ => panic!("unexpected"),
        }
    }
}
