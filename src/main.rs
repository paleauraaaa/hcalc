use std::{collections::HashMap, fmt::Display, io::Write};

use nom::{
    IResult, Parser, bytes::complete::is_not, character::complete::char, sequence::delimited,
};

fn parens(input: &str) -> IResult<&str, &str> {
    delimited(char('('), is_not(")"), char(')')).parse(input)
}

fn quotes(input: &str) -> IResult<&str, &str> {
    delimited(char('"'), is_not("\""), char('"')).parse(input)
}

fn coords(input: &str) -> Result<Coords, ()> {
    let input = if let Ok(i) = parens(input).map(|p| p.1) {
        i
    } else {
        input
    };
    let (x, y) = input.split_once(',').or(input.split_once(' ')).ok_or(())?;
    let x = x.trim().parse::<i32>().unwrap();
    let y = y.trim().parse::<i32>().unwrap();
    Ok(Coords((x, y)))
}

enum Quadrant {
    NE,
    SE,
    SW,
    NW,
}

fn quadrant(origin: Coords, target: Coords) -> Quadrant {
    let (x1, y1) = origin.0;
    let (x2, y2) = target.0;
    match (x2 - x1 >= 0, y2 - y1 >= 0) {
        (true, true) => Quadrant::NE,
        (true, false) => Quadrant::SE,
        (false, false) => Quadrant::SW,
        (false, true) => Quadrant::NW,
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Bearing(f64);
impl Display for Bearing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:05.1}°", self.0)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Coords((i32, i32));
impl Display for Coords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0.0, self.0.1)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct FireMission {
    coords: Coords,
}

#[derive(Debug, Default, Clone)]
struct State {
    firing_positions: HashMap<String, Coords>,
    fire_missions: HashMap<String, FireMission>,
}

impl State {
    fn new() -> Self {
        Self {
            firing_positions: HashMap::new(),
            fire_missions: HashMap::new(),
        }
    }
}

fn print_fire_mission(state: &State, position: &str, mission: &str) -> Result<(), ()> {
    let p = state.firing_positions.get(position).ok_or(())?;
    let c = state.fire_missions.get(mission).ok_or(())?.coords;
    let dist = distance(*p, c);
    let bearing = bearing(*p, c);
    let charge = Charge::for_distance(dist);
    let mils_quad = charge.map(|c| c.quadratic().solve(dist.0 as f64));
    let mils_lut = charge.map(|c| c.lut().mils_linear(dist).unwrap_or(0.0));
    let mils_lut2 = charge.map(|c| c.lut().mils_quadratic(dist).unwrap_or(0.0));
    let mils_quad_s = if let Some(m) = mils_quad {
        format!("{:06.1}mrad", m)
    } else {
        "OUT OF RANGE".to_string()
    };
    let mils_lut_s = if let Some(m) = mils_lut {
        format!("{:06.1}mrad", m)
    } else {
        "OUT OF RANGE".to_string()
    };
    let mils_lut2_s = if let Some(m) = mils_lut2 {
        format!("{:06.1}mrad", m)
    } else {
        "OUT OF RANGE".to_string()
    };
    println!(
        "\t\"{}\": distance: {}, bearing: {}, mils: {} (quadratic), {} (LUT linear), {} (LUT lagrange), charge: {}",
        mission, dist, bearing, mils_quad_s, mils_lut_s, mils_lut2_s, charge.unwrap()
    );
    Ok(())
}

fn new(state: &mut State, input: &str) -> Result<(), ()> {
    let (name, c) = input.split_once(':').ok_or(())?;
    let name = name.trim().to_string();
    let name = if let Ok(n) = quotes(&name).map(|p| p.1.to_string()) {
        n
    } else {
        name
    };
    let c = c.trim();
    let c = coords(c)?;
    let f = FireMission { coords: c };
    state.fire_missions.insert(name.clone(), f);
    println!("Added fire mission \"{}\" at coordinates {}.", name, c);
    for (n, _) in state.firing_positions.iter() {
        print_fire_mission(state, n, &name)?;
    }
    Ok(())
}

fn delete(state: &mut State, input: &str) -> Result<(), ()> {
    let name = input.trim();
    state.fire_missions.remove(name);
    println!("Deleted fire mission \"{}\"", name);
    Ok(())
}

fn edit(state: &mut State, input: &str) -> Result<(), ()> {
    let (name, c) = input.split_once(':').ok_or(())?;
    let name = name.trim();
    if !state.fire_missions.contains_key(name) {
        return Err(());
    }
    let c = c.trim();
    let c = coords(c)?;
    let f = FireMission { coords: c };
    state.fire_missions.insert(name.to_string(), f);
    println!("Edited fire mission \"{}\" to coordinates {}.", name, c);
    for (n, _) in state.firing_positions.iter() {
        print_fire_mission(state, n, name)?;
    }
    Ok(())
}

