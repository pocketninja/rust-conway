// A naive implementation of Conway's Game of Life!

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::Paragraph,
};
use std::io::{stdout, Result, Stdout};

use std::io;
use rand::Rng;
use std::{thread, time};
use std::cmp::{max, min};
use ratatui::layout::{Rect};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders};
use ratatui::widgets::block::Title;

enum LoopAction {
    Continue,
    Quit,
    Restart,
    SlowDown,
    SpeedUp,
}

struct Vector {
    x: i32,
    y: i32,
}

impl Vector {
    fn out_of_bounds(&self, min: &Vector, max: &Vector) -> bool {
        self.x < min.x || self.y < min.y || self.x >= max.x || self.y >= max.y
    }
}

struct Cell {
    alive: bool,
    coordinate: Vector,
}

impl Cell {
    fn determine_next_state(&self, world: &World) -> bool {
        let mut living_neighbours = 0;

        for x in -1..=1 {
            for y in -1..=1 {
                if x == 0 && y == 0 {
                    continue;
                }

                let lookup_coordinate = Vector {
                    x: self.coordinate.x + x,
                    y: self.coordinate.y + y,
                };

                if lookup_coordinate.out_of_bounds(&WORLD_MIN, &world.size) {
                    continue;
                }

                if !world.cells[lookup_coordinate.x as usize][lookup_coordinate.y as usize].alive {
                    continue;
                }

                living_neighbours += 1;
            }
        }

        match (self.alive, living_neighbours) {
            (true, 2) | (true, 3) | (false, 3) => true,
            _ => false,
        }
    }
}

struct World {
    frames: u64,
    size: Vector,
    cells: Vec<Vec<Cell>>,
    changed: bool,
}

impl World {
    fn new(size: &Vector, life_chance: f64) -> World {
        let mut cells = Vec::new();

        for x in 0..size.x {
            let mut row = Vec::new();

            for y in 0..size.y {
                row.push(Cell {
                    coordinate: Vector { x, y },
                    alive: rand::thread_rng().gen_range(0.0..1.0) < life_chance,
                });
            }

            cells.push(row);
        }

        World {
            frames: 0,
            cells,
            size: Vector { x: size.x, y: size.y },
            changed: false,
        }
    }

    fn tick(&mut self) {
        let mut new_states = Vec::new();

        for x in 0..self.size.x {
            for y in 0..self.size.y {
                let cell = &self.cells[x as usize][y as usize];

                let next_state = cell.determine_next_state(self);

                if next_state == cell.alive {
                    continue;
                }

                new_states.push((
                    x as usize,
                    y as usize,
                    next_state
                ));
            }
        }

        let did_change = new_states.len() > 0;

        for (x, y, state) in new_states {
            self.cells[x][y].alive = state;
        }

        self.frames += 1;
        self.changed = did_change;
    }

    fn draw_world(&self) -> String {
        let mut result = "".to_string();

        for y in 0..self.size.y {
            for x in 0..self.size.x {
                result.push_str(
                    format!("{}", if self.cells[x as usize][y as usize].alive { "#" } else { " " }).as_str()
                );
            }
            result.push_str("\n");
        }

        return result;
    }
}

const WORLD_MIN: Vector = Vector { x: 0, y: 0 };

fn main() -> Result<()> {
    let world_size = ask_for_world_size();
    println!("World size: {}x{}", world_size.x, world_size.y);

    let mut terminal = setup_terminal()?;
    clear_terminal(&mut terminal)?;

    let mut world = World::new(&world_size, 0.5);

    let mut milliseconds = 10;
    let mut sleep_duration = time::Duration::from_millis(milliseconds);

    loop {
        world.tick();

        draw_ui(&mut terminal, &world, &milliseconds)?;

        let loop_action = request_loop_action()?;

        match loop_action {
            LoopAction::SlowDown => {
                milliseconds = milliseconds + 10;
                sleep_duration = time::Duration::from_millis(milliseconds);
            }
            LoopAction::SpeedUp => {
                milliseconds = max(10, milliseconds - 10);
                sleep_duration = time::Duration::from_millis(milliseconds);
            }
            LoopAction::Quit => break,
            LoopAction::Restart => {
                world = World::new(&world_size, 0.5);
            }
            LoopAction::Continue => {}
        }

        thread::sleep(sleep_duration);
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn draw_ui(terminal: &mut Terminal<CrosstermBackend<Stdout>>, world: &World, sleep_delay: &u64) -> Result<()> {
    terminal.draw(|frame| {
        let frame_rect = frame.size();

        let info_rect = Rect::new(
            0,
            0,
            frame_rect.width,
            3,
        );
        let world_rect = Rect::new(
            0,
            3,
            min(world.size.x as u16, frame_rect.width),
            min(frame_rect.height - info_rect.height, frame_rect.height),
        );

        let info_block = Block::default()
            .title(Title::from("Rust Conway".bold()))
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let world_block = Block::default()
            .title("World")
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let info = format!(
            "{}uit / {}estart / {} slow down / {} speed up // {} // {}ms // Frame: {}",
            "[q]".bold().underlined(),
            "[r]".bold().underlined(),
            "[-]".bold().underlined(),
            "[+]".bold().underlined(),
            if world.changed { "Generating" } else { "Stable" },
            sleep_delay,
            world.frames
        );

        let info_paragraph = Paragraph::new(info)
            .white().on_blue()
            .block(info_block);

        let world_paragaph = Paragraph::new(world.draw_world())
            .white().on_black()
            .block(world_block);

        frame.render_widget(info_paragraph, info_rect);
        frame.render_widget(world_paragaph, world_rect);
    })?;
    Ok(())
}

fn request_loop_action() -> Result<LoopAction> {
    if event::poll(std::time::Duration::from_millis(1))? {
        if let event::Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(LoopAction::Continue);
            }

            return match key.code {
                KeyCode::Char('q') => Ok(LoopAction::Quit),
                KeyCode::Char('r') => Ok(LoopAction::Restart),
                KeyCode::Char('-') => Ok(LoopAction::SlowDown),
                KeyCode::Char('+') => Ok(LoopAction::SpeedUp),
                KeyCode::Char('=') => Ok(LoopAction::SpeedUp),
                _ => Ok(LoopAction::Continue),
            };
        }
    }

    // Continue...
    Ok(LoopAction::Continue)
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn clear_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    terminal.clear()?;
    Ok(())
}

fn ask_for_world_size() -> Vector {
    let mut world_size = Vector { x: 0, y: 0 };

    let mut coordinate_values: Vec<i32> = vec![0, 0];

    for i in 0..coordinate_values.len() {
        loop {
            let axis_label = match i {
                0 => "width",
                1 => "height",
                _ => panic!("Invalid axis label"),
            };

            println!("Enter the {} of the world: ", axis_label);

            let mut input = String::new();

            io::stdin().read_line(&mut input)
                .expect(&format!("Failed to read the {} of the world", axis_label));

            let value: i32 = match input.trim().parse() {
                Ok(value) => value,
                Err(_) => continue,
            };

            match value <= 1 {
                true => continue,
                _ => {
                    coordinate_values[i] = value;
                    break;
                }
            }
        }
    }

    world_size.x = coordinate_values[0];
    world_size.y = coordinate_values[1];

    world_size
}
