use alloc::vec::Vec;
use machine_core::{Letter, Reflector, Wiring};

// Historical Enigma wirings (I–VIII, Beta, Gamma). Notches given as the window letters.
pub const ROTORS: &[(&str, &str, &str)] = &[
    // (name, wiring, notch letters)
    ("I", "EKMFLGDQVZNTOWYHXUSPAIBRCJ", "Q"),
    ("II", "AJDKSIRUXBLHWTMCQGZNPYFVOE", "E"),
    ("III", "BDFHJLCPRTXVZNYEIWGAKMUSQO", "V"),
    ("IV", "ESOVPZJAYQUIRHXLNFTGKDCMWB", "J"),
    ("V", "VZBRGITYUPSDNHLXAWMJQOFECK", "Z"),
    ("VI", "JPGVOUMFYQBENHZRDKASXLICTW", "ZM"), // two notches
    ("VII", "NZJHGRCXMYSWBOUFAIVLPEKQDT", "ZM"),
    ("VIII", "FKQHTLXOCBJSPDZRAMEWNIUYGV", "ZM"),
    ("Beta", "LEYJVCNIXWPBQMDRTAKZGFUHOS", ""), // Greek rotors: no notch, never step
    ("Gamma", "FSOKANUERHMBTIYCWLQPZXVGJD", ""),
];

pub const REFLECTORS: &[(&str, &str)] = &[
    ("B", "YRUHQSLDPXNGOKMIEBFZCWVJAT"),
    ("C", "FVPJIAOYEDRZXWGCTKUQSBNMHL"),
    ("B-thin", "ENKQAUYWJICOPBLMDXZVFTHRGS"),
    ("C-thin", "RDOBJNTKVEHMLFCWZAXGYIPSUQ"),
];

pub fn wiring_of(name: &str) -> Option<Wiring> {
    let (_, w, _) = ROTORS.iter().find(|(n, _, _)| *n == name)?;
    Wiring::from_alphabet_string(w).ok()
}

pub fn notches_of(name: &str) -> Vec<Letter> {
    match ROTORS.iter().find(|(n, _, _)| *n == name) {
        Some((_, _, notch)) => notch.chars().filter_map(Letter::from_char).collect(),
        None => Vec::new(),
    }
}

pub fn reflector_of(name: &str) -> Option<Reflector> {
    let (_, s) = REFLECTORS.iter().find(|(n, _)| *n == name)?;
    Reflector::from_alphabet_string(s).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_rotor_wiring_is_a_valid_permutation() {
        for (name, s, _) in ROTORS {
            Wiring::from_alphabet_string(s).unwrap_or_else(|e| panic!("rotor {name}: {e:?}"));
        }
    }

    #[test]
    fn every_reflector_is_a_fixed_point_free_involution() {
        for (name, s) in REFLECTORS {
            Reflector::from_alphabet_string(s)
                .unwrap_or_else(|e| panic!("reflector {name}: {e:?}"));
        }
    }

    #[test]
    fn double_notch_rotors_have_two_turnovers() {
        for name in ["VI", "VII", "VIII"] {
            assert_eq!(notches_of(name).len(), 2, "{name} must notch at Z and M");
        }
        assert_eq!(notches_of("I").len(), 1);
        assert_eq!(notches_of("Beta").len(), 0);
    }
}
