use std::collections::HashMap;
use chess_core::{moves::Move, Board, piece::Color, position::Position};

#[derive(Clone)]
pub struct OpeningBook {
    positions: HashMap<String, Vec<BookMove>>,
}

#[derive(Clone)]
struct BookMove {
    mv: Move,
    weight: u32,  // Higher weight means more likely to be played
}

impl OpeningBook {
    pub fn new() -> Self {
        let mut book = Self {
            positions: HashMap::new(),
        };
        book.initialize_common_openings();
        book
    }

    fn initialize_common_openings(&mut self) {
        let mut board = Board::new();
        
        // 1. e4 lines
        let e4_move = Move::new(
            Position { rank: 2, file: 5 },
            Position { rank: 4, file: 5 }
        );
        self.add_line(&board, e4_move, 100);  // King's Pawn Opening
        
        // 1...e5 (Open Game)
        let mut e4_board = board.clone();
        e4_board.make_move(e4_move).unwrap();
        let e5_move = Move::new(
            Position { rank: 7, file: 5 },
            Position { rank: 5, file: 5 }
        );
        self.add_line(&e4_board, e5_move, 100);  // 1...e5
        
        // After 1. e4 e5, add main responses
        let mut open_game_board = e4_board.clone();
        open_game_board.make_move(e5_move).unwrap();
        
        // 2. Nf3 (Ruy Lopez/Italian Game setup)
        let nf3_move = Move::new(
            Position { rank: 1, file: 7 },
            Position { rank: 3, file: 6 }
        );
        self.add_line(&open_game_board, nf3_move, 100);  // 2. Nf3
        
        // After 2. Nf3, add 2...Nc6
        let mut ruy_board = open_game_board.clone();
        ruy_board.make_move(nf3_move).unwrap();
        let nc6_move = Move::new(
            Position { rank: 8, file: 2 },
            Position { rank: 6, file: 3 }
        );
        self.add_line(&ruy_board, nc6_move, 100);  // 2...Nc6
        
        // After 2...Nc6, add main variations
        let mut nc6_board = ruy_board.clone();
        nc6_board.make_move(nc6_move).unwrap();
        
        // 3. Bb5 (Ruy Lopez)
        self.add_line(&nc6_board, Move::new(
            Position { rank: 1, file: 6 },
            Position { rank: 5, file: 2 }
        ), 100);  // 3. Bb5
        
        // 3. Bc4 (Italian Game)
        self.add_line(&nc6_board, Move::new(
            Position { rank: 1, file: 6 },
            Position { rank: 4, file: 3 }
        ), 80);  // 3. Bc4
        
        // 1...c5 (Sicilian Defense)
        let c5_move = Move::new(
            Position { rank: 7, file: 3 },
            Position { rank: 5, file: 3 }
        );
        self.add_line(&e4_board, c5_move, 90);  // 1...c5
        
        // After 1. e4 c5, add main responses
        let mut sicilian_board = e4_board.clone();
        sicilian_board.make_move(c5_move).unwrap();
        
        // 2. Nf3 (Open Sicilian)
        let nf3_sicilian = Move::new(
            Position { rank: 1, file: 7 },
            Position { rank: 3, file: 6 }
        );
        self.add_line(&sicilian_board, nf3_sicilian, 100);  // 2. Nf3
        
        // After 2. Nf3, add main responses
        let mut open_sicilian = sicilian_board.clone();
        open_sicilian.make_move(nf3_sicilian).unwrap();
        
        // 2...d6 (Najdorf setup)
        self.add_line(&open_sicilian, Move::new(
            Position { rank: 7, file: 4 },
            Position { rank: 6, file: 4 }
        ), 90);  // 2...d6
        
        // 2...Nc6 (Classical Sicilian setup)
        self.add_line(&open_sicilian, Move::new(
            Position { rank: 8, file: 2 },
            Position { rank: 6, file: 3 }
        ), 80);  // 2...Nc6
        
        // 1. d4 lines
        let d4_move = Move::new(
            Position { rank: 2, file: 4 },
            Position { rank: 4, file: 4 }
        );
        self.add_line(&board, d4_move, 90);  // Queen's Pawn Opening
        
        // 1...d5 (Closed Game)
        let mut d4_board = board.clone();
        d4_board.make_move(d4_move).unwrap();
        let d5_move = Move::new(
            Position { rank: 7, file: 4 },
            Position { rank: 5, file: 4 }
        );
        self.add_line(&d4_board, d5_move, 100);  // 1...d5
        
        // After 1. d4 d5, add Queen's Gambit lines
        let mut qg_board = d4_board.clone();
        qg_board.make_move(d5_move).unwrap();
        
        // 2. c4 (Queen's Gambit)
        let c4_move = Move::new(
            Position { rank: 2, file: 3 },
            Position { rank: 4, file: 3 }
        );
        self.add_line(&qg_board, c4_move, 100);  // 2. c4
        
        // After 2. c4, add main responses
        let mut qg_offered = qg_board.clone();
        qg_offered.make_move(c4_move).unwrap();
        
        // 2...e6 (Queen's Gambit Declined)
        self.add_line(&qg_offered, Move::new(
            Position { rank: 7, file: 5 },
            Position { rank: 6, file: 5 }
        ), 90);  // 2...e6
        
        // 2...dxc4 (Queen's Gambit Accepted)
        self.add_line(&qg_offered, Move::new(
            Position { rank: 5, file: 4 },
            Position { rank: 4, file: 3 }
        ), 70);  // 2...dxc4
        
        // 1...Nf6 (Indian Defense)
        let nf6_move = Move::new(
            Position { rank: 8, file: 7 },
            Position { rank: 6, file: 6 }
        );
        self.add_line(&d4_board, nf6_move, 90);  // 1...Nf6
        
        // After 1. d4 Nf6, add responses
        let mut indian_board = d4_board.clone();
        indian_board.make_move(nf6_move).unwrap();
        
        // 2. c4 (King's Indian setup)
        self.add_line(&indian_board, Move::new(
            Position { rank: 2, file: 3 },
            Position { rank: 4, file: 3 }
        ), 90);  // 2. c4
        
        // Alternative first moves
        // Reti Opening
        self.add_line(&board, Move::new(
            Position { rank: 1, file: 7 },
            Position { rank: 3, file: 6 }
        ), 60);  // 1. Nf3
        
        // English Opening
        self.add_line(&board, Move::new(
            Position { rank: 2, file: 3 },
            Position { rank: 4, file: 3 }
        ), 50);  // 1. c4
    }

