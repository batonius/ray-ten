use macroquad::prelude::*;

use crate::ai::control_far_paddle;
use crate::math::Directions;
use crate::motion::{MotionResult, MotionTicker};
use crate::render::{camera::Camera, renderer::Renderer};
use crate::scene::{Obstacle, Plane, Scene, Sphere};
use crate::ui;
use std::time::Instant;

const MAX_DEPTH: usize = 5;
const SAMPLES_PER_PIXEL: usize = 4;
const MENU_CHANGE_TIMEOUT_MS: u128 = 300;

struct GameState {
    camera: Camera,
    scene: Scene,
    motion_ticker: MotionTicker,
    renderer: Renderer,
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
    ) -> isize {
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
            MotionResult::Colision(Obstacle::Plane(Plane::Near)) => -1,
            MotionResult::Colision(Obstacle::Plane(Plane::Far)) => 1,
            _ => 0,
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

pub struct GameDriver {
    width: u16,
    height: u16,
    game_state: GameState,
    ui_state: UIState,
    current_selected_item: usize,
    last_selection_changed_at: Instant,
    score: isize,
    image: Image,
    texture: Texture2D,
}

impl GameDriver {
    pub fn new(width: u16, height: u16) -> Self {
        let image = Image::gen_image_color(width, height, WHITE);
        let texture = Texture2D::from_image(&image);
        GameDriver {
            width,
            height,
            game_state: GameState::new(width, height),
            ui_state: UIState::MainMenu,
            current_selected_item: 0,
            last_selection_changed_at: Instant::now(),
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
        if self.last_selection_changed_at.elapsed().as_millis() < MENU_CHANGE_TIMEOUT_MS {
            return None;
        }
        match self.ui_state {
            UIState::Hud => {
                if is_key_down(KeyCode::Escape) {
                    self.last_selection_changed_at = Instant::now();
                    return Some(Action::Pause);
                }
            }
            UIState::MainMenu | UIState::PauseMenu | UIState::EndGame => {
                if is_key_down(KeyCode::Enter) {
                    self.last_selection_changed_at = Instant::now();
                    return Some(self.ui_state.menu_items()[self.current_selected_item].1);
                }

                if is_key_down(KeyCode::Escape) && self.ui_state == UIState::PauseMenu {
                    self.last_selection_changed_at = Instant::now();
                    return Some(Action::Continue);
                }
                if self.last_selection_changed_at.elapsed().as_millis() > MENU_CHANGE_TIMEOUT_MS {
                    if is_key_down(KeyCode::Up) {
                        self.current_selected_item =
                            (self.current_selected_item - 1) % self.ui_state.menu_items().len();
                        self.last_selection_changed_at = Instant::now();
                    }
                    if is_key_down(KeyCode::Down) {
                        self.current_selected_item =
                            (self.current_selected_item + 1) % self.ui_state.menu_items().len();
                        self.last_selection_changed_at = Instant::now();
                    }
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
                self.last_selection_changed_at = Instant::now();
            }
            Action::EndGame => {
                self.ui_state = UIState::EndGame;
                self.current_selected_item = 0;
                self.last_selection_changed_at = Instant::now();
            }
            Action::Continue => {
                self.ui_state = UIState::Hud;
            }
            Action::MainMenu => {
                self.ui_state = UIState::MainMenu;
                self.current_selected_item = 0;
                self.last_selection_changed_at = Instant::now();
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
                is_key_down(KeyCode::Up),
                is_key_down(KeyCode::Down),
                is_key_down(KeyCode::Left),
                is_key_down(KeyCode::Right),
            )
        } else {
            far_directions
        };
        self.score += self
            .game_state
            .advance(get_frame_time(), near_directions, far_directions);
    }

    fn draw(&mut self) {
        match self.ui_state {
            UIState::MainMenu => {
                self.game_state.render(0.3, &mut self.image);
                self.texture.update(&self.image);
                draw_texture(self.texture, 0.0, 0.0, WHITE);
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
                draw_texture(self.texture, 0.0, 0.0, WHITE);
                ui::show_hud_top_left(format!("Score: {}", self.score).as_str());
            }
            UIState::PauseMenu => {
                self.game_state.render(0.3, &mut self.image);
                self.texture.update(&self.image);
                draw_texture(self.texture, 0.0, 0.0, WHITE);
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
    }
}
