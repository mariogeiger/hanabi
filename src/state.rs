#![allow(dead_code)]

use getset::Getters;
use ndarray::{s, Array1, ArrayView1};
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
    pub fn new(color: usize) -> Color {
        assert!(color < 5);
        Color(color)
    }
    pub fn from_str(color: &str) -> Color {
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

    pub fn copies(&self) -> usize {
        [3, 2, 2, 2, 1][self.0]
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
                for _ in 0..value.copies() {
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

#[derive(Debug, Getters)]
#[get = "pub"]
pub struct State {
    turn: usize,
    turn_empty_deck: usize,
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
    GameOver,
    Error,
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
            turn_empty_deck: 0,
            clues: MAXCLUES,
            mistakes: 0,
            players: players,
            table: [0; 5],
            deck: deck,
            discard: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn gameover(&self) -> bool {
        self.turn_empty_deck > self.players.len()
            || self.mistakes >= MAXMISTAKES
            || self.score() >= 25
    }

    pub fn play(&mut self, position: usize) -> Result<(), IllegalMoves> {
        if self.gameover() {
            return Err(IllegalMoves::GameOver);
        }
        let p = self.turn % self.players.len();
        let cards = &mut self.players[p];
        if position >= cards.len() {
            return Err(IllegalMoves::Error);
        }
        let card = cards.remove(position);
        let success = self.table[card.color.0] == card.value.0;

        if success {
            self.table[card.color.0] += 1;
        } else {
            self.discard.push(card);
            self.mistakes += 1;
        }

        if let Some(card) = self.deck.pop() {
            self.players[p].insert(0, card);
        } else {
            self.turn_empty_deck += 1
        }

        self.history.push(Action::Play {
            player: p,
            position: position,
            card: card,
            success: success,
        });
        self.turn += 1;

        Ok(())
    }

    pub fn play_discard(&mut self, position: usize) -> Result<(), IllegalMoves> {
        if self.clues >= MAXCLUES {
            return Err(IllegalMoves::MaxClue);
        }
        if self.gameover() {
            return Err(IllegalMoves::GameOver);
        }
        let p = self.turn % self.players.len();
        let cards = &mut self.players[p];
        if position >= cards.len() {
            return Err(IllegalMoves::Error);
        }
        let card = cards.remove(position);
        self.discard.push(card);
        self.clues += 1;

        if let Some(card) = self.deck.pop() {
            self.players[p].insert(0, card);
        } else {
            self.turn_empty_deck += 1;
        }

        self.history.push(Action::Discard {
            player: p,
            position: position,
            card: card,
        });
        self.turn += 1;

        Ok(())
    }

    fn clue<F>(&mut self, target: usize, f: F) -> Result<usize, IllegalMoves>
    where
        F: Fn(&Card) -> bool,
    {
        if target >= self.players.len() {
            return Err(IllegalMoves::Error);
        }
        if self.gameover() {
            return Err(IllegalMoves::GameOver);
        }
        let p = self.turn % self.players.len();
        if p == target {
            return Err(IllegalMoves::SelfClue);
        }
        if self.clues == 0 {
            return Err(IllegalMoves::NoMoreClues);
        }
        if !self.players[target].iter().any(f) {
            return Err(IllegalMoves::EmptyClue);
        }
        self.clues -= 1;

        if self.deck.is_empty() {
            self.turn_empty_deck += 1;
        }
        self.turn += 1;

        Ok(p)
    }

    pub fn clue_color(&mut self, target: usize, color: Color) -> Result<(), IllegalMoves> {
        let p = self.clue(target, |x| x.color == color)?;

        self.history.push(Action::ColorClue {
            player: p,
            target: target,
            color: color,
        });

        Ok(())
    }

    pub fn clue_value(&mut self, target: usize, value: Value) -> Result<(), IllegalMoves> {
        let p = self.clue(target, |x| x.value == value)?;

        self.history.push(Action::ValueClue {
            player: p,
            target: target,
            value: value,
        });

        Ok(())
    }

    pub fn score(&self) -> usize {
        self.table.iter().sum()
    }

    pub fn encode(&self) -> Array1<f32> {
        let mut x = Array1::from_elem(
            (MAXPLAYERS - 2 + 1)
                + MAXPLAYERS
                + MAXCLUES
                + MAXMISTAKES
                + 50
                + 5 * 10
                + MAXPLAYERS * MAXCARDS * 10
                + 100 * 19,
            -1.0,
        );
        let mut off = 0;

        x[off + self.players.len() - 2] = 1.0;
        off += MAXPLAYERS - 2 + 1;

        let player = self.turn % self.players.len();
        x[off + player] = 1.0;
        off += MAXPLAYERS;

        for i in 0..self.clues {
            x[off + i] = 1.0;
        }
        off += MAXCLUES;

        for i in 0..self.mistakes {
            x[off + i] = 1.0;
        }
        off += MAXMISTAKES;

        for i in 0..self.deck.len() {
            x[off + i] = 1.0;
        }
        off += 50;

        for color in Color::all() {
            let cards: Vec<Card> = self
                .discard
                .iter()
                .filter(|card| card.color == color)
                .cloned()
                .collect();
            for value in Value::all() {
                for i in 0..cards.iter().filter(|card| card.value == value).count() {
                    x[off + i] = 1.0;
                }
                off += value.copies();
            }
        }

        for (i, cards) in self.players.iter().enumerate() {
            if i != player {
                for (j, card) in cards.iter().enumerate() {
                    x[off + 10 * j + card.value.0] = 1.0;
                    x[off + 10 * j + 5 + card.color.0] = 1.0;
                }
            }
            off += MAXCARDS * 10;
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

    pub fn decode(&mut self, x: &ArrayView1<f32>) -> Result<(), IllegalMoves> {
        if x.len() != 3 + MAXCARDS + MAXPLAYERS + 10 {
            return Err(IllegalMoves::Error);
        }
        match argmax(&x.slice(s![..3])) {
            0 => {
                self.play(argmax(&x.slice(s![3..3 + MAXCARDS])))?;
            }
            1 => {
                self.play_discard(argmax(&x.slice(s![3..3 + MAXCARDS])))?;
            }
            2 => {
                let target = argmax(&x.slice(s![3 + MAXCARDS..3 + MAXCARDS + MAXPLAYERS]));
                let i = argmax(&x.slice(s![-10..]));
                if i < 5 {
                    self.clue_value(target, Value::new(i))?;
                } else {
                    self.clue_color(target, Color::new(i - 5))?;
                }
            }
            _ => {
                panic!();
            }
        }
        Ok(())
    }
}

fn argmax(x: &ArrayView1<f32>) -> usize {
    let mut i = 0;
    let mut max = x[0];
    for j in 1..x.len() {
        if x[j] > max {
            i = j;
            max = x[j];
        }
    }
    i
}