fn add(state: &mut State, input: &str) -> Result<(), ()> {
    let (name, c) = input.split_once(':').ok_or(())?;
    let name = name.trim();
    let name = if let Ok(n) = quotes(name).map(|p| p.1) {
        n
    } else {
        name
    };
    let c = c.trim();
    let c = coords(c)?;
    state.firing_positions.insert(name.to_string(), c);
    println!("Added firing position \"{}\" at coordinates {}.", name, c);
    Ok(())
}

fn list(state: &State) {
    println!("Fire missions:");
    for (name, &f) in &state.fire_missions {
        println!("\t\"{}\": {}", name, f.coords);
        for (n, _) in state.firing_positions.iter() {
            print_fire_mission(state, n, name).unwrap();
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Distance(i32);
impl Display for Distance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}m", self.0)
    }
}

fn distance(a: Coords, b: Coords) -> Distance {
    let dx = (b.0.0 - a.0.0) as f64;
    let dy = (b.0.1 - a.0.1) as f64;
    Distance((dx * dx + dy * dy).sqrt() as i32)
}

fn bearing(a: Coords, b: Coords) -> Bearing {
    let dx = (b.0.0 - a.0.0) as f64;
    let dy = (b.0.1 - a.0.1) as f64;
    let off = match quadrant(a, b) {
        Quadrant::NE | Quadrant::SE => 0.0,
        Quadrant::NW | Quadrant::SW => 360.0,
    };
    Bearing(dx.atan2(dy).to_degrees() + off)
}

#[derive(Debug, Default, Clone, Copy)]
struct Quadratic {
    a: f64,
    b: f64,
    c: f64,
}

impl Quadratic {
    const fn solve(&self, x: f64) -> f64 {
        self.a * x * x + self.b * x + self.c
    }
}

static CHARGE_1_LUT: [(i32, i32); 12] = [
    (950, 1245),
    (1000, 1221),
    (1050, 1197),
    (1100, 1171),
    (1150, 1149),
    (1200, 1115),
    (1250, 1084),
    (1300, 1050),
    (1350, 1011),
    (1400, 965),
    (1450, 907),
    (1500, 800),
];

static CHARGE_2_LUT: [(i32, i32); 21] = [
    (1500, 1270),
    (1550, 1257),
    (1600, 1243),
    (1650, 1229),
    (1700, 1214),
    (1750, 1200),
    (1800, 1185),
    (1850, 1169),
    (1900, 1153),
    (1950, 1136),
    (2000, 1119),
    (2050, 1101),
    (2100, 1082),
    (2150, 1062),
    (2200, 1041),
    (2250, 1018),
    (2300, 995),
    (2350, 967),
    (2400, 938),
    (2450, 904),
    (2500, 860),
];

static CHARGE_4_LUT: [(i32, i32); 20] = [
    (2600, 1271),
    (2700, 1255),
    (2800, 1240),
    (2900, 1224),
    (3000, 1207),
    (3100, 1190),
    (3200, 1172),
    (3300, 1154),
    (3400, 1135),
    (3500, 1116),
    (3600, 1095),
    (3700, 1074),
    (3800, 1052),
    (3900, 1028),
    (4000, 1003),
    (4100, 976),
    (4200, 946),
    (4300, 912),
    (4400, 874),
    (4500, 828),
];

static CHARGE_5_LUT: [(i32, i32); 24] = [
    (3000, 1271),
    (3100, 1258),
    (3200, 1244),
    (3300, 1230),
    (3400, 1216),
    (3500, 1202),
    (3600, 1187),
    (3700, 1172),
    (3800, 1156),
    (3900, 1140),
    (4000, 1024),
    (4100, 1107),
    (4200, 1089),
    (4300, 1071),
    (4400, 1052),
    (4500, 1032),
    (4600, 1011),
    (4700, 989),
    (4800, 966),
    (4900, 941),
    (5000, 913),
    (5100, 883),
    (5200, 850),
    (5300, 809),
];

struct Lut {
    points: &'static [(i32, i32)],
    range_inc: i32,
}

impl Lut {
    fn mils_linear(&self, distance: Distance) -> Option<f64> {
        let rounded = (distance.0 / self.range_inc) * self.range_inc;
        let rem = distance.0 % self.range_inc;
        let next = rounded + self.range_inc;
        let low = self.points.iter().find(|(d, _)| *d == rounded)?;
        let high = self.points.iter().find(|(d, _)| *d == next)?;
        let meters = high.0 - low.0;
        let mils = low.1 - high.1;
        let mils_per_meter = mils as f64 / meters as f64;
        Some(low.1 as f64 - (rem as f64 * mils_per_meter))
    }

