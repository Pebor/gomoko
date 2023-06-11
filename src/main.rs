use std::sync::Arc;

use rand::seq::SliceRandom;
use sfml::{
    graphics::{
        Font, PrimitiveType, RectangleShape, Text, Vertex, VertexBuffer, VertexBufferUsage,
    },
    window::mouse,
};

use sfml::{
    graphics::{CircleShape, Color, RenderTarget, RenderWindow, Shape, Transformable},
    system::Vector2f,
    window::{ContextSettings, Event, Key, Style},
};

macro_rules! board_loop {
    ($g_x:ident, $g_y:ident, $code:block) => {
        for $g_y in 0..GRID_SIZE as usize {
            for $g_x in 0..GRID_SIZE as usize {
                $code
            }
        }
    };
}

const APP_WIDTH: u32 = 800;
const APP_HEIGHT: u32 = 800;
const GRID_SIZE: u32 = 20;

#[derive(Clone, Copy, PartialEq)]
enum BoardSpot {
    Black,
    White,
}

#[derive(Default, Clone, Copy, Debug)]
struct ChoosingCell {
    attack: u8,
    defense: u8,
    must: bool,
}

#[derive(Clone, Copy)]
enum Mode {
    Attacking,
    Defending,
}

#[derive(PartialEq)]
enum GameState {
    Menu,
    InGame,
    End { player_won: bool },
}

struct Button<'a> {
    body: RectangleShape<'a>,
    text: Text<'a>,
}

impl<'a> Button<'a> {
    fn new(x: i32, y: i32, w: i32, h: i32, text: &str, font: &'a Font) -> Self {
        let mut body = RectangleShape::with_size(Vector2f::new(w as f32, h as f32));
        let bounds = body.local_bounds();
        body.set_origin(Vector2f::new(bounds.width / 2.0, bounds.height / 2.0));
        body.set_position(Vector2f::new(x as f32, y as f32));
        body.set_fill_color(Color::BLACK);
        body.set_outline_color(Color::WHITE);
        body.set_outline_thickness(2.0);

        let mut text = Text::new(text, &font, (w as f32 / 4.25) as u32);
        let bounds = text.local_bounds();
        text.set_origin(Vector2f::new(bounds.width / 2.0, bounds.height / 2.0));
        text.set_position(Vector2f::new(x as f32, y as f32));
        text.move_(-Vector2f::new(bounds.left, bounds.top));

        Self { body, text }
    }

    fn render(&self, win: &mut RenderWindow) {
        win.draw(&self.body);
        win.draw(&self.text);
    }

    fn is_clicked(&self, x: i32, y: i32) -> bool {
        self.body.global_bounds().contains2(x as f32, y as f32)
    }
}

