mod data;

use machine_core::{FixedScrambler, Letter, Plugboard, Reflector, Rotor, step_stack};

pub struct Enigma {
    greek: Option<Rotor>,
    left: Rotor,
    mid: Rotor,
    right: Rotor,
    reflector: Reflector,
    plugboard: Plugboard,
}

#[derive(Debug)]
pub enum EnigmaError {
    UnknownRotor(String),
    UnknownReflector(String),
    BadPositions(String),
    Wiring(machine_core::WiringError),
}

impl Enigma {
    pub fn builder() -> EnigmaBuilder {
        EnigmaBuilder::default()
    }

    pub fn m3(
        rotors: [&str; 3],
        reflector: &str,
        rings: &str,
        positions: &str,
        plugs: &str,
    ) -> Result<Enigma, EnigmaError> {
        Enigma::builder()
            .rotors(rotors)
            .reflector(reflector)
            .rings(rings)
            .positions(positions)
            .plugboard(plugs)
            .build()
    }

    pub fn m4(
        greek: &str,
        rotors: [&str; 3],
        reflector: &str,
        rings4: &str,
        positions4: &str,
        plugs: &str,
    ) -> Result<Enigma, EnigmaError> {
        if rings4.len() != 4 || positions4.len() != 4 {
            return Err(EnigmaError::BadPositions(format!("{rings4}/{positions4}")));
        }
        let (g_ring, rings3) = rings4.split_at(1);
        let (g_pos, poss3) = positions4.split_at(1);

        let mut m = Enigma::builder()
            .rotors(rotors)
            .reflector(reflector)
            .rings(rings3)
            .positions(poss3)
            .plugboard(plugs)
            .build()?;

        let g_ring = Letter::from_char(g_ring.chars().next().unwrap())
            .ok_or_else(|| EnigmaError::BadPositions(rings4.to_string()))?;
        let g_pos = Letter::from_char(g_pos.chars().next().unwrap())
            .ok_or_else(|| EnigmaError::BadPositions(positions4.to_string()))?;
        let wiring =
            data::wiring_of(greek).ok_or_else(|| EnigmaError::UnknownRotor(greek.to_string()))?;
        m.greek = Some(Rotor::new(wiring, data::notches_of(greek), g_ring, g_pos));
        Ok(m)
    }

    pub fn from_settings(s: EnigmaSettings) -> Result<Enigma, EnigmaError> {
        Enigma::builder()
            .rotors([&s.rotors[0], &s.rotors[1], &s.rotors[2]])
            .reflector(&s.reflector)
            .rings(&s.rings)
            .positions(&s.positions)
            .plugboard(&s.plugboard)
            .build()
    }

    pub fn scrambler_at(settings: &EnigmaSettings, k: usize) -> Result<FixedScrambler, EnigmaError> {
        let mut bare = settings.clone();
        bare.plugboard = String::new();
        let mut m = Enigma::from_settings(bare)?;
        for _ in 0..=k {
            step_stack(&mut m.left, &mut m.mid, &mut m.right);
        }
        Ok(FixedScrambler::from_fn(|l| m.scramble_no_step(l)))
    }

    pub fn encode_letter(&mut self, c: Letter) -> Letter {
        step_stack(&mut self.left, &mut self.mid, &mut self.right);
        let x = self.plugboard.map(c);
        let x = self.scramble_no_step(x);
        self.plugboard.map(x)
    }

    pub fn encode_char(&mut self, c: char) -> char {
        match Letter::from_char(c) {
            Some(l) => char::from(self.encode_letter(l)),
            None => c,
        }
    }

    pub fn greek_window(&self) -> char {
        self.greek.as_ref().map(|g| char::from(g.pos())).unwrap_or('-')
    }

    fn scramble_no_step(&self, c: Letter) -> Letter {
        let x = self.right.encode_forward(c);
        let x = self.mid.encode_forward(x);
        let x = self.left.encode_forward(x);
        let x = match &self.greek {
            Some(g) => {
                let x = g.encode_forward(x);
                let x = self.reflector.map(x);
                g.encode_backward(x)
            }
            None => self.reflector.map(x),
        };
        let x = self.left.encode_backward(x);
        let x = self.mid.encode_backward(x);
        self.right.encode_backward(x)
    }
}

#[derive(Default)]
pub struct EnigmaBuilder {
    rotors: [String; 3],
    reflector: String,
    rings: String,
    positions: String,
    plugboard: String,
}

impl EnigmaBuilder {
    pub fn rotors(mut self, names: [&str; 3]) -> Self {
        self.rotors = names.map(String::from);
        self
    }

    pub fn reflector(mut self, name: &str) -> Self {
        self.reflector = name.into();
        self
    }

    pub fn rings(mut self, name: &str) -> Self {
        self.rings = name.into();
        self
    }

    pub fn positions(mut self, name: &str) -> Self {
        self.positions = name.into();
        self
    }

    pub fn plugboard(mut self, name: &str) -> Self {
        self.plugboard = name.into();
        self
    }

    pub fn build(self) -> Result<Enigma, EnigmaError> {
        let rings = parse_triplet(&self.rings)?;
        let poss = parse_triplet(&self.positions)?;

        let make = |i: usize| -> Result<Rotor, EnigmaError> {
            let name = &self.rotors[i];
            let wiring =
                data::wiring_of(name).ok_or_else(|| EnigmaError::UnknownRotor(name.clone()))?;
            let notches = data::notches_of(name);
            Ok(Rotor::new(wiring, notches, rings[i], poss[i]))
        };

        Ok(Enigma {
            greek: None,
            left: make(0)?,
            mid: make(1)?,
            right: make(2)?,
            reflector: data::reflector_of(&self.reflector)
                .ok_or_else(|| EnigmaError::UnknownReflector(self.reflector.clone()))?,
            plugboard: Plugboard::parse(&self.plugboard).map_err(EnigmaError::Wiring)?,
        })
    }
}

fn parse_triplet(s: &str) -> Result<[Letter; 3], EnigmaError> {
    let v: Vec<Letter> = s.chars().filter_map(Letter::from_char).collect();
    if v.len() != 3 {
        return Err(EnigmaError::BadPositions(s.to_string()));
    }
    Ok([v[0], v[1], v[2]])
}

#[derive(Clone, Debug)]
pub struct EnigmaSettings {
    pub rotors: [String; 3],
    pub reflector: String,
    pub rings: String,
    pub positions: String,
    pub plugboard: String,
}

pub fn encrypt(settings: &EnigmaSettings, text: &str) -> Result<String, EnigmaError> {
    let mut m = Enigma::from_settings(settings.clone())?;
    Ok(text.chars().map(|c| m.encode_char(c)).collect())
}
