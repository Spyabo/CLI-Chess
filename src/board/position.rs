use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PositionError {
    #[error("Invalid position: {0}")]
    InvalidPosition(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i8,
    pub y: i8,
}

impl Position {
    pub fn new(x: i8, y: i8) -> Option<Self> {
        if (0..8).contains(&x) && (0..8).contains(&y) {
            Some(Self { x, y })
        } else {
            None
        }
    }
    
    pub fn file(&self) -> i8 {
        self.x
    }
    
    pub fn rank(&self) -> i8 {
        self.y
    }

    pub fn is_valid(&self) -> bool {
        (0..8).contains(&self.x) && (0..8).contains(&self.y)
    }
    
    pub fn from_xy(x: i8, y: i8) -> Option<Self> {
        if (0..8).contains(&x) && (0..8).contains(&y) {
            Some(Self { x, y })
        } else {
            None
        }
    }
    
    pub fn is_valid_file(file: i8) -> bool {
        (0..8).contains(&file)
    }
    
    pub fn is_valid_rank(rank: i8) -> bool {
        (0..8).contains(&rank)
    }

    pub fn from_notation(notation: &str) -> Result<Self, PositionError> {
        if notation.len() != 2 {
            return Err(PositionError::InvalidPosition(
                "Notation must be 2 characters long".to_string(),
            ));
        }

        let mut chars = notation.chars();
        let file = chars.next().unwrap().to_ascii_lowercase();
        let rank = chars.next().unwrap();

        if !('a'..='h').contains(&file) || !('1'..='8').contains(&rank) {
            return Err(PositionError::InvalidPosition(
                "Invalid file or rank".to_string(),
            ));
        }

        let x = (file as i8) - ('a' as i8);
        let y = (rank as i8) - ('1' as i8);

        match Position::new(x, y) {
            Some(pos) => Ok(pos),
            None => Err(PositionError::InvalidPosition("Position out of bounds".to_string())),
        }
    }

    pub fn to_notation(&self) -> String {
        if !self.is_valid() {
            return "-".to_string();
        }
        let file = (b'a' + self.x as u8) as char;
        let rank = (b'1' + self.y as u8) as char;
        format!("{}{}", file, rank)
    }
}

impl From<(i8, i8)> for Position {
    fn from((x, y): (i8, i8)) -> Self {
        Self { x, y }
    }
}

impl FromStr for Position {
    type Err = PositionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_notation(s)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_notation())
    }
}

impl Position {
    pub fn distance(&self, other: &Position) -> (i8, i8) {
        (other.x - self.x, other.y - self.y)
    }
    
    pub fn is_adjacent(&self, other: &Position) -> bool {
        let (dx, dy) = self.distance(other);
        dx.abs() <= 1 && dy.abs() <= 1 && (dx != 0 || dy != 0)
    }
    
    pub fn is_straight_line(&self, other: &Position) -> bool {
        self.x == other.x || self.y == other.y
    }
    
    pub fn is_diagonal(&self, other: &Position) -> bool {
        let (dx, dy) = self.distance(other);
        dx.abs() == dy.abs() && dx != 0
    }
    
    pub fn squares_between(&self, other: &Position) -> Vec<Position> {
        let mut squares = Vec::new();
        let (dx, dy) = (other.x - self.x, other.y - self.y);
        
        if dx == 0 && dy == 0 {
            return squares;
        }
        
        let step_x = dx.signum();
        let step_y = dy.signum();
        
        let mut current = *self;
        current.x += step_x;
        current.y += step_y;
        
        while current != *other {
            squares.push(current);
            current.x += step_x;
            current.y += step_y;
        }
        
        squares
    }
}

impl Add<(i8, i8)> for Position {
    type Output = Self;

    fn add(self, (dx, dy): (i8, i8)) -> Self::Output {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

impl AddAssign<(i8, i8)> for Position {
    fn add_assign(&mut self, (dx, dy): (i8, i8)) {
        self.x += dx;
        self.y += dy;
    }
}

impl Sub<(i8, i8)> for Position {
    type Output = Self;

    fn sub(self, (dx, dy): (i8, i8)) -> Self::Output {
        Self {
            x: self.x - dx,
            y: self.y - dy,
        }
    }
}

impl SubAssign<(i8, i8)> for Position {
    fn sub_assign(&mut self, (dx, dy): (i8, i8)) {
        self.x -= dx;
        self.y -= dy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(0, 0).unwrap();
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 0);
        assert!(pos.is_valid());
    }

    #[test]
    fn test_position_notation() {
        let pos = Position::from_notation("a1").unwrap();
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 0);
        assert_eq!(pos.to_notation(), "a1");

        let pos = Position::from_notation("h8").unwrap();
        assert_eq!(pos.x, 7);
        assert_eq!(pos.y, 7);
        assert_eq!(pos.to_notation(), "h8");
    }

    #[test]
    fn test_invalid_notation() {
        assert!(Position::from_notation("i1").is_err());
        assert!(Position::from_notation("a9").is_err());
        assert!(Position::from_notation("a").is_err());
        assert!(Position::from_notation("a11").is_err());
    }
}
