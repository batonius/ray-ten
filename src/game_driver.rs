use macroquad::prelude::*;

use crate::ai::control_far_paddle;
use crate::math::Directions;
use crate::motion::{MotionResult, MotionTicker};
use crate::render::{camera::Camera, renderer::Renderer};
use crate::scene::{Obstacle, Plane, Scene, Sphere};
use crate::ui;

const MAX_DEPTH: usize = 5;
const SAMPLES_PER_PIXEL: usize = 1;
const MENU_CHANGE_TIMEOUT: f32 = 0.2;
const COLLISION_NOTICE_TIMEOUT: f32 = 1.0;

struct GameState {
    camera: Camera,
    scene: Scene,
    motion_ticker: MotionTicker,
    renderer: Renderer,
}

#[derive(Clone, Copy)]
enum AdvanceResult {
    PlayerHit,
    AIHit,
    PlayerMiss,
    AIMiss,
    None,
}

impl GameState {
    fn new(width: u16, height: u16) -> Self {
        Self {
            camera: Camera::new(width as f32 / height as f32, 2.0f32),
            scene: Scene::new(),
            motion_ticker: MotionTicker::new(),
            renderer: Renderer::new((width, height), SAMPLES_PER_PIXEL, MAX_DEPTH),
        }
    }

    fn advance(
        &mut self,
        elapsed: f32,
        near_paddle_directions: Directions,
        far_paddle_directions: Directions,
    ) -> AdvanceResult {
        let motion_result = self.motion_ticker.tick(
            &mut self.scene,
            elapsed,
            near_paddle_directions,
            far_paddle_directions,
        );
        let near_paddle_pos = self.scene.sphere_pos(Sphere::NearPaddle);
        self.camera
            .move_origin_to(near_paddle_pos.x(), near_paddle_pos.y());

        match motion_result {
            MotionResult::Colision(Obstacle::Plane(Plane::Near)) => AdvanceResult::PlayerMiss,
            MotionResult::Colision(Obstacle::Plane(Plane::Far)) => AdvanceResult::AIMiss,
            MotionResult::Colision(Obstacle::Sphere(Sphere::NearPaddle)) => {
                AdvanceResult::PlayerHit
            }
            MotionResult::Colision(Obstacle::Sphere(Sphere::FarPaddle)) => AdvanceResult::AIHit,
            _ => AdvanceResult::None,
        }
    }

    fn render(&self, coef: f32, image: &mut Image) {
        self.renderer
            .render(&self.scene, &self.camera, coef, image.get_image_data_mut());
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Action {
    NewGame,
    Exit,
    Pause,
    EndGame,
    Continue,
    MainMenu,
}

const MENU_ITEMS: [&[(&str, Action)]; 4] = [
    &[("New game", Action::NewGame), ("Exit", Action::Exit)],
    &[],
    &[
        ("Continue", Action::Continue),
        ("Main menu", Action::EndGame),
        ("Exit", Action::Exit),
    ],
    &[("Main menu", Action::MainMenu)],
];

#[derive(Clone, Copy, PartialEq)]
enum UIState {
    MainMenu,
    Hud,
    PauseMenu,
    EndGame,
}

impl UIState {
    const fn menu_items(&self) -> &'static [(&'static str, Action)] {
        MENU_ITEMS[*self as usize]
    }
}

#[derive(Clone, Copy, PartialEq)]
enum TouchDirection {
    Center,
    Left,
    Right,
    Top,
    Bottom,
}

pub struct GameDriver {
    width: u16,
    height: u16,
    game_state: GameState,
    ui_state: UIState,
    current_selected_item: usize,
    since_last_selection_change: f32,
    since_last_collision: f32,
    last_collision: bool,
    score: isize,
    image: Image,
    texture: Texture2D,
}

