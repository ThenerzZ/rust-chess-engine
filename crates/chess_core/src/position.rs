#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub rank: u8,  // 1-8
    pub file: u8,  // a-h (1-8)
}

impl Position {
    pub fn new(file: u8, rank: u8) -> Option<Self> {
        if file <= 8 && rank <= 8 {
            Some(Self { file, rank })
        } else {
            None
        }
    }

    pub fn from_algebraic(notation: &str) -> Option<Self> {
        if notation.len() != 2 {
            return None;
        }
        
        let file = notation.chars().next().unwrap();
        let rank = notation.chars().nth(1).unwrap();
        
        if !('a'..='h').contains(&file) || !('1'..='8').contains(&rank) {
            return None;
        }

        Some(Self {
            file: (file as u8) - b'a' + 1,
            rank: (rank as u8) - b'0',
        })
    }
} 