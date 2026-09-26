#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use core::array::from_fn;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Letter(u8);

impl Letter {
    pub fn new(n: u8) -> Option<Letter> {
        if n < 26 { Some(Letter(n)) } else { None }
    }

    pub fn from_char(c: char) -> Option<Letter> {
        if c.is_ascii_uppercase() {
            Some(Letter(c as u8 - b'A'))
        } else {
            None
        }
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug)]
pub struct NotALetter(pub char);

impl TryFrom<char> for Letter {
    type Error = NotALetter;
    fn try_from(c: char) -> Result<Letter, NotALetter> {
        Letter::from_char(c).ok_or(NotALetter(c))
    }
}

impl From<Letter> for char {
    fn from(l: Letter) -> char {
        (b'A' + l.0) as char
    }
}
pub struct Wiring {
    forward: [u8; 26],
    inverse: [u8; 26],
}

#[derive(Debug, PartialEq)]
pub enum WiringError {
    WrongLength(usize),
    BadChar(char),
    NotAPermutation,
    NotInvolution,
}

impl Wiring {
    pub fn from_alphabet_string(s: &str) -> Result<Wiring, WiringError> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() != 26 {
            return Err(WiringError::WrongLength(chars.len()));
        }
        let mut table = [0u8; 26];
        for (i, &c) in chars.iter().enumerate() {
            if !c.is_ascii_uppercase() {
                return Err(WiringError::BadChar(c));
            }
            table[i] = c as u8 - b'A';
        }
        Wiring::from_table(table)
    }
    pub fn forward(&self, l: Letter) -> Letter {
        Letter(self.forward[l.index()])
    }
    pub fn inverse(&self, l: Letter) -> Letter {
        Letter(self.inverse[l.index()])
    }

    pub fn from_table(table: [u8; 26]) -> Result<Wiring, WiringError> {
        let mut seen = [false; 26];
        for &v in &table {
            if v >= 26 {
                return Err(WiringError::BadChar((b'A' + v) as char)); // or a dedicated variant
            }
            if seen[v as usize] {
                return Err(WiringError::NotAPermutation);
            }
            seen[v as usize] = true;
        }

        let forward = table;
        let mut inverse = [0u8; 26];
        for (i, &v) in forward.iter().enumerate() {
            inverse[v as usize] = i as u8;
        }
        Ok(Wiring { forward, inverse })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_round_trips() {
        for c in 'A'..='Z' {
            let l = Letter::try_from(c).unwrap();
            assert_eq!(char::from(l), c);
        }
    }

    #[test]
    fn index_matches_position_in_alphabet() {
        assert_eq!(Letter::try_from('A').unwrap().index(), 0);
        assert_eq!(Letter::try_from('Z').unwrap().index(), 25);
    }

    #[test]
    fn lowercase_and_non_letters_are_rejected() {
        assert!(Letter::try_from('a').is_err());
        assert!(Letter::try_from('1').is_err());
        assert!(Letter::try_from(' ').is_err());
    }
}

#[cfg(test)]
mod wiring_tests {
    use std::array::from_fn;

    use proptest::prop_assert_eq;

    use super::*;

    const ROTOR_I: &str = "EKMFLGDQVZNTOWYHXUSPAIBRCJ";

    fn random_wiring(seed: u64) -> Wiring {
        let mut a: [u8; 26] = from_fn(|i| i as u8);
        let mut state = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        for i in (1..26).rev() {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = (state >> 33) as usize % (i + 1);
            a.swap(i, j);
        }

        Wiring::from_table(a).unwrap()
    }

    #[test]
    fn forward_maps_to_first_letter() {
        let w = Wiring::from_alphabet_string(ROTOR_I).unwrap();
        assert_eq!(
            w.forward(Letter::try_from('A').unwrap()),
            Letter::try_from('E').unwrap()
        );
    }

    #[test]
    fn rejects_non_permutations() {
        assert!(Wiring::from_alphabet_string("ABC").is_err());
        assert!(Wiring::from_alphabet_string("AACDEFGHIJKLMNOPQRSTUVWXYZ").is_err());
        assert!(Wiring::from_alphabet_string("EKMFLGDQVZNTOWYHXUSPAIBRC1").is_err());
    }

    proptest::proptest! {
        #[test]
        fn inverse_undoes_forward(seed in 0u64 .. 10_000) {
            let w = random_wiring(seed);
            for i in 0..26u8 {
                let x = Letter::try_from((b'A' + i) as char).unwrap();
                prop_assert_eq!(w.inverse(w.forward(x)), x)
            }
        }
    }
}

pub struct Rotor {
    wiring: Wiring,
    notches: Vec<Letter>,
    ring: Letter,
    pos: Letter,
}

impl Rotor {
    pub fn new(wiring: Wiring, notches: Vec<Letter>, ring: Letter, pos: Letter) -> Rotor {
        Rotor {
            wiring,
            notches,
            ring,
            pos,
        }
    }