impl GameDriver {
    pub fn new(width: u16, height: u16) -> Self {
        let image = Image::gen_image_color(width, height, WHITE);
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);
        GameDriver {
            width,
            height,
            game_state: GameState::new(width, height),
            ui_state: UIState::MainMenu,
            current_selected_item: 0,
            since_last_selection_change: 0.0,
            since_last_collision: COLLISION_NOTICE_TIMEOUT + 1.0,
            last_collision: false,
            score: 0,
            image,
            texture,
        }
    }

    pub fn next_frame(&mut self) -> bool {
        if let Some(action) = self.process_inputs() {
            if let Action::Exit = action {
                return false;
            }
            self.process_action(action);
        }
        self.advance();
        self.draw();
        true
    }

    fn process_inputs(&mut self) -> Option<Action> {
        self.since_last_selection_change += get_frame_time();
        self.since_last_collision += get_frame_time();

        if self.since_last_selection_change < MENU_CHANGE_TIMEOUT {
            return None;
        }
        match self.ui_state {
            UIState::Hud => {
                if is_key_down(KeyCode::Escape)
                    || Self::is_direction_clicked(TouchDirection::Center)
                {
                    self.since_last_selection_change = 0.0;
                    return Some(Action::Pause);
                }
            }
            UIState::MainMenu | UIState::PauseMenu | UIState::EndGame => {
                if is_key_down(KeyCode::Enter) || Self::is_direction_clicked(TouchDirection::Center)
                {
                    self.since_last_selection_change = 0.0;
                    return Some(self.ui_state.menu_items()[self.current_selected_item].1);
                }

                if is_key_down(KeyCode::Escape) && self.ui_state == UIState::PauseMenu {
                    self.since_last_selection_change = 0.0;
                    return Some(Action::Continue);
                }
                if is_key_down(KeyCode::Up)
                    || Self::is_direction_clicked(TouchDirection::Top)
                    || Self::is_direction_clicked(TouchDirection::Left)
                {
                    self.current_selected_item =
                        (self.current_selected_item - 1) % self.ui_state.menu_items().len();
                    self.since_last_selection_change = 0.0;
                }
                if is_key_down(KeyCode::Down)
                    || Self::is_direction_clicked(TouchDirection::Bottom)
                    || Self::is_direction_clicked(TouchDirection::Right)
                {
                    self.current_selected_item =
                        (self.current_selected_item + 1) % self.ui_state.menu_items().len();
                    self.since_last_selection_change = 0.0;
                }
            }
        }
        None
    }

    fn process_action(&mut self, action: Action) {
        match action {
            Action::NewGame => {
                self.game_state = GameState::new(self.width, self.height);
                self.ui_state = UIState::Hud;
                self.score = 0;
            }
            Action::Exit => (),
            Action::Pause => {
                self.ui_state = UIState::PauseMenu;
                self.current_selected_item = 0;
                self.since_last_selection_change = 0.0;
            }
            Action::EndGame => {
                self.ui_state = UIState::EndGame;
                self.current_selected_item = 0;
                self.since_last_selection_change = 0.0;
            }
            Action::Continue => {
                self.ui_state = UIState::Hud;
            }
            Action::MainMenu => {
                self.ui_state = UIState::MainMenu;
                self.current_selected_item = 0;
                self.since_last_selection_change = 0.0;
            }
        }
    }

    fn advance(&mut self) {
        if let UIState::PauseMenu = self.ui_state {
            return;
        }
        let far_directions = control_far_paddle(&self.game_state.scene);
        let near_directions = if let UIState::Hud = self.ui_state {
            Directions::new(
                is_key_down(KeyCode::Up) || Self::is_direction_pressed(TouchDirection::Top),
                is_key_down(KeyCode::Down) || Self::is_direction_pressed(TouchDirection::Bottom),
                is_key_down(KeyCode::Left) || Self::is_direction_pressed(TouchDirection::Left),
                is_key_down(KeyCode::Right) || Self::is_direction_pressed(TouchDirection::Right),
            )
        } else {
            far_directions
        };

        match self
            .game_state
            .advance(get_frame_time(), near_directions, far_directions)
        {
            AdvanceResult::PlayerHit => {
                self.since_last_collision = 0.0;
                self.last_collision = true;
            }
            AdvanceResult::AIHit => {}
            AdvanceResult::PlayerMiss => {
                self.since_last_collision = 0.0;
                self.last_collision = false;
                self.score -= 1;
            }
            AdvanceResult::AIMiss => {
                self.score += 1;
            }
            AdvanceResult::None => {}
        }
    }

    fn draw(&mut self) {
        match self.ui_state {
            UIState::MainMenu => {
                self.game_state.render(0.3, &mut self.image);
                self.texture.update(&self.image);
                draw_texture_ex(
                    self.texture,
                    0.0,
                    0.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(screen_width(), screen_height())),
                        ..Default::default()
                    },
                );
                ui::show_title("ray ten");
                ui::show_menu(
                    UIState::MainMenu
                        .menu_items()
                        .iter()
                        .map(|&(text, _)| text.to_string()),
                    self.current_selected_item,
                )
            }
            UIState::Hud => {
                self.game_state.render(1.0, &mut self.image);
                self.texture.update(&self.image);
                draw_texture_ex(
                    self.texture,
                    0.0,
                    0.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(screen_width(), screen_height())),
                        ..Default::default()
                    },
                );
                ui::show_hud_top_left(format!("Score: {}", self.score).as_str());
                if self.since_last_collision < COLLISION_NOTICE_TIMEOUT {
                    if self.last_collision {
                        ui::show_hud_top_right("Hit!", true)
                    } else {
                        ui::show_hud_top_right("Miss!", false)
                    }
                }
            }
            UIState::PauseMenu => {
                self.game_state.render(0.3, &mut self.image);
                self.texture.update(&self.image);
                draw_texture_ex(
                    self.texture,
                    0.0,
                    0.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(screen_width(), screen_height())),
                        ..Default::default()
                    },
                );
                ui::show_title("Paused");
                ui::show_menu(
                    UIState::PauseMenu
                        .menu_items()
                        .iter()
                        .map(|&(text, _)| text.to_string()),
                    self.current_selected_item,
                )
            }
            UIState::EndGame => {
                ui::show_title(format!("Final score: {}", self.score).as_str());
                ui::show_menu(
                    UIState::EndGame
                        .menu_items()
                        .iter()
                        .map(|&(text, _)| text.to_string()),
                    self.current_selected_item,
                )
            }
        }
        ui::show_debug_bottom_left(
            format!(
                "fps: {}, rps: {:.4}M",
                get_fps(),
                ((self.height as f32) * (self.width as f32) * (SAMPLES_PER_PIXEL as f32)
                    / get_frame_time()
                    / 1_000_000f32)
            )
            .as_str(),
        );
    }

    fn is_pos_in_direction(direction: TouchDirection, pos: Vec2) -> bool {
        match direction {
            TouchDirection::Center => {
                pos.x >= -0.2 && pos.x <= 0.2 && pos.y >= -0.2 && pos.y <= 0.2
            }
            TouchDirection::Left => pos.x < -0.2,
            TouchDirection::Right => pos.x > 0.2,
            TouchDirection::Top => pos.y < -0.2,
            TouchDirection::Bottom => pos.y > 0.2,
        }
    }

    fn is_direction_pressed(direction: TouchDirection) -> bool {
        if !is_mouse_button_down(MouseButton::Left) {
            return false;
        }
        Self::is_pos_in_direction(direction, mouse_position_local())
    }

    fn is_direction_clicked(direction: TouchDirection) -> bool {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return false;
        }
        Self::is_pos_in_direction(direction, mouse_position_local())
    }
}
