//! common functions to help working with states

use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coord {
    pub location: Location,
    pub idx: u8,
}

impl Coord {
    pub fn new(location: Location, idx: u8) -> Self {
        Self { location, idx }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Location {
    /// Contains which foundation stack this coord is in
    Foundation(u8),
    /// Contains which tableau stack this coord is in
    Tableau(u8),
    Talon,
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Foundation(p) => write!(f, "Foundation-{}", p),
            Self::Tableau(p) => write!(f, "Tableau-{}", p),
            Self::Talon => write!(f, "Talon"),
        }
    }
}

impl FromStr for Location {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((location, pile)) = s.split_once('-') {
            match location {
                "Foundation" => Ok(Self::Foundation(FromStr::from_str(pile).map_err(|_| ())?)),
                "Tableau" => Ok(Self::Tableau(FromStr::from_str(pile).map_err(|_| ())?)),
                _ => Err(()),
            }
        } else if s == "Talon" {
            Ok(Self::Talon)
        } else {
            Err(())
        }
    }
}

pub(crate) fn iter_to_arr<const N: usize, T: Copy>(
    iter: &mut impl Iterator<Item = T>,
) -> [Option<T>; N] {
    let mut a = [None; N];
    a.iter_mut()
        .take(N)
        .zip(iter)
        .for_each(|(a, i)| *a = Some(i));

    a
}

pub(crate) fn combine<const N: usize, const M: usize, const L: usize, T: Default + Copy>(
    n: [T; N],
    m: [T; M],
) -> [T; L] {
    debug_assert_eq!(N + M, L);
    let mut l = [T::default(); L];

    l[..N].copy_from_slice(&n[..N]);
    l[N..(M + N)].copy_from_slice(&m[..M]);

    l
}

/// Find last occurence of item in an iterator, returning its index; breaks on first false
pub fn find_last_idx<T>(
    iter: impl Iterator<Item = T>,
    mut pred: impl FnMut(&T) -> bool,
) -> Option<usize> {
    let mut idx = None;

    for (i, item) in iter.enumerate() {
        if pred(&item) {
            idx = Some(i);
        } else {
            break;
        }
    }

    idx
}

/// Find last occurence of item in an iterator returning said item, breaks on first false
pub fn find_last<T>(iter: impl Iterator<Item = T>, mut pred: impl FnMut(&T) -> bool) -> Option<T> {
    let mut res = None;

    for item in iter {
        if pred(&item) {
            res = Some(item);
        } else {
            break;
        }
    }

    res
}