    fn shift(&self) -> i16 {
        (self.pos.index() as i16 - self.ring.index() as i16).rem_euclid(26)
    }

    pub fn encode_forward(&self, c: Letter) -> Letter {
        let shift = self.shift();
        let entry = (c.index() as i16 + shift).rem_euclid(26) as u8;
        let exit = self.wiring.forward(Letter(entry)).index() as i16;
        Letter((exit - shift).rem_euclid(26) as u8)
    }

    pub fn encode_backward(&self, c: Letter) -> Letter {
        let shift = self.shift();
        let entry = (c.index() as i16 + shift).rem_euclid(26) as u8;
        let exit = self.wiring.inverse(Letter(entry)).index() as i16;
        Letter((exit - shift).rem_euclid(26) as u8)
    }

    pub fn step(&mut self) {
        self.pos = Letter((self.pos.index() as u8 + 1) % 26);
    }

    pub fn is_at_notch(&self) -> bool {
        self.notches.iter().any(|&n| n == self.pos)
    }

    pub fn pos(&self) -> Letter {
        self.pos
    }

    pub fn set_pos(&mut self, p: Letter) {
        self.pos = p
    }
}

#[cfg(test)]
mod rotor_offset_tests {
    use proptest::prop_assert_eq;

    use super::*;

    fn l(c: char) -> Letter {
        Letter::try_from(c).unwrap()
    }

    const TEST_1_STRING: &str = "EKMFLGDQVZNTOWYHXUSPAIBRCJ";
    const TEST_3_STRING: &str = "BDFHJLCPRTXVZNYEIWGAKMUSQO";

    #[test]
    fn identity_ring_and_position_is_just_the_wiring() {
        let r = Rotor::new(
            Wiring::from_alphabet_string(TEST_1_STRING).unwrap(),
            vec![l('Q')],
            l('A'),
            l('A'),
        );
        assert_eq!(r.encode_forward(l('A')), l('E'))
    }
    #[test]
    fn stepping_the_position_shifts_the_contact() {
        // Rotor I, ring A, position K (shift = 10). Encoding 'A':
        //   entry = (0 + 10) = 10  ->  wiring[10] = 'N' (13)  ->  out = (13 - 10) = 3 = 'D'.
        let r = Rotor::new(
            Wiring::from_alphabet_string(TEST_1_STRING).unwrap(),
            vec![l('Q')],
            l('A'),
            l('K'),
        );
        assert_eq!(r.encode_forward(l('A')), l('D'));
    }

    #[test]
    fn forward_then_backward_is_identity_at_any_setting() {
        let r = Rotor::new(
            Wiring::from_alphabet_string(TEST_3_STRING).unwrap(),
            vec![l('V')],
            l('F'),
            l('K'),
        );
        for c in 'A'..='Z' {
            assert_eq!(r.encode_backward(r.encode_forward(l(c))), l(c));
        }
    }

    proptest::proptest! {
        #[test]
        fn offset_depends_only_on_pos_minus_ring(pos in 0u8..26, ring in 0u8..26, k in 0u8..26, c in 0u8..26) {
            let wiring = || Wiring::from_alphabet_string("EKMFLGDQVZNTOWYHXUSPAIBRCJ").unwrap();
            let base = Rotor::new(wiring(), vec![Letter::new(0).unwrap()], Letter::new(ring).unwrap(), Letter::new(pos).unwrap());
            let shift = Rotor::new(wiring(), vec![Letter::new(0).unwrap()], Letter::new((ring + k) % 26).unwrap(), Letter::new((pos + k) % 26).unwrap());
            let x = Letter::new(c).unwrap();
            prop_assert_eq!(base.encode_forward(x), shift.encode_forward(x));
        }
    }
}

pub struct Reflector(Wiring);

impl Reflector {
    pub fn from_alphabet_string(s: &str) -> Result<Reflector, WiringError> {
        let w = Wiring::from_alphabet_string(s)?;
        for i in 0..26u8 {
            let l = Letter(i);
            if w.forward(l) == l {
                return Err(WiringError::NotInvolution);
            }
            if w.forward(w.forward(l)) != l {
                return Err(WiringError::NotInvolution);
            }
        }
        Ok(Reflector(w))
    }
    pub fn map(&self, l: Letter) -> Letter {
        self.0.forward(l)
    }
}

pub struct Plugboard {
    map: [u8; 26],
}

impl Plugboard {
    pub fn parse(s: &str) -> Result<Plugboard, WiringError> {
        let mut map: [u8; 26] = from_fn(|i| i as u8);
        for pair in s.split_whitespace() {
            let bytes: Vec<char> = pair.chars().collect();
            if bytes.len() != 2 {
                return Err(WiringError::WrongLength(bytes.len()));
            }
            let a = Letter::from_char(bytes[0])
                .ok_or(WiringError::BadChar(bytes[0]))?
                .index();
            let b = Letter::from_char(bytes[1])
                .ok_or(WiringError::BadChar(bytes[1]))?
                .index();
            if map[a] != a as u8 || map[b] != b as u8 {
                return Err(WiringError::NotAPermutation);
            }
            map[a] = b as u8;
            map[b] = a as u8;
        }
        Ok(Plugboard { map })
    }
    pub fn map(&self, l: Letter) -> Letter {
        Letter(self.map[l.index()])
    }
}

