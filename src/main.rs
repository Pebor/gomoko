use std::env::temp_dir;

use sfml::{
    graphics::{PrimitiveType, Vertex, VertexBuffer, VertexBufferUsage},
    window::mouse,
};

use sfml::{
    graphics::{CircleShape, Color, RenderTarget, RenderWindow, Shape, Transformable},
    system::Vector2f,
    window::{ContextSettings, Event, Key, Style},
};

const APP_WIDTH: u32 = 600;
const APP_HEIGHT: u32 = 600;
const GRID_SIZE: u32 = 20;

#[derive(Clone, Copy)]
enum BoardSpot {
    Black,
    White,
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

    let mut board = [[None; GRID_SIZE as usize]; GRID_SIZE as usize];
    let mut turn = true; // False - White; True - Black

    const GRID_W: u32 = APP_WIDTH / GRID_SIZE;
    const GRID_H: u32 = APP_HEIGHT / GRID_SIZE;
    const LINE_COL: Color = Color::BLACK;

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

    loop {
        while let Some(event) = win.poll_event() {
            match event {
                Event::Closed
                | Event::KeyPressed {
                    code: Key::Escape, ..
                } => return,
                Event::MouseButtonPressed { button, x, y } => {
                    if let mouse::Button::Left = button {
                        if turn {
                            let x = (x as f32 / GRID_W as f32).floor() as usize;
                            let y = (y as f32 / GRID_H as f32).floor() as usize;

                            if let None = board[y][x] {
                                board[y][x] = Some(BoardSpot::White);
                                turn = !turn;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if !turn {
            let (x, y, ai_grid) = ai_choose(board.clone());
            board[y][x] = Some(BoardSpot::Black);
            turn = !turn;
        }

        win.clear(Color::rgb(155, 100, 55));
        win.draw(&board_lines);
        for x in 0..GRID_SIZE as usize {
            for y in 0..GRID_SIZE as usize {
                if let Some(b_spot) = board[y][x] {
                    let mut spot = CircleShape::new((GRID_W / 2) as f32, 48);
                    spot.set_position(Vector2f::new(
                        (x * GRID_W as usize) as f32,
                        (y * GRID_H as usize) as f32,
                    ));

                    spot.set_fill_color(match b_spot {
                        BoardSpot::White => Color::WHITE,
                        BoardSpot::Black => Color::BLACK,
                    });

                    win.draw(&spot);
                }
            }
        }
        win.display();
    }
}

#[derive(Default, Clone, Copy)]
struct ChoosingCell {
    attack: u8,
    defense: u8,
}

#[derive(Clone, Copy)]
enum Mode {
    Attacking,
    Defending,
}

type DebugGrid = [[ChoosingCell; 20]; 20];

fn ai_choose(board: [[Option<BoardSpot>; 20]; 20]) -> (usize, usize, DebugGrid) {
    let mut ai_grid: DebugGrid = [[ChoosingCell {
        attack: 0,
        defense: 0,
    }; 20]; 20];
    let mut tmp_grid = board;

    for g_y in 0..GRID_SIZE as usize {
        for g_x in 0..GRID_SIZE as usize {
            if let None = tmp_grid[g_y][g_x] {
                for off_y in [-1, 1] {
                    for off_x in [-1, 1] {
                        let mut x = g_x as i32 + off_x;
                        let mut y = g_y as i32 + off_y;
                        let mut att = 0;
                        let mut def = 0;

                        if is_inbound(x, y) {
                            if let Some(spot) = tmp_grid[y as usize][x as usize] {
                                let mode = match spot {
                                    BoardSpot::Black => {
                                        att += 10;
                                        Mode::Attacking
                                    }
                                    BoardSpot::White => {
                                        def += 10;
                                        Mode::Defending
                                    }
                                };

                                loop {
                                    x += off_x;
                                    y += off_y;

                                    if is_inbound(x, y) {
                                        if let Some(other_spot) = tmp_grid[y as usize][x as usize] {
                                            match (other_spot, mode) {
                                                (BoardSpot::Black, Mode::Attacking) => {
                                                    att += 10;
                                                    continue;
                                                }
                                                (BoardSpot::Black, Mode::Defending) => {
                                                    def /= 2;
                                                    break;
                                                }
                                                (BoardSpot::White, Mode::Attacking) => {
                                                    att /= 2;
                                                    break;
                                                }
                                                (BoardSpot::White, Mode::Defending) => {
                                                    def += 10;
                                                    println!(
                                                        "!!!!x: {x} y: {y}\tatt: {att} def: {def}"
                                                    );
                                                    continue;
                                                }
                                            }
                                        }
                                    } else {
                                        def /= 2;
                                        att /= 2;
                                        x -= off_x;
                                        y -= off_y;
                                    }

                                    break;
                                }
                                // TODO: make it add to existing choosing cells
                                println!("x: {x} y: {y}\tatt: {att} def: {def}");
                                ai_grid[y as usize][x as usize] = ChoosingCell {
                                    attack: att,
                                    defense: def,
                                };
                            }
                        }
                    }
                }
            }
        }
    }

    #[derive(Default)]
    struct PosValue {
        x: usize,
        y: usize,
        val: u8,
    }

    let mut highest_att = PosValue::default();
    let mut highest_def = PosValue::default();

    for ai_y in 0..GRID_SIZE as usize {
        for ai_x in 0..GRID_SIZE as usize {
            let choosing_cell = ai_grid[ai_y][ai_x];

            if choosing_cell.attack > highest_att.val {
                highest_att = PosValue {
                    x: ai_x,
                    y: ai_y,
                    val: choosing_cell.attack,
                }
            }

            if choosing_cell.defense > highest_def.val {
                highest_def = PosValue {
                    x: ai_x,
                    y: ai_y,
                    val: choosing_cell.defense,
                }
            }
        }
    }
    println!("----------------");

    if highest_att.val >= 40 || highest_att.val > highest_def.val {
        return (highest_att.x, highest_att.y, ai_grid);
    } else if highest_def.val >= 40 || highest_def.val > highest_att.val {
        return (highest_def.x, highest_def.y, ai_grid);
    }

    (highest_att.x, highest_att.y, ai_grid)
}

fn is_inbound(x: i32, y: i32) -> bool {
    x >= 0 && x < GRID_SIZE as i32 && y >= 0 && y < GRID_SIZE as i32
}
