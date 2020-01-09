#![allow(dead_code)]

extern crate rand;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt;

#[derive(Clone, Copy, PartialEq)]
struct Color(i32);

impl Color {
    fn all() -> Vec<Color> {
        (0..5).map(|x| Color(x)).collect()
    }
    fn r() -> Color {
        Color(0)
    }
    fn g() -> Color {
        Color(1)
    }
    fn b() -> Color {
        Color(2)
    }
    fn y() -> Color {
        Color(3)
    }
    fn p() -> Color {
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
            _ => write!(f, "?"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Value(i32);

impl Value {
    fn all() -> Vec<Value> {
        (0..5).map(|x| Value(x)).collect()
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0 + 1)
    }
}

#[derive(Clone, Copy)]
struct Card {
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
                let copies = [3, 2, 2, 2, 1][value.0 as usize];
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

enum Action {
    Play {
        player: i32,
        position: i32,
        card: Card,
        success: bool,
    },
    Discard {
        player: i32,
        position: i32,
        card: Card,
    },
    ColorClue {
        player: i32,
        target: i32,
        color: Color,
    },
    ValueClue {
        player: i32,
        target: i32,
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
            } => write!(
                f,
                "P{} clues P{} about {}'s",
                player + 1,
                target + 1,
                color
            ),
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
struct State {
    turn: i32,
    clues: i32,
    mistakes: i32,
    players: Vec<Vec<Card>>,
    table: [i32; 5],
    deck: Vec<Card>,
    discard: Vec<Card>,
    history: Vec<Action>,
}

#[derive(Debug)]
enum IllegalMoves {
    MaxClue,
    NoMoreClues,
    SelfClue,
    EmptyClue,
}

const MAXCLUES: i32 = 8;
const MAXMISTAKES: i32 = 3;

impl State {
    fn new(nplayer: usize) -> State {
        let mut deck = Card::deck();
        deck.shuffle(&mut thread_rng());

        let nc = [0, 0, 5, 5, 4, 4][nplayer as usize];
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

    fn discard(&mut self, position: i32) -> Result<(), IllegalMoves> {
        if self.clues >= MAXCLUES {
            return Result::Err(IllegalMoves::MaxClue);
        }
        let p = self.turn % self.players.len() as i32;
        let card = self.players[p as usize].remove(position as usize);
        self.discard.push(card);
        self.clues += 1;

        if let Some(card) = self.deck.pop() {
            self.players[p as usize].insert(0, card);
        }

        self.history.push(Action::Discard {
            player: p,
            position: position,
            card: card,
        });
        self.turn += 1;

        Result::Ok(())
    }

    fn play(&mut self, position: i32) {
        let p = self.turn % self.players.len() as i32;
        let card = self.players[p as usize].remove(position as usize);
        let success = self.table[card.color.0 as usize] == card.value.0 as i32;

        if success {
            self.table[card.color.0 as usize] += 1;
        } else {
            self.discard.push(card);
            self.mistakes += 1;
        }

        if let Some(card) = self.deck.pop() {
            self.players[p as usize].insert(0, card);
        }

        self.history.push(Action::Play {
            player: p,
            position: position,
            card: card,
            success: success,
        });
        self.turn += 1;
    }

    fn clue_color(&mut self, target: i32, color: Color) -> Result<(), IllegalMoves> {
        let p = self.turn % self.players.len() as i32;
        if p == target {
            return Result::Err(IllegalMoves::SelfClue);
        }
        if self.clues == 0 {
            return Result::Err(IllegalMoves::NoMoreClues);
        }
        if !self.players[target as usize]
            .iter()
            .any(|x| x.color == color)
        {
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

    fn clue_value(&mut self, target: i32, value: Value) -> Result<(), IllegalMoves> {
        let p = self.turn % self.players.len() as i32;
        if p == target {
            return Result::Err(IllegalMoves::SelfClue);
        }
        if self.clues == 0 {
            return Result::Err(IllegalMoves::NoMoreClues);
        }
        if !self.players[target as usize]
            .iter()
            .any(|x| x.value == value)
        {
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
}

fn main() {
    let mut state = State::new(3);
    println!("{:?}", state);

    state.clue_color(1, Color::r()).unwrap();
    println!("{:?}", state);

    state.play(0);
    println!("{:?}", state);

    state.discard(4).unwrap();
    println!("{:?}", state);
}