    fn mils_quadratic(&self, distance: Distance) -> Option<f64> {
        let rounded = (distance.0 / self.range_inc) * self.range_inc;
        let rem = distance.0 - rounded;
        let next = rounded + self.range_inc;
        let next2 = rounded + self.range_inc * 2;
        let prev = rounded - self.range_inc;
        let third = if rem > self.range_inc / 2 { next2 } else { prev };
        let a = self.points.iter().find(|(d, _)| *d == rounded)?;
        let b = self.points.iter().find(|(d, _)| *d == next)?;
        let c = self.points.iter().find(|(d, _)| *d == third)?;
        let points = [a, b, c];
        let mut sum = 0.0;
        for i in 0..3 {
            let mut prod = 1.0;
            for j in 0..3 {
                if i == j {
                    continue;
                }

                prod *= (distance.0 as f64 - points[j].0 as f64) / (points[i].0 as f64 - points[j].0 as f64);
                
            }
            sum += prod * points[i].1 as f64;
        }
        Some(sum)
    }
}

#[derive(Debug, Clone, Copy)]
enum Charge {
    One,
    Two,
    Three,
    Four,
    Five,
}

impl Display for Charge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Charge::One => write!(f, "Charge 1"),
            Charge::Two => write!(f, "Charge 2"),
            Charge::Three => write!(f, "Charge 3"),
            Charge::Four => write!(f, "Charge 4"),
            Charge::Five => write!(f, "Charge 5"),
        }
    }
}

impl Charge {
    const fn for_distance(distance: Distance) -> Option<Self> {
        if distance.0 < 950 {
            None
        } else if distance.0 < 1500 {
            Some(Charge::One)
        } else if distance.0 <= 2500 {
            Some(Charge::Two)
        } else if distance.0 <= 4500 {
            Some(Charge::Four)
        } else if distance.0 <= 5300 {
            Some(Charge::Five)
        } else {
            None
        }
    }

    const fn quadratic(&self) -> Quadratic {
        match self {
            Charge::One => Quadratic {
                a: -9.84E-04,
                b: 1.68,
                c: 525.0,
            },
            Charge::Two => Quadratic {
                a: -2.05E-04,
                b: 0.435,
                c: 1072.0,
            },
            Charge::Three => Quadratic {
                a: -1.04E-04,
                b: 0.313,
                c: 1065.0,
            },
            Charge::Four => Quadratic {
                a: -0.0000601618,
                b: 0.206291,
                c: 1133.45796,
            },
            Charge::Five => Quadratic {
                a: -4.2E-05,
                b: 0.159,
                c: 1165.0,
            },
        }
    }

    const fn lut(&self) -> Lut {
        match self {
            Charge::One => Lut { points: &CHARGE_1_LUT, range_inc: 50 },
            Charge::Two => Lut { points: &CHARGE_2_LUT, range_inc: 50 },
            Charge::Three => unimplemented!(),
            Charge::Four => Lut { points: &CHARGE_4_LUT, range_inc: 100 },
            Charge::Five => Lut { points: &CHARGE_5_LUT, range_inc: 100 },
        }
    }
}

fn session() -> Result<bool, ()> {
    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();
    let mut state = State::new();

    let mut input = String::new();
    loop {
        print!("> ");
        stdout.flush().unwrap();

        stdin.read_line(&mut input).unwrap();
        let i = input.trim();
        if i.is_empty() {
            continue;
        }

        let (cmd, i) = i.split_once(' ').unwrap_or((i, ""));
        match cmd.to_lowercase().as_str() {
            "new" | "n" => {
                if let Err(_) = new(&mut state, i) {
                    eprintln!("Invalid input for new fire mission. Format: n[ew] <name>: <coords>");
                }
            }
            "delete" | "d" => {
                if let Err(_) = delete(&mut state, i) {
                    eprintln!("Invalid input for delete fire mission. Format: d[elete] <name>");
                }
            }
            "edit" | "e" => {
                if let Err(_) = edit(&mut state, i) {
                    eprintln!(
                        "Invalid input for edit fire mission. Format: e[dit] <name>: <coords>"
                    );
                }
            }
            "add" | "a" => {
                if let Err(_) = add(&mut state, i) {
                    eprintln!("Invalid input for add fire mission. Format: a[dd] <name>: <coords>");
                }
            }
            "list" | "l" => list(&state),
            "reset" | "r" => {
                println!("");
                return Ok(true);
            }
            "quit" | "q" => return Ok(false),
            _ => eprintln!("Unknown command: {}", cmd),
        };
        input.clear();
    }
}

fn main() {
    while let Ok(restart) = session() {
        if !restart {
            break;
        }
    }
}
