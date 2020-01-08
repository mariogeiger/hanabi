#![allow(dead_code)]

extern crate rand;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt;

#[derive(Clone, Copy)]
struct Color(i32);

impl Color {
    fn all() -> Vec<Color> {
        vec![Color(0), Color(1), Color(2), Color(3), Color(4)]
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

#[derive(Clone, Copy)]
struct Value(i32);

impl Value {
    fn all() -> Vec<Value> {
        vec![Value(0), Value(1), Value(2), Value(3), Value(4)]
    }
    fn repetition(&self) -> i32 {
        match self.0 {
            0 => 3,
            1 => 2,
            2 => 2,
            3 => 2,
            4 => 1,
            _ => 0,
        }
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
                for _ in 0..value.repetition() {
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

#[derive(Debug)]
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
    Clue {
        player: i32,
        target: i32,
        position: i32,
        card: Card,
    },
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

const MAXCLUES: i32 = 8;
const MAXMISTAKES: i32 = 3;

impl State {
    fn new() -> State {
        let mut deck = Card::deck();
        deck.shuffle(&mut thread_rng());

        let mut iter = deck.chunks_exact(5);
        let players = vec![
            iter.next().unwrap().to_vec(),
            iter.next().unwrap().to_vec(),
            iter.next().unwrap().to_vec(),
        ];
        deck = deck[players.len() * 5..].to_vec();

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

    fn discard(&mut self, position: i32) {
        assert!(self.clues < MAXCLUES);
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

    fn clue(&mut self, target: i32, position: i32) {
        let p = self.turn % self.players.len() as i32;
        assert!(p != target);
        assert!(self.clues > 0);
        let card = self.players[target as usize][position as usize];
        self.clues -= 1;

        self.history.push(Action::Clue {
            player: p,
            target: target,
            position: position,
            card: card,
        });
        self.turn += 1;
    }
}

fn main() {
    let mut state = State::new();
    println!("{:?}", state);

    state.clue(1, 0);
    println!("{:?}", state);

    state.play(0);
    println!("{:?}", state);

    state.discard(4);
    println!("{:?}", state);
}
