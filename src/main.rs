use std::{
    collections::HashSet,
    io,
    time::{Duration, Instant},
};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rand::Rng;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

struct Game {
    snake: Vec<(u16, u16)>,
    food: HashSet<(u16, u16)>,
    direction: Direction,
    game_over: bool,
    frame_size: (u16, u16),  // (width, height)
}

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Game {
    fn new() -> Self {
        let mut food = HashSet::new();

        Self {
            snake: vec![(2, 2)],
            food,
            direction: Direction::Right,
            game_over: false,
            frame_size: (0, 0),  // Will be updated when game starts
        }
    }

    fn generate_food(&mut self) {
        let mut rng = rand::thread_rng();
        let (width, height) = self.frame_size;
        
        // Account for borders and ensure food is within playable area
        let max_x = width.saturating_sub(2);
        let max_y = height.saturating_sub(2);
        
        // Calculate total empty space in frame
        let total_space = (max_x - 1) * (max_y - 1);
        let empty_space = total_space - self.snake.len() as u16;
        
        // Generate random position in empty space
        let food_location = rng.gen_range(1..=empty_space);
        
        // Map food_location to x,y coordinates using modular arithmetic
        // TODO: its possible to place food on top of the snake or other food
        // Need to exclude snake/food locations from final mapping
        let x = (food_location % max_x).max(1);
        let y = (food_location / max_x).max(1);
        
        self.food.insert((x, y));
    }

    fn update(&mut self) {
        if self.game_over {
            return;
        }

        let (head_x, head_y) = self.snake[0];
        let (width, height) = self.frame_size;
        
        // Account for borders by subtracting 1 from boundaries
        let max_x = width.saturating_sub(2);
        let max_y = height.saturating_sub(2);

        let new_head = match self.direction {
            Direction::Up => (head_x, head_y.saturating_sub(1)),
            Direction::Down => (head_x, if head_y >= max_y { max_y } else { head_y + 1 }),
            Direction::Left => (head_x.saturating_sub(1), head_y),
            Direction::Right => (if head_x >= max_x { max_x } else { head_x + 1 }, head_y),
        };

        // Check if snake hit the boundaries
        if new_head.0 == 0 || new_head.0 >= max_x || new_head.1 == 0 || new_head.1 >= max_y {
            self.game_over = true;
            return;
        }

        self.snake.insert(0, new_head);
        
        // Check if snake ate any food
        if self.food.remove(&new_head) {
            // Don't remove the tail to make the snake grow
            self.generate_food(); // Generate new food when one is eaten
        } else {
            // Remove tail only if food wasn't eaten
            self.snake.pop();
        }
        
        // TODO: as we only remove food when eaten, we only need to to this at init time
        // But that requires changing the game init to be aware of frame size.
        if self.food.len() < 3 {
            self.generate_food();
        }
        
    }

    fn update_frame_size(&mut self, width: u16, height: u16) {
        self.frame_size = (width, height);
    }
}

fn main() -> Result<()> {
    // Terminal initialization
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    // Game state
    let mut game = Game::new();
    let mut last_update = Instant::now();
    let tick_rate = Duration::from_millis(100);

    loop {
        terminal.draw(|frame| {
            let size = frame.size();
            game.update_frame_size(size.width, size.height);
            
            // Draw game area
            let block = Block::default()
                .title("Snake Game")
                .borders(Borders::ALL);
            frame.render_widget(block, size);

            // Draw snake
            for &(x, y) in &game.snake {
                let snake_cell = Paragraph::new("█");
                frame.render_widget(
                    snake_cell,
                    Rect::new(x, y, 1, 1),
                );
            }

            // Draw all food items
            for &(x, y) in &game.food {
                let food_cell = Paragraph::new("●");
                frame.render_widget(
                    food_cell,
                    Rect::new(x, y, 1, 1),
                );
            }

            // Draw game over message if needed
            if game.game_over {
                let game_over = Paragraph::new("Game Over! Press 'q' to quit")
                    .alignment(Alignment::Center);
                frame.render_widget(
                    game_over,
                    Rect::new(
                        size.width / 4,
                        size.height / 2,
                        size.width / 2,
                        3,
                    ),
                );
            }
        })?;

        // Input handling
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => {
                        if game.direction != Direction::Down {
                            game.direction = Direction::Up;
                        }
                    }
                    KeyCode::Down => {
                        if game.direction != Direction::Up {
                            game.direction = Direction::Down;
                        }
                    }
                    KeyCode::Left => {
                        if game.direction != Direction::Right {
                            game.direction = Direction::Left;
                        }
                    }
                    KeyCode::Right => {
                        if game.direction != Direction::Left {
                            game.direction = Direction::Right;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Update game state
        if last_update.elapsed() >= tick_rate {
            game.update();
            last_update = Instant::now();
        }
    }

    // Cleanup
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
