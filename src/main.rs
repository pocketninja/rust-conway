// A naive implementation of Conway's Game of Life!

use std::io;
use rand::Rng;
use std::{thread, time};


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
}

impl World {
    fn new(size: Vector, life_chance: f64) -> World {
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

        World { frames: 0, cells, size }
    }

    fn tick(&mut self) -> bool {
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

        did_change
    }

    fn draw_world(&self) {
        let border = format!("+{}+", "-".repeat(self.size.x as usize));
        println!("{}", border);

        for y in 0..self.size.y {
            print!("|");
            for x in 0..self.size.x {
                print!("{}", if self.cells[x as usize][y as usize].alive { "#" } else { " " });
            }
            print!("|");
            println!();
        }

        println!("{}", border);
    }
}

const WORLD_MIN: Vector = Vector { x: 0, y: 0 };

fn main() {
    clear_screen();

    let world_size = ask_for_world_size();

    println!("World size: {}x{}", world_size.x, world_size.y);

    let mut world = World::new(world_size, 0.2);

    let sleep_duration = time::Duration::from_millis(10);

    loop {
        clear_screen();

        let world_changed = world.tick();
        world.draw_world();

        println!("Frame: {}", world.frames);

        if !world_changed {
            println!("World has reached a stable state");
            break;
        }

        thread::sleep(sleep_duration);
    }
}

fn clear_screen() {
    // print!("\x1B[2J\x1B[1;1H");
    clearscreen::clear().expect("failed to clear screen");
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
