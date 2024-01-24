/// Types for the deck of cards
///
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Suit {
    Hearts,
    Spades,
    Clubs,
    Diamonds,
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
