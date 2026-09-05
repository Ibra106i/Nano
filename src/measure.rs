use cosmic_text::{FontSystem, SwashCache, Buffer, Metrics, Shaping, Attrs, Family, Wrap};

pub struct TextMeasurer {
    font_system: FontSystem,
    cache: SwashCache,
}

impl std::fmt::Debug for TextMeasurer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextMeasurer").finish()
    }
}

impl Clone for TextMeasurer {
    fn clone(&self) -> Self {
        TextMeasurer::new()
    }
}

impl TextMeasurer {
    pub fn new() -> Self {
        let mut font_system = FontSystem::new();
        font_system.db_mut().load_system_fonts();
        TextMeasurer {
            font_system,
            cache: SwashCache::new(),
        }
    }

    fn make_attrs() -> Attrs<'static> {
        Attrs::new().family(Family::Name("Source Serif 4"))
    }

    pub fn col_from_x(&mut self, text: &str, target_x: f32, font_size: f32) -> usize {
        let metrics = Metrics::new(font_size, font_size * 1.4);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        buffer.set_size(&mut self.font_system, Some(5000.0), Some(font_size * 2.0));
        buffer.set_text(
            &mut self.font_system,
            text,
            Self::make_attrs(),
            Shaping::Advanced,
        );

        let mut x = 0.0;
        let mut col = 0;
        let chars: Vec<char> = text.chars().collect();

        for line in buffer.lines.iter_mut() {
            let layout = line.layout(
                &mut self.font_system,
                font_size,
                None,
                Wrap::None,
                None,
                4,
            );
            for layout_line in layout {
                for glyph in &layout_line.glyphs {
                    if x + glyph.w > target_x {
                        return col;
                    }
                    x += glyph.w;
                    col += 1;
                }
            }
        }
        col.min(chars.len())
    }
}
