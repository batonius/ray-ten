use macroquad::prelude::*;

lazy_static! {
    static ref FONT: Font =
        load_ttf_font_from_bytes(include_bytes!("../assets/LiberationMono-Regular.ttf"))
            .expect("Can't load font.");
}

const TITLE_FG_COLOR: Color = RED;
const TITLE_SHADE_COLOR: Color = WHITE;
const TITLE_FONT_SCALE: f32 = 0.2;

const MENU_FG_COLOR: Color = BLUE;
const MENU_SHADE_COLOR: Color = WHITE;
const MENU_FONT_SCALE: f32 = 0.1;
const MENU_LINE_SPACING: f32 = 1.2;

const HUD_FG_COLOR: Color = GREEN;
const HUD_BAD_FG_COLOR: Color = RED;
const HUD_SHADE_COLOR: Color = GRAY;
const HUD_FONT_SCALE: f32 = 0.1;

const DEBUG_FG_COLOR: Color = DARKGRAY;
const DEBUG_FONT_SCALE: f32 = 0.05;

const SHADE_OFFSET: f32 = 2.0;

pub fn show_title(title: &str) {
    let screen_width = screen_width();
    let screen_height = screen_height();
    let font_size = (TITLE_FONT_SCALE * screen_height) as u16;
    let title_dimensions = measure_text(title, Some(*FONT), font_size, 1.0);

    draw_text_ex(
        title,
        (screen_width - title_dimensions.width) / 2.0 + SHADE_OFFSET,
        (screen_height / 2.0 - title_dimensions.height) / 2.0
            + title_dimensions.height
            + SHADE_OFFSET,
        TextParams {
            color: TITLE_SHADE_COLOR,
            font_size,
            font: *FONT,
            ..Default::default()
        },
    );

    draw_text_ex(
        title,
        (screen_width - title_dimensions.width) / 2.0,
        (screen_height / 2.0 - title_dimensions.height) / 2.0 + title_dimensions.height,
        TextParams {
            color: TITLE_FG_COLOR,
            font_size,
            font: *FONT,
            ..Default::default()
        },
    );
}

pub fn show_menu<I: IntoIterator<Item = String>>(items: I, selected_item: usize) {
    let screen_width = screen_width();
    let screen_height = screen_height();

    let font_size = (MENU_FONT_SCALE * screen_height) as u16;

    let mut items = items.into_iter().collect::<Vec<_>>();
    items[selected_item] = format!("> {} <", items[selected_item]);

    let dimensions = items
        .iter()
        .map(|str| measure_text(str.as_str(), Some(*FONT), font_size, 1.0))
        .collect::<Vec<_>>();

    let menu_height = dimensions.iter().fold(0.0, |height, dimension| {
        height + dimension.height * MENU_LINE_SPACING
    });

    let mut y = screen_height / 2.0 + (screen_height / 2.0 - menu_height) / 2.0;

    for (item, dimension) in items.iter().zip(dimensions.iter()) {
        y += dimension.height * MENU_LINE_SPACING;

        draw_text_ex(
            item.as_str(),
            (screen_width - dimension.width) / 2.0 + SHADE_OFFSET,
            y + SHADE_OFFSET,
            TextParams {
                color: MENU_SHADE_COLOR,
                font_size,
                font: *FONT,
                ..Default::default()
            },
        );

        draw_text_ex(
            item.as_str(),
            (screen_width - dimension.width) / 2.0,
            y,
            TextParams {
                color: MENU_FG_COLOR,
                font_size,
                font: *FONT,
                ..Default::default()
            },
        );
    }
}

pub fn show_hud_top_left(text: &str) {
    let screen_height = screen_height();
    let font_size = (HUD_FONT_SCALE * screen_height) as u16;
    let title_dimensions = measure_text(text, Some(*FONT), font_size, 1.0);
    draw_text_ex(
        text,
        0.0 + SHADE_OFFSET,
        title_dimensions.height + SHADE_OFFSET,
        TextParams {
            color: HUD_SHADE_COLOR,
            font_size,
            font: *FONT,
            ..Default::default()
        },
    );

    draw_text_ex(
        text,
        0.0,
        title_dimensions.height,
        TextParams {
            color: HUD_FG_COLOR,
            font_size,
            font: *FONT,
            ..Default::default()
        },
    );
}

pub fn show_hud_top_right(text: &str, good: bool) {
    let screen_width = screen_width();
    let screen_height = screen_height();
    let font_size = (HUD_FONT_SCALE * screen_height) as u16;
    let color = if good { HUD_FG_COLOR } else { HUD_BAD_FG_COLOR };
    let text_dimensions = measure_text(text, Some(*FONT), font_size, 1.0);
    draw_text_ex(
        text,
        screen_width - text_dimensions.width + SHADE_OFFSET,
        text_dimensions.height + SHADE_OFFSET,
        TextParams {
            color: HUD_SHADE_COLOR,
            font_size,
            font: *FONT,
            ..Default::default()
        },
    );

    draw_text_ex(
        text,
        screen_width - text_dimensions.width,
        text_dimensions.height,
        TextParams {
            color,
            font_size,
            font: *FONT,
            ..Default::default()
        },
    );
}

pub fn show_debug_bottom_left(text: &str) {
    let screen_height = screen_height();
    let font_size = (DEBUG_FONT_SCALE * screen_height) as u16;
    let text_dimensions = measure_text(text, Some(*FONT), font_size, 1.0);

    draw_text_ex(
        text,
        0.0,
        screen_height - text_dimensions.offset_y,
        TextParams {
            color: DEBUG_FG_COLOR,
            font_size,
            font: *FONT,
            ..Default::default()
        },
    );
}
