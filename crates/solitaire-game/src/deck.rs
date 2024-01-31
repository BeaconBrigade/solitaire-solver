/// Types for the deck of cards
///
use std::{fmt::Write, str::FromStr};

use rand::seq::SliceRandom;

#[derive(Debug, Clone, Copy)]
pub struct Deck(pub [Card; 52]);

impl Deck {
    pub fn new_shuffled() -> Self {
        let mut rng = rand::thread_rng();
        let mut deck = ORDERED;
        deck.shuffle(&mut rng);

        Self(deck)
    }

    pub fn new_ordered() -> Self {
        Self(ORDERED)
    }
}

impl Default for Deck {
    fn default() -> Self {
        Self::new_shuffled()
    }
}

impl ToString for Deck {
    fn to_string(&self) -> String {
        let mut buf = String::new();
        for card in self.0 {
            writeln!(&mut buf, "{}", card.to_string()).unwrap();
        }
        buf
    }
}

impl FromStr for Deck {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let arr = s
            .lines()
            .map(str::trim)
            .take(52)
            .map(Card::from_str)
            .collect::<Result<Vec<Card>, ()>>()?
            .try_into()
            .map_err(|_| ())?;

        Ok(Self(arr))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card {
    pub suit: Suit,
    pub value: Value,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            suit: Suit::Hearts,
            value: Value::Ace,
        }
    }
}

impl Card {
    pub const fn new(suit: Suit, value: Value) -> Self {
        Self { suit, value }
    }

    pub fn has_same_colour(&self, other: &Card) -> bool {
        ((self.suit == Suit::Hearts || self.suit == Suit::Diamonds)
            && (other.suit == Suit::Hearts || other.suit == Suit::Diamonds))
            || ((self.suit == Suit::Clubs || self.suit == Suit::Spades)
                && (other.suit == Suit::Clubs || other.suit == Suit::Spades))
    }
}

impl ToString for Card {
    fn to_string(&self) -> String {
        format!("{} {}", self.suit.to_string(), self.value.to_string())
    }
}

impl FromStr for Card {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (suit, val) = s.split_once(' ').ok_or(())?;
        Ok(Self::new(Suit::from_str(suit)?, Value::from_str(val)?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Suit {
    Hearts,
    Spades,
    Clubs,
    Diamonds,
}

impl ToString for Suit {
    fn to_string(&self) -> String {
        match self {
            Self::Hearts => "Hearts",
            Self::Spades => "Spades",
            Self::Clubs => "Clubs",
            Self::Diamonds => "Diamonds",
        }
        .to_string()
    }
}

impl FromStr for Suit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Hearts" => Ok(Self::Hearts),
            "Spades" => Ok(Self::Spades),
            "Clubs" => Ok(Self::Clubs),
            "Diamonds" => Ok(Self::Diamonds),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Value {
    Ace = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Ace => "Ace",
            Two => "Two",
            Three => "Three",
            Four => "Four",
            Five => "Five",
            Six => "Six",
            Seven => "Seven",
            Eight => "Eight",
            Nine => "Nine",
            Ten => "Ten",
            Jack => "Jack",
            Queen => "Queen",
            King => "King",
        }
        .to_string()
    }
}

impl FromStr for Value {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Ace" | "1" => Ok(Self::Ace),
            "Two" | "2" => Ok(Self::Two),
            "Three" | "3" => Ok(Self::Three),
            "Four" | "4" => Ok(Self::Four),
            "Five" | "5" => Ok(Self::Five),
            "Six" | "6" => Ok(Self::Six),
            "Seven" | "7" => Ok(Self::Seven),
            "Eight" | "8" => Ok(Self::Eight),
            "Nine" | "9" => Ok(Self::Nine),
            "Ten" | "10" => Ok(Self::Ten),
            "Jack" | "11" => Ok(Self::Jack),
            "Queen" | "12" => Ok(Self::Queen),
            "King" | "13" => Ok(Self::King),
            _ => Err(()),
        }
    }
}

use Suit::*;
use Value::*;
const ORDERED: [Card; 52] = [
    Card::new(Hearts, Ace),
    Card::new(Hearts, Two),
    Card::new(Hearts, Three),
    Card::new(Hearts, Four),
    Card::new(Hearts, Five),
    Card::new(Hearts, Six),
    Card::new(Hearts, Seven),
    Card::new(Hearts, Eight),
    Card::new(Hearts, Nine),
    Card::new(Hearts, Ten),
    Card::new(Hearts, Jack),
    Card::new(Hearts, Queen),
    Card::new(Hearts, King),
    Card::new(Diamonds, Ace),
    Card::new(Diamonds, Two),
    Card::new(Diamonds, Three),
    Card::new(Diamonds, Four),
    Card::new(Diamonds, Five),
    Card::new(Diamonds, Six),
    Card::new(Diamonds, Seven),
    Card::new(Diamonds, Eight),
    Card::new(Diamonds, Nine),
    Card::new(Diamonds, Ten),
    Card::new(Diamonds, Jack),
    Card::new(Diamonds, Queen),
    Card::new(Diamonds, King),
    Card::new(Spades, Ace),
    Card::new(Spades, Two),
    Card::new(Spades, Three),
    Card::new(Spades, Four),
    Card::new(Spades, Five),
    Card::new(Spades, Six),
    Card::new(Spades, Seven),
    Card::new(Spades, Eight),
    Card::new(Spades, Nine),
    Card::new(Spades, Ten),
    Card::new(Spades, Jack),
    Card::new(Spades, Queen),
    Card::new(Spades, King),
    Card::new(Clubs, Ace),
    Card::new(Clubs, Two),
    Card::new(Clubs, Three),
    Card::new(Clubs, Four),
    Card::new(Clubs, Five),
    Card::new(Clubs, Six),
    Card::new(Clubs, Seven),
    Card::new(Clubs, Eight),
    Card::new(Clubs, Nine),
    Card::new(Clubs, Ten),
    Card::new(Clubs, Jack),
    Card::new(Clubs, Queen),
    Card::new(Clubs, King),
];