fn main() {
    let context_settings = ContextSettings {
        antialiasing_level: 1,
        ..Default::default()
    };

    let mut win = RenderWindow::new(
        (APP_WIDTH, APP_HEIGHT),
        "Gomokus",
        Style::CLOSE,
        &context_settings,
    );

    win.set_vertical_sync_enabled(true);

    let main_font = Font::from_file("res/Arial.ttf").unwrap();

    const GRID_W: u32 = APP_WIDTH / GRID_SIZE;
    const GRID_H: u32 = APP_HEIGHT / GRID_SIZE;
    const LINE_COL: Color = Color::BLACK;

    let mut show_debug = false;

    let mut state = GameState::Menu;

    let mut board = [[None; GRID_SIZE as usize]; GRID_SIZE as usize];
    let mut turn = true; // False - White; True - Black

    let mut ai_grid: DebugGrid = [[ChoosingCell {
        attack: 0,
        defense: 0,
        must: false,
    }; 20]; 20];

    let mut temp: Vec<Vertex> = vec![];
    for i in 0..GRID_SIZE {
        temp.push(Vertex::with_pos_color(
            Vector2f::new((i * GRID_W) as f32, 0.0),
            LINE_COL,
        ));
        temp.push(Vertex::with_pos_color(
            Vector2f::new((i * GRID_W) as f32, APP_HEIGHT as f32),
            LINE_COL,
        ));
    }

    for i in 0..GRID_SIZE {
        temp.push(Vertex::with_pos_color(
            Vector2f::new(0.0, (i * GRID_H) as f32),
            LINE_COL,
        ));
        temp.push(Vertex::with_pos_color(
            Vector2f::new(APP_WIDTH as f32, (i * GRID_H) as f32),
            LINE_COL,
        ));
    }

    let mut board_lines = VertexBuffer::new(
        PrimitiveType::LINES,
        GRID_SIZE * 2,
        VertexBufferUsage::STATIC,
    );
    board_lines.update(temp.as_slice(), 0);

    let draw_game = |board: &[[Option<BoardSpot>; 20]; 20],
                     ai_grid: &[[ChoosingCell; 20]; 20],
                     show_debug: &bool,
                     win: &mut RenderWindow| {
        win.clear(Color::rgb(155, 100, 55));
        win.draw(&board_lines);
        board_loop!(g_y, g_x, {
            if let Some(b_spot) = board[g_y][g_x] {
                let mut spot = CircleShape::new((GRID_W / 2) as f32, 48);
                spot.set_position(Vector2f::new(
                    (g_x * GRID_W as usize) as f32,
                    (g_y * GRID_H as usize) as f32,
                ));

                spot.set_fill_color(match b_spot {
                    BoardSpot::White => Color::WHITE,
                    BoardSpot::Black => Color::BLACK,
                });

                win.draw(&spot);
            }

            let cell = ai_grid[g_y][g_x];

            if !(cell.attack == 0 && cell.defense == 0) && *show_debug {
                let mut info_text = format!("{}", cell.attack);
                let mut info = Text::new(&info_text, &main_font, 16);
                info.set_position(Vector2f::new(
                    (g_x * GRID_W as usize) as f32,
                    (g_y * GRID_H as usize) as f32,
                ));
                info.set_fill_color(Color::RED);
                win.draw(&info);

                info_text = format!("{}", cell.defense);
                info = Text::new(&info_text, &main_font, 16);
                info.set_position(Vector2f::new(
                    (g_x * GRID_W as usize) as f32,
                    (g_y * GRID_H as usize + GRID_H as usize - 16) as f32,
                ));
                info.set_fill_color(Color::BLUE);
                win.draw(&info);
            }
        });
    };
    let play_btn = Button::new(
        APP_WIDTH as i32 / 2,
        APP_HEIGHT as i32 / 2 - 50,
        200,
        80,
        "Play",
        &main_font,
    );
    let quit_btn = Button::new(
        APP_WIDTH as i32 / 2,
        APP_HEIGHT as i32 / 2 + 50,
        200,
        80,
        "Exit",
        &main_font,
    );

    let retry_btn = Button::new(
        APP_WIDTH as i32 / 2,
        APP_HEIGHT as i32 / 2 + 50,
        200,
        80,
        "Retry?",
        &main_font,
    );

    loop {
        while let Some(event) = win.poll_event() {
            match event {
                Event::Closed
                | Event::KeyPressed {
                    code: Key::Escape, ..
                } => state = GameState::Menu,
                Event::KeyPressed { code: Key::F1, .. } => show_debug = !show_debug,
                Event::MouseButtonPressed { button, x, y } => {
                    if let mouse::Button::Left = button {
                        match state {
                            GameState::Menu => {
                                if play_btn.is_clicked(x, y) {
                                    board = [[None; GRID_SIZE as usize]; GRID_SIZE as usize];
                                    turn = true;
                                    state = GameState::InGame;
                                } else if quit_btn.is_clicked(x, y) {
                                    return;
                                }
                            }
                            GameState::InGame => {
                                if turn {
                                    let x = (x as f32 / GRID_W as f32).floor() as usize;
                                    let y = (y as f32 / GRID_H as f32).floor() as usize;

                                    if let None = board[y][x] {
                                        board[y][x] = Some(BoardSpot::White);
                                        turn = !turn;
                                    }
                                }
                            }
                            GameState::End { player_won } => {
                                if retry_btn.is_clicked(x, y) {
                                    board = [[None; GRID_SIZE as usize]; GRID_SIZE as usize];
                                    turn = true;
                                    state = GameState::InGame;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        match state {
            GameState::Menu => {
                win.clear(Color::BLACK);

                play_btn.render(&mut win);
                quit_btn.render(&mut win);
            }
            GameState::InGame => {
                board_loop!(g_y, g_x, {
                    if let Some(spot) = board[g_y][g_x] {
                        for off_y in -1..=1 {
                            for off_x in -1..=1 {
                                let mut x = g_x as i32 + off_x;
                                let mut y = g_y as i32 + off_y;
                                if !(off_x == 0 && off_y == 0)
                                    && is_inbound(x, y)
                                    && board[y as usize][x as usize] == Some(spot)
                                {
                                    let mut count = 2;

                                    'find: loop {
                                        x += off_x;
                                        y += off_y;

                                        if is_inbound(x, y) {
                                            if board[y as usize][x as usize] == Some(spot) {
                                                count += 1;
                                                continue;
                                            }
                                        }
                                        break 'find;
                                    }

                                    if count >= 5 {
                                        state = match spot {
                                            BoardSpot::Black => {
                                                GameState::End { player_won: false }
                                            }
                                            BoardSpot::White => GameState::End { player_won: true },
                                        }
                                    }
                                }
                            }
                        }
                    }
                });

                if !turn {
                    let (x, y, ai) = ai_choose(board.clone());
                    ai_grid = ai;
                    board[y][x] = Some(BoardSpot::Black);
                    turn = !turn;
                }

                draw_game(&board, &ai_grid, &show_debug, &mut win);
            }
            GameState::End { player_won } => {
                draw_game(&board, &ai_grid, &show_debug, &mut win);

                let mut overlay_bg = RectangleShape::new();
                overlay_bg.set_size(Vector2f::new(APP_WIDTH as f32, APP_WIDTH as f32));
                overlay_bg.set_fill_color(Color::rgba(0, 0, 0, 100));

                let won_str = match player_won {
                    true => "You Won! :)",
                    false => "AI won :(",
                };

                let mut win_text = Text::new(won_str, &main_font, 64);
                let txt_rect = win_text.local_bounds();
                win_text.set_origin(Vector2f::new(txt_rect.width / 2.0, txt_rect.height / 2.0));
                win_text.set_position(Vector2f::new(
                    APP_WIDTH as f32 / 2.0,
                    APP_HEIGHT as f32 / 2.0 - 100.0,
                ));
                win_text.set_fill_color(match player_won {
                    true => Color::GREEN,
                    false => Color::RED,
                });
                win_text.set_outline_color(Color::BLACK);
                win_text.set_outline_thickness(3.0);

                win.draw(&overlay_bg);
                win.draw(&win_text);
                retry_btn.render(&mut win);
            }
        }

        win.display();
    }
}

type DebugGrid = [[ChoosingCell; 20]; 20];

fn ai_choose(board: [[Option<BoardSpot>; 20]; 20]) -> (usize, usize, DebugGrid) {
    let mut ai_grid: DebugGrid = [[ChoosingCell::default(); 20]; 20];
    let tmp_grid = board;

    board_loop!(g_y, g_x, {
        if tmp_grid[g_y][g_x].is_none() {
            let mut tmp = vec![];

            for off_y in -1..=1 {
                for off_x in -1..=1 {
                    let mut x = g_x as i32 + off_x;
                    let mut y = g_y as i32 + off_y;
                    let mut att = 0;
                    let mut def = 0;
                    let mut final_att = 0;
                    let mut final_def = 0;
                    let mut must = false;

                    if !(off_x == 0 && off_y == 0) && is_inbound(x, y) {
                        if let Some(spot) = tmp_grid[y as usize][x as usize] {
                            let mode = match spot {
                                BoardSpot::Black => Mode::Attacking,
                                BoardSpot::White => Mode::Defending,
                            };

                            let mut att_penalty = 0;
                            let mut def_penalty = 0;
                            for dir in [1, -1] {
                                x = g_x as i32;
                                y = g_y as i32;
                                loop {
                                    x += off_x * dir;
                                    y += off_y * dir;

                                    if is_inbound(x, y) {
                                        if let Some(other_spot) = tmp_grid[y as usize][x as usize] {
                                            match (other_spot, mode) {
                                                (BoardSpot::Black, Mode::Attacking) => {
                                                    att += 10;
                                                    continue;
                                                }
                                                (BoardSpot::Black, Mode::Defending) => {
                                                    def_penalty += 1;
                                                    break;
                                                }
                                                (BoardSpot::White, Mode::Attacking) => {
                                                    att_penalty += 1;
                                                    break;
                                                }
                                                (BoardSpot::White, Mode::Defending) => {
                                                    def += 10;
                                                    continue;
                                                }
                                            }
                                        }
                                    } else {
                                        def_penalty += 1;
                                        att_penalty += 1;
                                        break;
                                    }

                                    break;
                                }
                                if !must {
                                    must = def >= 40 || att >= 40;
                                }

                                // att /= 1 + (1 * att_penalty as u8);
                                // def /= 1 + (1 * def_penalty as u8);

                                final_att += att;
                                final_def += def;

                                att = 0;
                                def = 0;
                            }

                            if !must {
                                must = final_def >= 40 || final_att >= 40;
                            }

                            final_att /= 1 + att_penalty;
                            final_def /= 1 + def_penalty;

                            tmp.push(ChoosingCell {
                                attack: final_att,
                                defense: final_def,
                                must,
                            });
                        }
                    }
                }
            }

            if tmp.len() >= 1 {
                let mut highest_att = 0;
                let mut highest_def = 0;
                let mut att_index = 0;
                let mut def_index = 0;

                for (i, cell) in tmp.iter().enumerate() {
                    if cell.attack > highest_att {
                        highest_att = cell.attack;
                        att_index = i;
                    }
                    if cell.defense > highest_def {
                        highest_def = cell.defense;
                        def_index = i;
                    }
                }

                if highest_def > highest_att {
                    ai_grid[g_y][g_x] = tmp[def_index];
                } else if highest_att > highest_def {
                    ai_grid[g_y][g_x] = tmp[att_index];
                } else {
                    ai_grid[g_y][g_x] = tmp[def_index];
                }
            }
        }
    });

    #[derive(Default, Debug)]
    struct Pos {
        x: usize,
        y: usize,
    }

    let mut highest_att = 0;
    let mut highest_def = 0;

    let mut att_pos = vec![];
    let mut def_pos = vec![];

    'sort_loop: for ai_y in 0..GRID_SIZE as usize {
        for ai_x in 0..GRID_SIZE as usize {
            let choosing_cell = ai_grid[ai_y][ai_x];

            if choosing_cell.must {
                att_pos.clear();
                def_pos.clear();
                att_pos.push(Pos { x: ai_x, y: ai_y });
                def_pos.push(Pos { x: ai_x, y: ai_y });
                highest_att = 0;
                highest_def = 0;
                break 'sort_loop;
            }

            if choosing_cell.attack != 0 && choosing_cell.attack == highest_att {
                att_pos.push(Pos { x: ai_x, y: ai_y });
            } else if choosing_cell.attack > highest_att {
                att_pos.clear();
                att_pos.push(Pos { x: ai_x, y: ai_y });
                highest_att = choosing_cell.attack;
            }

            if choosing_cell.defense != 0 && choosing_cell.defense == highest_def {
                def_pos.push(Pos { x: ai_x, y: ai_y });
            } else if choosing_cell.defense > highest_def {
                def_pos.clear();
                def_pos.push(Pos { x: ai_x, y: ai_y });
                highest_def = choosing_cell.defense;
            }
        }
    }

    let pos;

    if highest_att == 0 && highest_def == 0 {
        pos = &att_pos[0];
    } else if highest_att > highest_def {
        pos = att_pos.choose(&mut rand::thread_rng()).unwrap();
    } else if highest_def > highest_att {
        pos = def_pos.choose(&mut rand::thread_rng()).unwrap();
    } else {
        pos = att_pos.choose(&mut rand::thread_rng()).unwrap();
    }

    (pos.x, pos.y, ai_grid)
}

fn is_inbound(x: i32, y: i32) -> bool {
    x >= 0 && x < GRID_SIZE as i32 && y >= 0 && y < GRID_SIZE as i32
}
