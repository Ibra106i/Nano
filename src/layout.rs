pub const TITLE_BAR_H: f32 = 38.0;
pub const MENU_BAR_H: f32 = 32.0;
pub const TOOLBAR_H: f32 = 38.0;
pub const RULER_H: f32 = 22.0;
pub const STATUS_BAR_H: f32 = 26.0;
pub const PAGE_WIDTH: f32 = 614.0;
pub const PAGE_PADDING: f32 = 24.0;
pub const LINE_NUMBER_WIDTH: f32 = 40.0;
pub const GUTTER_SPACING: f32 = 16.0;
pub const LINE_NUMBER_GUTTER_LEFT: f32 = LINE_NUMBER_WIDTH + GUTTER_SPACING + 1.0;

pub fn page_top() -> f32 {
    TITLE_BAR_H + MENU_BAR_H + TOOLBAR_H + RULER_H
}

pub fn page_left() -> f32 {
    LINE_NUMBER_GUTTER_LEFT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_top_is_sum_of_bars() {
        assert_eq!(page_top(), TITLE_BAR_H + MENU_BAR_H + TOOLBAR_H + RULER_H);
    }

    #[test]
    fn page_left_is_gutter_left() {
        assert_eq!(page_left(), LINE_NUMBER_GUTTER_LEFT);
    }

    #[test]
    fn gutter_left_is_sum() {
        assert_eq!(LINE_NUMBER_GUTTER_LEFT, LINE_NUMBER_WIDTH + GUTTER_SPACING + 1.0);
    }
}
