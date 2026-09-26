use enigma::{Enigma, EnigmaSettings};
use machine_core::{Letter, Plugboard, Scrambler};

fn l(c: char) -> Letter {
    Letter::from_char(c).unwrap()
}

fn settings_no_plugs() -> EnigmaSettings {
    EnigmaSettings {
        rotors: ["I".into(), "II".into(), "III".into()],
        reflector: "B".into(),
        rings: "BGK".into(),
        positions: "QEV".into(),
        plugboard: "".into(),
    }
}

fn settings_with_plugs() -> EnigmaSettings {
    EnigmaSettings {
        rotors: ["II".into(), "IV".into(), "V".into()],
        reflector: "B".into(),
        rings: "BUL".into(),
        positions: "ABL".into(),
        plugboard: "AV BS CG".into(),
    }
}

#[test]
fn bdzgo_known_answer() {
    let mut m = Enigma::builder()
        .rotors(["I", "II", "III"])
        .reflector("B")
        .rings("AAA")
        .positions("AAA")
        .plugboard("")
        .build()
        .unwrap();
    let out: String = "AAAAAAAAAAAAAAAAAAAAAAAAA"
        .chars()
        .map(|c| m.encode_char(c))
        .collect();
    assert_eq!(&out[..5], "BDZGO");
    assert_eq!(out, "BDZGOWCXLTKSBTMCDLPBMUQOF");
}

#[test]
fn ring_setting_changes_the_output() {
    let run = |rings: &str| {
        let s = EnigmaSettings {
            rotors: ["I".into(), "II".into(), "III".into()],
            reflector: "B".into(),
            rings: rings.into(),
            positions: "AAA".into(),
            plugboard: "".into(),
        };
        enigma::encrypt(&s, "AAAAAAAAAAAAAAAAAAAA").unwrap()
    };
    assert_ne!(run("AAA"), run("BBB"));
}

#[test]
fn greek_rotor_never_steps() {
    let mut m = Enigma::m4("Beta", ["I", "II", "III"], "B-thin", "AAAA", "MAAA", "").unwrap();
    for _ in 0..100 {
        m.encode_char('A');
    }
    assert_eq!(m.greek_window(), 'M');
}

#[test]
fn greek_rotor_participates() {
    let enc = |greek_pos: &str| {
        let mut m = Enigma::m4(
            "Beta",
            ["I", "II", "III"],
            "B-thin",
            "AAAA",
            &format!("{greek_pos}AAA"),
            "",
        )
        .unwrap();
        (0..10).map(|_| m.encode_char('A')).collect::<String>()
    };
    assert_ne!(enc("A"), enc("B"));
}

#[test]
fn m4_reduces_to_m3_when_greek_neutralized() {
    let plain = "ENIGMAREVEALSITSELF";
    let m3_out = {
        let mut m = Enigma::m3(["I", "II", "III"], "B", "AAA", "AAA", "").unwrap();
        plain.chars().map(|c| m.encode_char(c)).collect::<String>()
    };
    let m4_out = {
        let mut m = Enigma::m4("Beta", ["I", "II", "III"], "B-thin", "AAAA", "AAAA", "").unwrap();
        plain.chars().map(|c| m.encode_char(c)).collect::<String>()
    };
    assert_eq!(m3_out, m4_out);
}

#[test]
fn scrambler_without_plugboard_is_a_fixed_point_free_involution() {
    let s = Enigma::scrambler_at(&settings_no_plugs(), 0).unwrap();
    for c in 'A'..='Z' {
        assert_ne!(s.map(l(c)), l(c));
        assert_eq!(s.map(s.map(l(c))), l(c));
    }
}

#[test]
fn plugboard_is_a_separable_layer() {
    let settings = settings_with_plugs();
    let s = Enigma::scrambler_at(&settings, 0).unwrap();
    let pb = Plugboard::parse(&settings.plugboard).unwrap();
    for c in 'A'..='Z' {
        let full = {
            let mut m = Enigma::from_settings(settings.clone()).unwrap();
            m.encode_letter(l(c))
        };
        assert_eq!(full, pb.map(s.map(pb.map(l(c)))));
    }
}

#[test]
fn scrambler_offset_matches_the_stepped_machine() {
    let settings = settings_no_plugs();
    let mut m = Enigma::from_settings(settings.clone()).unwrap();
    for k in 0..30usize {
        let s = Enigma::scrambler_at(&settings, k).unwrap();
        assert_eq!(m.encode_letter(l('A')), s.map(l('A')), "offset {k}");
    }
}
