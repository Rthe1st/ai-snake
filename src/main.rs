use std::{
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
    food: (u16, u16),
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
        Self {
            snake: vec![(2, 2)],
            food: (5, 5),
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
        
        // Generate food position that's not on the snake
        loop {
            let new_food = (
                rng.gen_range(1..max_x),
                rng.gen_range(1..max_y),
            );
            
            if !self.snake.contains(&new_food) {
                self.food = new_food;
                break;
            }
        }
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
        
        // Check if snake ate the food
        if new_head == self.food {
            // Don't remove the tail to make the snake grow
            self.generate_food();
        } else {
            // Remove tail only if food wasn't eaten
            self.snake.pop();
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

            // Draw food
            let food_cell = Paragraph::new("●");
            frame.render_widget(
                food_cell,
                Rect::new(game.food.0, game.food.1, 1, 1),
            );

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