    pub fn get_book_move(&self, board: &Board) -> Option<Move> {
        let position_key = self.get_position_key(board);
        self.positions.get(&position_key).and_then(|moves| {
            // Choose a move based on weights
            if moves.is_empty() {
                return None;
            }
            
            let total_weight: u32 = moves.iter().map(|m| m.weight).sum();
            let mut chosen_weight = rand::random::<u32>() % total_weight;
            
            for book_move in moves {
                if chosen_weight < book_move.weight {
                    return Some(book_move.mv);
                }
                chosen_weight -= book_move.weight;
            }
            
            Some(moves[0].mv)  // Fallback to first move
        })
    }

    pub fn add_line(&mut self, board: &Board, mv: Move, weight: u32) {
        let position_key = self.get_position_key(board);
        self.positions
            .entry(position_key)
            .or_insert_with(Vec::new)
            .push(BookMove { mv, weight });
    }

    // Generate a unique key for the board position
    fn get_position_key(&self, board: &Board) -> String {
        let mut key = String::new();
        
        // Add pieces to key
        for rank in 1..=8 {
            for file in 1..=8 {
                let pos = Position { rank, file };
                if let Some(piece) = board.get_piece(pos) {
                    let color_char = match piece.color {
                        Color::White => 'w',
                        Color::Black => 'b',
                    };
                    let piece_char = match piece.piece_type {
                        chess_core::piece::PieceType::Pawn => 'p',
                        chess_core::piece::PieceType::Knight => 'n',
                        chess_core::piece::PieceType::Bishop => 'b',
                        chess_core::piece::PieceType::Rook => 'r',
                        chess_core::piece::PieceType::Queen => 'q',
                        chess_core::piece::PieceType::King => 'k',
                    };
                    key.push_str(&format!("{}{}:{}{},", pos.rank, pos.file, color_char, piece_char));
                }
            }
        }
        
        // Add current turn
        key.push_str(&format!("turn:{}", if board.current_turn() == Color::White { "w" } else { "b" }));
        
        key
    }
} 