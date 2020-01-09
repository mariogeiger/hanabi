#![allow(dead_code)]

use ndarray::Array1;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt;

#[derive(Clone, Copy, PartialEq)]
pub struct Color(usize);

const MAXCLUES: usize = 8;
const MAXMISTAKES: usize = 3;
const MAXPLAYERS: usize = 5;
const MAXCARDS: usize = 5;

impl Color {
    pub fn all() -> Vec<Color> {
        (0..5).map(|x| Color(x)).collect()
    }
    pub fn new(color: &str) -> Color {
        match color {
            "r" => Color(0),
            "g" => Color(1),
            "b" => Color(2),
            "y" => Color(3),
            "p" => Color(4),
            _ => panic!(),
        }
    }
    pub fn r() -> Color {
        Color(0)
    }
    pub fn g() -> Color {
        Color(1)
    }
    pub fn b() -> Color {
        Color(2)
    }
    pub fn y() -> Color {
        Color(3)
    }
    pub fn p() -> Color {
        Color(4)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            0 => write!(f, "r"),
            1 => write!(f, "g"),
            2 => write!(f, "b"),
            3 => write!(f, "y"),
            4 => write!(f, "p"),
            _ => panic!(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Value(usize);

impl Value {
    pub fn new(value: usize) -> Value {
        assert!(value < 5);
        Value(value)
    }
    pub fn all() -> Vec<Value> {
        (0..5).map(|x| Value(x)).collect()
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0 + 1)
    }
}

#[derive(Clone, Copy)]
pub struct Card {
    value: Value,
    color: Color,
}

impl Card {
    fn new(value: Value, color: Color) -> Card {
        Card {
            value: value,
            color: color,
        }
    }

    fn deck() -> Vec<Card> {
        let mut deck = Vec::new();
        for color in Color::all() {
            for value in Value::all() {
                let copies = [3, 2, 2, 2, 1][value.0];
                for _ in 0..copies {
                    deck.push(Card::new(value, color));
                }
            }
        }
        deck
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.value, self.color)
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub enum Action {
    Play {
        player: usize,
        position: usize,
        card: Card,
        success: bool,
    },
    Discard {
        player: usize,
        position: usize,
        card: Card,
    },
    ColorClue {
        player: usize,
        target: usize,
        color: Color,
    },
    ValueClue {
        player: usize,
        target: usize,
        value: Value,
    },
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Play {
                player,
                position,
                card,
                success,
            } => {
                if *success {
                    write!(
                        f,
                        "P{} plays {} from position #{}",
                        player + 1,
                        card,
                        position + 1
                    )
                } else {
                    write!(
                        f,
                        "P{} plays wrongly {} from position #{}",
                        player + 1,
                        card,
                        position + 1
                    )
                }
            }
            Action::Discard {
                player,
                position,
                card,
            } => write!(
                f,
                "P{} discard {} from position #{}",
                player + 1,
                card,
                position + 1
            ),
            Action::ColorClue {
                player,
                target,
                color,
            } => write!(f, "P{} clues P{} about {}'s", player + 1, target + 1, color),
            Action::ValueClue {
                player,
                target,
                value,
            } => write!(f, "P{} clues P{} about {}'s", player + 1, target + 1, value),
        }
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug)]
pub struct State {
    turn: usize,
    clues: usize,
    mistakes: usize,
    players: Vec<Vec<Card>>,
    table: [usize; 5],
    deck: Vec<Card>,
    discard: Vec<Card>,
    history: Vec<Action>,
}

#[derive(Debug)]
pub enum IllegalMoves {
    MaxClue,
    NoMoreClues,
    SelfClue,
    EmptyClue,
}

impl State {
    pub fn new(nplayer: usize) -> State {
        let mut deck = Card::deck();
        deck.shuffle(&mut thread_rng());

        let nc = [0, 0, MAXCARDS, MAXCARDS, MAXCARDS - 1, MAXCARDS - 1][nplayer];
        let players: Vec<Vec<Card>> = (0..nplayer)
            .map(|i| deck[i * nc..(i + 1) * nc].to_vec())
            .collect();
        deck = deck[nplayer * nc..].to_vec();

        State {
            turn: 0,
            clues: MAXCLUES,
            mistakes: 0,
            players: players,
            table: [0; 5],
            deck: deck,
            discard: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn discard(&mut self, position: usize) -> Result<(), IllegalMoves> {
        if self.clues >= MAXCLUES {
            return Result::Err(IllegalMoves::MaxClue);
        }
        let p = self.turn % self.players.len();
        let card = self.players[p].remove(position);
        self.discard.push(card);
        self.clues += 1;

        if let Some(card) = self.deck.pop() {
            self.players[p].insert(0, card);
        }

        self.history.push(Action::Discard {
            player: p,
            position: position,
            card: card,
        });
        self.turn += 1;

        Result::Ok(())
    }

    pub fn play(&mut self, position: usize) {
        let p = self.turn % self.players.len();
        let card = self.players[p].remove(position);
        let success = self.table[card.color.0] == card.value.0;

        if success {
            self.table[card.color.0] += 1;
        } else {
            self.discard.push(card);
            self.mistakes += 1;
        }

        if let Some(card) = self.deck.pop() {
            self.players[p].insert(0, card);
        }

        self.history.push(Action::Play {
            player: p,
            position: position,
            card: card,
            success: success,
        });
        self.turn += 1;
    }

    pub fn clue_color(&mut self, target: usize, color: Color) -> Result<(), IllegalMoves> {
        let p = self.turn % self.players.len();
        if p == target {
            return Result::Err(IllegalMoves::SelfClue);
        }
        if self.clues == 0 {
            return Result::Err(IllegalMoves::NoMoreClues);
        }
        if !self.players[target].iter().any(|x| x.color == color) {
            return Result::Err(IllegalMoves::EmptyClue);
        }
        self.clues -= 1;

        self.history.push(Action::ColorClue {
            player: p,
            target: target,
            color: color,
        });
        self.turn += 1;

        Result::Ok(())
    }

    pub fn clue_value(&mut self, target: usize, value: Value) -> Result<(), IllegalMoves> {
        let p = self.turn % self.players.len();
        if p == target {
            return Result::Err(IllegalMoves::SelfClue);
        }
        if self.clues == 0 {
            return Result::Err(IllegalMoves::NoMoreClues);
        }
        if !self.players[target].iter().any(|x| x.value == value) {
            return Result::Err(IllegalMoves::EmptyClue);
        }
        self.clues -= 1;

        self.history.push(Action::ValueClue {
            player: p,
            target: target,
            value: value,
        });
        self.turn += 1;

        Result::Ok(())
    }

    pub fn score(&self) -> usize {
        self.table.iter().sum()
    }

    pub fn turn(&self) -> usize {
        self.turn
    }

    pub fn encode(&self) -> Array1<f32> {
        let mut x = Array1::from_elem((MAXPLAYERS - 2 + 1) + MAXPLAYERS + MAXCLUES + MAXMISTAKES + MAXPLAYERS * MAXCARDS * 10 + 100 * 19, -1.0);
        let mut off = 0;

        x[off + self.players.len() - 2] = 1.0;
        off += MAXPLAYERS - 2 + 1;

        x[off + self.turn % self.players.len()] = 1.0;
        off += MAXPLAYERS;

        for _ in 0..self.clues {
            x[off] = 1.0;
            off += 1;
        }
        off += MAXCLUES - self.clues;

        for _ in 0..self.mistakes {
            x[off] = 1.0;
            off += 1;
        }
        off += MAXMISTAKES - self.mistakes;

        for cards in &self.players {
            for card in cards {
                x[off + card.value.0] = 1.0;
                off += 5;
                x[off + card.color.0] = 1.0;
                off += 5;
            }
            off += (MAXCARDS - cards.len()) * 10;
        }
        off += (MAXPLAYERS - self.players.len()) * MAXCARDS * 10;

        for &cards in &self.table {
            for _ in 0..cards {
                x[off] = 1.0;
                off += 1;
            }
            off += 5 - cards;
        }

        for action in self.history.iter().rev() {
            assert!(MAXCARDS == MAXPLAYERS);
            match action {
                Action::Play {
                    player: _,
                    position,
                    card,
                    success,
                } => {
                    if *success {
                        x[off] = 1.0;
                    } else {
                        x[off + 1] = 1.0;
                    }
                    off += 4;

                    x[off + position] = 1.0;
                    off += MAXCARDS;

                    x[off + card.value.0] = 1.0;
                    off += 5;
                    x[off + card.color.0] = 1.0;
                    off += 5;
                }
                Action::Discard {
                    player: _,
                    position,
                    card,
                } => {
                    x[off + 2] = 1.0;
                    off += 4;

                    x[off + position] = 1.0;
                    off += MAXCARDS;

                    x[off + card.value.0] = 1.0;
                    off += 5;
                    x[off + card.color.0] = 1.0;
                    off += 5;
                }
                Action::ColorClue {
                    player: _,
                    target,
                    color,
                } => {
                    x[off + 3] = 1.0;
                    off += 4;

                    x[off + target] = 1.0;
                    off += MAXPLAYERS;

                    off += 5;
                    x[off + color.0] = 1.0;
                    off += 5;
                }
                Action::ValueClue {
                    player: _,
                    target,
                    value,
                } => {
                    x[off + 3] = 1.0;
                    off += 4;

                    x[off + target] = 1.0;
                    off += MAXPLAYERS;

                    x[off + value.0] = 1.0;
                    off += 5;
                    off += 5;
                }
            }
        }

        x
    }
}
