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
use rand::SeedableRng;

struct Game {
    snake: Vec<(u16, u16)>,
    food: HashSet<(u16, u16)>,
    direction: Direction,
    game_over: bool,
    frame_size: (u16, u16),  // (width, height)
    rng: rand::rngs::StdRng,
}

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Game {
    fn new(width: u16, height: u16, seed: u64) -> Self {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        
        // Generate random starting position
        let start_x = rng.gen_range(0..width);
        let start_y = rng.gen_range(0..height);
        
        let mut game = Self {
            snake: vec![(start_x, start_y)],
            food: HashSet::new(),
            direction: Direction::Right,
            game_over: false,
            frame_size: (width, height),
            rng,
        };

        // Initialize food items
        while game.food.len() < 10 {
            game.generate_food();
        }

        game
    }

    fn generate_food(&mut self) {
        let (width, height) = self.frame_size;
        
        // Try to find a valid position that's not on the snake
        let mut attempts = 0;
        while attempts < 100 { // Limit attempts to prevent infinite loop
            let x = self.rng.gen_range(0..width);
            let y = self.rng.gen_range(0..height);
            let pos = (x, y);
            
            // Check if position is valid (not on snake or existing food)
            if !self.snake.contains(&pos) && !self.food.contains(&pos) {
                self.food.insert(pos);
                return;
            }
            attempts += 1;
        }
        
        // If we couldn't find a position after max attempts, try systematic approach
        for x in 0..width {
            for y in 0..height {
                let pos = (x, y);
                if !self.snake.contains(&pos) && !self.food.contains(&pos) {
                    self.food.insert(pos);
                    return;
                }
            }
        }
    }

    fn update(&mut self) {
        if self.game_over {
            return;
        }

        let (head_x, head_y) = self.snake[0];
        let (width, height) = self.frame_size;

        let new_head = match self.direction {
            Direction::Up => (head_x, head_y.saturating_sub(1)),
            Direction::Down => (head_x, if head_y >= height - 1 { height - 1 } else { head_y + 1 }),
            Direction::Left => (head_x.saturating_sub(1), head_y),
            Direction::Right => (if head_x >= width - 1 { width - 1 } else { head_x + 1 }, head_y),
        };

        // Check if snake hit the boundaries or itself
        if new_head.0 >= width || new_head.1 >= height || self.snake.contains(&new_head) {
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
        
        while self.food.len() < 10 {
            self.generate_food();
        }
    }
}

fn main() -> Result<()> {
    // Terminal initialization
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    // Get initial terminal size and account for borders
    let size = terminal.size()?;
    let game_width = size.width.saturating_sub(4);  // Subtract 4 to account for borders
    let game_height = size.height.saturating_sub(4);
    
    // Game state initialization with frame size
    let mut game = Game::new(game_width, game_height, rand::thread_rng().gen());
    let mut last_update = Instant::now();
    let tick_rate = Duration::from_millis(100);

    loop {
        terminal.draw(|frame| {
            let size = frame.size();
            
            // Draw game area
            let block = Block::default()
                .title("Snake Game")
                .borders(Borders::ALL);
            frame.render_widget(block, size);

            // Draw snake (offset by 2 to account for borders)
            for &(x, y) in &game.snake {
                let snake_cell = Paragraph::new("█");
                frame.render_widget(
                    snake_cell,
                    Rect::new(x + 2, y + 2, 1, 1),
                );
            }

            // Draw all food items (offset by 2 to account for borders)
            for &(x, y) in &game.food {
                let food_cell = Paragraph::new("●");
                frame.render_widget(
                    food_cell,
                    Rect::new(x + 2, y + 2, 1, 1),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_food_within_bounds() {
        let mut game = Game::new(10, 10, 42); // Create a 20x20 game area
        game.food.clear(); // Clear initial food
        
        // Generate and test 50 food positions to ensure they're all within bounds
        for _ in 0..50 {
            game.generate_food();
            let food = game.food.iter().next().unwrap();
            
            // Food should be within playable area (accounting for borders)
            assert!(food.0 <= 10, "Food x position {} should be within bounds 1..18", food.0);
            assert!(food.1 <= 10, "Food y position {} should be within bounds 1..18", food.1);
            
            game.food.clear(); // Clear for next iteration
        }
    }

    #[test]
    fn test_generate_food_not_on_snake() {
        let mut game = Game::new(7, 7, 42);
        game.food.clear();
        
        // Create a snake with multiple segments
        game.snake = vec![
            (5, 5),
            (5, 6),
            (5, 7),
            (6, 7),
            (7, 7),
        ];
        
        // Calculate total possible positions
        let width = game.frame_size.0;
        let height = game.frame_size.1;
        let total_positions = (width * height) as usize;
        let available_positions = total_positions - game.snake.len();
        
        // Generate food until we fill all available positions (plus 1 to make sure there's no error even when board is full)
        while game.food.len() < available_positions {
            game.generate_food();
        }

        // Food should not be on any snake segment
        for food in &game.food {
            assert!(!game.snake.contains(food), "Food {:?} should not be on snake {:?}", food, game.snake);
        }
    }
}
