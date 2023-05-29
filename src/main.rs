use sfml::{
    graphics::{PrimitiveType, Vertex, VertexBuffer, VertexBufferUsage},
    system::{Vector2, Vector2u},
    window::mouse,
};

use {
    rand::{thread_rng, Rng},
    sfml::{
        audio::{Sound, SoundBuffer, SoundSource},
        graphics::{
            CircleShape, Color, Font, RectangleShape, RenderTarget, RenderWindow, Shape, Text,
            Transformable,
        },
        system::{Clock, Time, Vector2f},
        window::{ContextSettings, Event, Key, Style},
    },
    std::{env, f32::consts::PI},
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
            let (x, y) = ai_choose(board.clone());
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

struct ChoosingCell {
    attack: u8,
    defense: u8,
}

fn ai_choose(board: [[Option<BoardSpot>; 20]; 20]) -> (usize, usize) {
    (0, 0)
}
