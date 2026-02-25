use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

pub struct PixelFramebuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<(u8, u8, u8)>,
}

impl PixelFramebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![(0, 0, 0); (width * height) as usize],
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.pixels
            .resize((width * height) as usize, (0, 0, 0));
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.pixels.fill((0, 0, 0));
    }
}

pub struct HalfBlockWidget<'a> {
    pub framebuffer: &'a PixelFramebuffer,
}

impl<'a> Widget for HalfBlockWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let fb = self.framebuffer;
        if fb.width == 0 || fb.height == 0 {
            return;
        }

        for row in 0..area.height {
            let top_y = (row as u32) * 2;
            let bot_y = top_y + 1;

            for col in 0..area.width.min(fb.width as u16) {
                let top_pixel = if top_y < fb.height {
                    let idx = (top_y * fb.width + col as u32) as usize;
                    fb.pixels.get(idx).copied().unwrap_or((0, 0, 0))
                } else {
                    (0, 0, 0)
                };

                let bot_pixel = if bot_y < fb.height {
                    let idx = (bot_y * fb.width + col as u32) as usize;
                    fb.pixels.get(idx).copied().unwrap_or((0, 0, 0))
                } else {
                    (0, 0, 0)
                };

                let cell = buf.get_mut(area.x + col, area.y + row);
                cell.set_symbol("\u{2580}"); // ▀
                cell.set_style(
                    Style::default()
                        .fg(Color::Rgb(top_pixel.0, top_pixel.1, top_pixel.2))
                        .bg(Color::Rgb(bot_pixel.0, bot_pixel.1, bot_pixel.2)),
                );
            }
        }
    }
}
