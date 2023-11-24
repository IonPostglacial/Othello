use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Player { Black, White }

impl Player {
    fn opponent(self) -> Player {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }

    fn to_char(self) -> char {
        match self {
            Player::Black => 'X',
            Player::White => 'O',
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Direction { pub dx: i8, pub dy: i8 }

#[derive(Debug, Clone, Copy)]
struct Position { pub x: i8, pub y: i8 }

impl Position {
    fn neighbor(self, d: Direction) -> Position {
        Position { x: self.x.saturating_add(d.dx), y: self.y.saturating_add(d.dy) }
    }

    fn is_valid(self) -> bool {
        self.x >= 0 && self.x < 8 && self.y >= 0 && self.y < 8
    }
}

enum Command {
    PlayAt(Position),
    Pass,
    Victory(Option<Player>),
}

static MOVES: [&str; 64] = [
    "a1\n", "b1\n", "c1\n", "d1\n", "e1\n", "f1\n", "g1\n", "h1\n",
    "a2\n", "b2\n", "c2\n", "d2\n", "e2\n", "f2\n", "g2\n", "h2\n",
    "a3\n", "b3\n", "c3\n", "d3\n", "e3\n", "f3\n", "g3\n", "h3\n",
    "a4\n", "b4\n", "c4\n", "d4\n", "e4\n", "f4\n", "g4\n", "h4\n",
    "a5\n", "b5\n", "c5\n", "d5\n", "e5\n", "f5\n", "g5\n", "h5\n",
    "a6\n", "b6\n", "c6\n", "d6\n", "e6\n", "f6\n", "g6\n", "h6\n",
    "a7\n", "b7\n", "c7\n", "d7\n", "e7\n", "f7\n", "g7\n", "h7\n",
    "a8\n", "b8\n", "c8\n", "d8\n", "e8\n", "f8\n", "g8\n", "h8\n",
];

impl Command {
    fn parse(cmd: &str) -> Result<Command, ()> {
        match cmd {
            "black\n" => Ok(Command::Victory(Some(Player::Black))),
            "white\n" => Ok(Command::Victory(Some(Player::White))),
            "draw\n" => Ok(Command::Victory(None)),
            "pass\n" => Ok(Command::Pass),
            _ => {
                match cmd.as_bytes() {
                    [x, y, 10] => {
                        let pos = Position { x: (x - b'a') as i8, y: (y - b'1') as i8 };
                        if pos.is_valid() {
                            Ok(Command::PlayAt(pos))
                        } else {
                            Err(())
                        }
                    },
                    _ => Err(())
                }
            }
        }
    }

    fn stringify(cmd: &Command) -> &str {
        match cmd {
            Command::PlayAt(pos @ Position { x, y }) => {
                if !pos.is_valid() { return "pass\n" }
                MOVES[(x + 8 * y) as usize]
            }
            Command::Pass => "pass\n",
            Command::Victory(winner) => {
                match winner {
                    Some(Player::Black) => "black\n",
                    Some(Player::White) => "white\n",
                    None => "draw\n",
                }
            }
        }
    }
}

#[derive(Clone)]
struct Board {
    cells: [[u8; 8]; 2],
}

static PLAY_DIRECTIONS: [Direction; 8] = [
    Direction {dx: -1, dy: -1}, Direction {dx: -1, dy: 0}, Direction {dx: -1, dy: 1}, 
    Direction {dx: 0, dy: -1}, Direction {dx: 0, dy: 1},
    Direction {dx: 1, dy: -1}, Direction {dx: 1, dy: 0}, Direction {dx: 1, dy: 1}];

impl Board {
    fn new() -> Board {
        Board { 
            cells: [
                [
                    0b00000000,
                    0b00000000,
                    0b00000000,
                    0b00010000,
                    0b00001000,
                    0b00000000,
                    0b00000000,
                    0b00000000,
                ],
                [
                    0b00000000,
                    0b00000000,
                    0b00000000,
                    0b00001000,
                    0b00010000,
                    0b00000000,
                    0b00000000,
                    0b00000000,
                ],
            ]
        }
    }

    fn player_at(&self, pos: Position) -> Option<Player> {
        let x_mask = 1 << pos.x;
        if self.cells[Player::Black as usize][pos.y as usize] & x_mask != 0 {
            Some(Player::Black)
        } else if self.cells[Player::White as usize][pos.y as usize] & x_mask != 0 {
            Some(Player::White)
        } else {
            None
        }
    }

    fn player_score(&self, player: Player) -> i64 {
        self.cells[player as usize].iter().map(|byte| byte.count_ones()).sum::<u32>() as i64
    }

    fn heuristic(&self, player: Player) -> i64 {
        self.player_score(player) - self.player_score(player.opponent())
    }

    fn find_bridge_candidate<'a>(&self, bridge: &'a mut [Position; 8], p: Position, d: Direction, player: Player, played: bool) -> &'a [Position] {
        let mut length = 1usize;
        if !p.is_valid() || (!played && self.player_at(p) != None) {
            return &bridge[0..0]
        }
        let mut current_pos = p.neighbor(d);
        bridge[0] = current_pos;
    
        while current_pos.is_valid() && self.player_at(current_pos).map_or(false, |o| o == player.opponent()) {
            current_pos = current_pos.neighbor(d);
            bridge[length] = current_pos;
            length += 1
        }
        if current_pos.is_valid() && self.player_at(current_pos).map_or(false, |o| o == player) && length > 1 {
            return &bridge[0..length]
        } else {
            &bridge[0..0]
        }
    }

    fn set_cell(&mut self, p: Position, player: Player) {
        self.cells[player as usize][p.y as usize] |= 1 << p.x;
        self.cells[player.opponent() as usize][p.y as usize] &= !(1 << p.x);
    }

    fn play_at(&mut self, p: Position, player: Player) -> bool {
        let mut played = false;
        let mut buffer = [Position{x: 0, y: 0}; 8];
    
        for dir in PLAY_DIRECTIONS.iter() {
            let bridge = self.find_bridge_candidate(&mut buffer, p, *dir, player, played);
            if bridge.len() > 0 {
                played = true;
                self.set_cell(p, player);
                for position in bridge.iter() {
                    self.set_cell(*position, player);
                }
            }
        }
        return played
    }
}

fn negamax_ab(board: &Board, depth: usize, alpha: i64, beta: i64, player: Player) -> i64 {
	if depth == 0 {
		return board.heuristic(player)
	}
	let mut alpha = alpha;
    let mut terminal_node = true;
	let mut score: i64 = i32::MIN as i64;
'forEachNodes:
	for y in 0..8 {
		for x in 0..8 {
			let mut child = board.clone();
			if child.play_at(Position{x, y}, player) {
				terminal_node = false;
				score = std::cmp::max(score, -negamax_ab(&child, depth-1, -beta, -alpha, player.opponent()));
				alpha = std::cmp::max(alpha, score);
				if alpha >= beta {
					break 'forEachNodes
				}
			}
		}
	}
	if terminal_node {
		score = board.heuristic(player)
	}
	return score
}

fn negamax(board: &Board, depth: usize, player: Player) -> i64 {
	negamax_ab(board, depth, i32::MIN as i64, i32::MAX as i64, player)
}

fn draw_board(board: &Board) {
    let mut buf = "  a b c d e f g h\n".to_string();
    for y in 0..8 {
        buf.push((b'1'.saturating_add_signed(y)).into());
        buf.push(' ');
        for x in 0..8 {
            match board.player_at(Position { x, y }) {
                Some(player) => {
                    buf.push(player.to_char())
                }
                None => buf.push('.')
            }
            buf.push(' ')
        }
        writeln!(buf, "").expect("couldn't write to board buffer")
    }
    println!("{buf}")
}

fn arg_to_player(arg: &str) -> Result<Player, ()> {
    match arg {
        "black" => Ok(Player::Black),
        "white" => Ok(Player::White),
        _=> Err(())
    }
}

fn human_play(board: &mut Board, player: Player, input: &mut String) -> bool {
    input.clear();
    println!("{}?", player.to_char());
    std::io::stdin().read_line(input).expect("invalid string");
    let cmd = Command::parse(&input);
    match cmd {
        Err(()) => {
            println!("invalid command '{input}'");
            true
        }
        Ok(Command::Victory(winner)) => {
            match winner {
                Some(Player::Black) => println!("black won"),
                Some(Player::White) => println!("white won"),
                None => println!("it's a draw"),
            }
            true
        },
        Ok(Command::Pass) => false,
        Ok(Command::PlayAt(pos)) => {
            !board.play_at(pos, player)
        }
    }
}

fn machine_play(board: &mut Board, player: Player) -> bool {
    let mut best_score = i64::MIN;
    let mut best_play: Option<Position> = None;
    for y in 0..8 {
        for x in 0..8 {
            let mut copy = board.clone();
            let position = Position { x, y };
            if copy.play_at(position, player) {
                let score = negamax(&copy, 8, player);
                if score > best_score {
                    best_score = score;
                    best_play = Some(position);
                }
            }
        }
    }
    match best_play {
        Some(position) => {
            board.play_at(position, player);
            println!("{}", Command::stringify(&Command::PlayAt(position)));
            false
        }
        None => true,
    }
}

fn main() {
    let mut board = Board::new();
    let mut current_player = Player::Black;
    let mut input = String::new();
    let mut game_over = false;
    let mut last_passed = false;
    let mut count = 4;
    let args: Vec<String> = std::env::args().collect();
    let machine_player = args.get(1).map_or(Player::Black, |arg|arg_to_player(&arg).expect("invalid color"));
    draw_board(&board);
    while !game_over {
        let passed = if current_player == machine_player {
            machine_play(&mut board, current_player)
        } else {
            human_play(&mut board, current_player, &mut input)
        };
        if !passed {
            count += 1;
        }
        println!("X: {}, O: {}", board.player_score(Player::Black), board.player_score(Player::White));
        game_over = (passed && last_passed) || count == 64;
        last_passed = passed;
        draw_board(&board);
        current_player = current_player.opponent()
    }
}
