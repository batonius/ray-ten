use macroquad::prelude::*;

lazy_static! {
    static ref FONT: Font =
        load_ttf_font_from_bytes(include_bytes!("../assets/LiberationMono-Regular.ttf"))
            .expect("Can't load font.");
}

const TITLE_FG_COLOR: Color = RED;
const TITLE_SHADE_COLOR: Color = WHITE;
const TITLE_FONT_SIZE: u16 = 150;

const MENU_FG_COLOR: Color = BLUE;
const MENU_SHADE_COLOR: Color = WHITE;
const MENU_FONT_SIZE: u16 = 100;
const MENU_LINE_SPACING: f32 = 1.2;

const HUD_FG_COLOR: Color = GREEN;
const HUD_SHADE_COLOR: Color = GRAY;
const HUD_FONT_SIZE: u16 = 80;

const SHADE_OFFSET: f32 = 2.0;

pub fn show_title(title: &str) {
    let title_dimensions = measure_text(title, Some(*FONT), TITLE_FONT_SIZE, 1.0);
    let screen_width = screen_width();
    let screen_height = screen_height();

    draw_text_ex(
        title,
        (screen_width - title_dimensions.width) / 2.0 + SHADE_OFFSET,
        (screen_height / 2.0 - title_dimensions.height) / 2.0
            + title_dimensions.height
            + SHADE_OFFSET,
        TextParams {
            color: TITLE_SHADE_COLOR,
            font_size: TITLE_FONT_SIZE,
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
            font_size: TITLE_FONT_SIZE,
            font: *FONT,
            ..Default::default()
        },
    );
}

pub fn show_menu<I: IntoIterator<Item = String>>(items: I, selected_item: usize) {
    let screen_width = screen_width();
    let screen_height = screen_height();

    let mut items = items.into_iter().collect::<Vec<_>>();
    items[selected_item] = format!("> {} <", items[selected_item]);

    let dimensions = items
        .iter()
        .map(|str| measure_text(str.as_str(), Some(*FONT), MENU_FONT_SIZE, 1.0))
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
                font_size: MENU_FONT_SIZE,
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
                font_size: MENU_FONT_SIZE,
                font: *FONT,
                ..Default::default()
            },
        );
    }
}

pub fn show_hud_top_left(text: &str) {
    let title_dimensions = measure_text(text, Some(*FONT), HUD_FONT_SIZE, 1.0);
    draw_text_ex(
        text,
        0.0 + SHADE_OFFSET,
        title_dimensions.height + SHADE_OFFSET,
        TextParams {
            color: HUD_SHADE_COLOR,
            font_size: HUD_FONT_SIZE,
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
            font_size: HUD_FONT_SIZE,
            font: *FONT,
            ..Default::default()
        },
    );
}

pub fn show_hud_top_right(text: &str) {
    let text_dimensions = measure_text(text, Some(*FONT), HUD_FONT_SIZE, 1.0);
    let screen_width = screen_width();
    draw_text_ex(
        text,
        screen_width - text_dimensions.width + SHADE_OFFSET,
        text_dimensions.height + SHADE_OFFSET,
        TextParams {
            color: HUD_SHADE_COLOR,
            font_size: HUD_FONT_SIZE,
            font: *FONT,
            ..Default::default()
        },
    );

    draw_text_ex(
        text,
        screen_width - text_dimensions.width,
        text_dimensions.height,
        TextParams {
            color: HUD_FG_COLOR,
            font_size: HUD_FONT_SIZE,
            font: *FONT,
            ..Default::default()
        },
    );
}