#[cfg(test)]
mod involution_tests {
    use super::*;
    fn l(c: char) -> Letter {
        Letter::try_from(c).unwrap()
    }

    #[test]
    fn reflector_b_is_a_fixed_point_free_involution() {
        let r = Reflector::from_alphabet_string("YRUHQSLDPXNGOKMIEBFZCWVJAT").unwrap();
        for c in 'A'..='Z' {
            assert_ne!(r.map(l(c)), l(c));
            assert_eq!(r.map(r.map(l(c))), l(c));
        }
    }

    #[test]
    fn plugboard_swaps_pairs_and_passes_others_through() {
        let pb = Plugboard::parse("AM FI").unwrap();
        assert_eq!(pb.map(l('A')), l('M'));
        assert_eq!(pb.map(l('M')), l('A'));
        assert_eq!(pb.map(l('F')), l('I'));
        assert_eq!(pb.map(l('Q')), l('Q'));
    }
    #[test]
    fn plugboard_rejects_reused_letters() {
        assert!(Plugboard::parse("AM AF").is_err()); // 'A' used twice
    }

    #[test]
    fn empty_plugboard_is_identity() {
        let pb = Plugboard::parse("").unwrap();
        for c in 'A'..='Z' {
            assert_eq!(pb.map(l(c)), l(c));
        }
    }
}

pub fn step_stack(left: &mut Rotor, mid: &mut Rotor, right: &mut Rotor) {
    let right_at_notch = right.is_at_notch();
    let mid_at_notch = mid.is_at_notch();

    if mid_at_notch {
        left.step();
        mid.step();
    } else if right_at_notch {
        mid.step();
    }
    right.step();
}

#[cfg(test)]
mod stepping_tests {
    use super::*;

    fn l(c: char) -> Letter {
        Letter::try_from(c).unwrap()
    }

    // (wiring, notch, start position) → a Rotor with ring A.
    fn rotor(wiring: &str, notch: char, pos: char) -> Rotor {
        Rotor::new(
            Wiring::from_alphabet_string(wiring).unwrap(),
            vec![l(notch)],
            l('A'),
            l(pos),
        )
    }

    fn window(left: &Rotor, mid: &Rotor, right: &Rotor) -> String {
        [left.pos(), mid.pos(), right.pos()]
            .iter()
            .map(|p| char::from(*p))
            .collect()
    }

    #[test]
    fn middle_rotor_double_steps() {
        let mut left = rotor("EKMFLGDQVZNTOWYHXUSPAIBRCJ", 'Q', 'A'); // I
        let mut mid = rotor("AJDKSIRUXBLHWTMCQGZNPYFVOE", 'E', 'D'); // II
        let mut right = rotor("BDFHJLCPRTXVZNYEIWGAKMUSQO", 'V', 'U'); // III

        step_stack(&mut left, &mut mid, &mut right);
        assert_eq!(window(&left, &mid, &right), "ADV"); // right steps U→V
        step_stack(&mut left, &mut mid, &mut right);
        assert_eq!(window(&left, &mid, &right), "AEW"); // right at notch V → mid steps
        step_stack(&mut left, &mut mid, &mut right);
        assert_eq!(window(&left, &mid, &right), "BFX"); // mid at own notch E → double-step
    }

    #[test]
    fn right_rotor_steps_every_keypress() {
        let mut left = rotor("EKMFLGDQVZNTOWYHXUSPAIBRCJ", 'Q', 'A');
        let mut mid = rotor("AJDKSIRUXBLHWTMCQGZNPYFVOE", 'E', 'A');
        let mut right = rotor("BDFHJLCPRTXVZNYEIWGAKMUSQO", 'V', 'A');
        step_stack(&mut left, &mut mid, &mut right);
        assert_eq!(window(&left, &mid, &right), "AAB");
        step_stack(&mut left, &mut mid, &mut right);
        assert_eq!(window(&left, &mid, &right), "AAC");
    }
}


pub trait Scrambler {
    fn map(&self, l: Letter) -> Letter;
}

pub struct FixedScrambler([u8; 26]);

impl FixedScrambler {
    pub fn from_fn(mut f: impl FnMut(Letter) -> Letter) -> FixedScrambler {
        FixedScrambler(core::array::from_fn(|i| f(Letter::new(i as u8).unwrap()).index() as u8))
    }
}

impl Scrambler for FixedScrambler {
    fn map(&self, l: Letter) -> Letter {
        Letter(self.0[l.index()])
    }
}
