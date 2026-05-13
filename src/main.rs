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
    let x = x.trim().parse::<i32>().map_err(|_| ())?;
    let y = y.trim().parse::<i32>().map_err(|_| ())?;
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
    let charge = Charge::for_distance(dist).unwrap_or(Charge::Five);
    let mils_quad = charge.quadratic().solve(dist.0 as f64);
    let mils_quad_s = format!("{:06.1}mrad", mils_quad);
    let mils_lut_linear = charge.lut().mils_linear(dist);
    let mils_lut_quadratic = charge.lut().mils_quadratic(dist);
    let time_of_flight = charge.lut().time_of_flight(dist);

    let mils_lut_s = if let Some(m) = mils_lut_linear {
        format!("{:06.1}mrad", m)
    } else {
        "OUT OF RANGE".to_string()
    };
    let mils_lut2_s = if let Some(m) = mils_lut_quadratic {
        format!("{:06.1}mrad", m)
    } else {
        "OUT OF RANGE".to_string()
    };
    let time_of_flight = if let Some(t) = time_of_flight {
        format!("{:.1}s", t)
    } else {
        "OUT OF RANGE".to_string()
    };
    println!(
        "\t{} - distance: {}, bearing: {}, elevation: {} (quadratic), {} (LUT linear), {} (LUT lagrange), charge: {}, time of flight: {}",
        mission, dist, bearing, mils_quad_s, mils_lut_s, mils_lut2_s, charge, time_of_flight
    );
    Ok(())
}

fn new(state: &mut State, input: &str) -> Result<(), ()> {
    let (input, delim) = if input.starts_with('"') {
        (&input[1..], '"')
    } else {
        (input, ' ')
    };
    let (name, c) = input.split_once(delim).ok_or(())?;
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
    let (input, delim) = if input.starts_with('"') {
        (&input[1..], '"')
    } else {
        (input, ' ')
    };
    let (name, c) = input.split_once(delim).ok_or(())?;
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
    let (input, delim) = if input.starts_with('"') {
        (&input[1..], '"')
    } else {
        (input, ' ')
    };
    let (name, c) = input.split_once(delim).ok_or(())?;
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

struct ChargeInfo {
    range: i32,
    elevation: i32,
    time_of_flight: f64,
}

static CHARGE_1_LUT: [ChargeInfo; 12] = [
    ChargeInfo {
        range: 950,
        elevation: 1245,
        time_of_flight: 24.4,
    },
    ChargeInfo {
        range: 1000,
        elevation: 1221,
        time_of_flight: 24.2,
    },
    ChargeInfo {
        range: 1050,
        elevation: 1197,
        time_of_flight: 24.0,
    },
    ChargeInfo {
        range: 1100,
        elevation: 1171,
        time_of_flight: 23.7,
    },
    ChargeInfo {
        range: 1150,
        elevation: 1149,
        time_of_flight: 23.4,
    },
    ChargeInfo {
        range: 1200,
        elevation: 1115,
        time_of_flight: 23.1,
    },
    ChargeInfo {
        range: 1250,
        elevation: 1084,
        time_of_flight: 22.7,
    },
    ChargeInfo {
        range: 1300,
        elevation: 1050,
        time_of_flight: 22.3,
    },
    ChargeInfo {
        range: 1350,
        elevation: 1011,
        time_of_flight: 21.8,
    },
    ChargeInfo {
        range: 1400,
        elevation: 965,
        time_of_flight: 21.2,
    },
    ChargeInfo {
        range: 1450,
        elevation: 907,
        time_of_flight: 20.3,
    },
    ChargeInfo {
        range: 1500,
        elevation: 800,
        time_of_flight: 18.6,
    },
];

static CHARGE_2_LUT: [ChargeInfo; 21] = [
    ChargeInfo {
        range: 1500,
        elevation: 1270,
        time_of_flight: 31.1,
    },
    ChargeInfo {
        range: 1550,
        elevation: 1257,
        time_of_flight: 33.0,
    },
    ChargeInfo {
        range: 1600,
        elevation: 1243,
        time_of_flight: 32.9,
    },
    ChargeInfo {
        range: 1650,
        elevation: 1229,
        time_of_flight: 32.7,
    },
    ChargeInfo {
        range: 1700,
        elevation: 1214,
        time_of_flight: 32.5,
    },
    ChargeInfo {
        range: 1750,
        elevation: 1200,
        time_of_flight: 32.4,
    },
    ChargeInfo {
        range: 1800,
        elevation: 1185,
        time_of_flight: 32.1,
    },
    ChargeInfo {
        range: 1850,
        elevation: 1169,
        time_of_flight: 31.9,
    },
    ChargeInfo {
        range: 1900,
        elevation: 1153,
        time_of_flight: 31.7,
    },
    ChargeInfo {
        range: 1950,
        elevation: 1136,
        time_of_flight: 31.5,
    },
    ChargeInfo {
        range: 2000,
        elevation: 1119,
        time_of_flight: 31.2,
    },
    ChargeInfo {
        range: 2050,
        elevation: 1101,
        time_of_flight: 31.0,
    },
    ChargeInfo {
        range: 2100,
        elevation: 1082,
        time_of_flight: 30.7,
    },
    ChargeInfo {
        range: 2150,
        elevation: 1062,
        time_of_flight: 30.3,
    },
    ChargeInfo {
        range: 2200,
        elevation: 1041,
        time_of_flight: 30.0,
    },
    ChargeInfo {
        range: 2250,
        elevation: 1018,
        time_of_flight: 29.6,
    },
    ChargeInfo {
        range: 2300,
        elevation: 995,
        time_of_flight: 29.2,
    },
    ChargeInfo {
        range: 2350,
        elevation: 967,
        time_of_flight: 28.7,
    },
    ChargeInfo {
        range: 2400,
        elevation: 938,
        time_of_flight: 28.1,
    },
    ChargeInfo {
        range: 2450,
        elevation: 904,
        time_of_flight: 27.5,
    },
    ChargeInfo {
        range: 2500,
        elevation: 860,
        time_of_flight: 26.5,
    },
];

static CHARGE_4_LUT: [ChargeInfo; 20] = [
    ChargeInfo {
        range: 2600,
        elevation: 1271,
        time_of_flight: 47.2,
    },
    ChargeInfo {
        range: 2700,
        elevation: 1255,
        time_of_flight: 47.0,
    },
    ChargeInfo {
        range: 2800,
        elevation: 1240,
        time_of_flight: 46.7,
    },
    ChargeInfo {
        range: 2900,
        elevation: 1224,
        time_of_flight: 46.5,
    },
    ChargeInfo {
        range: 3000,
        elevation: 1207,
        time_of_flight: 46.2,
    },
    ChargeInfo {
        range: 3100,
        elevation: 1190,
        time_of_flight: 45.9,
    },
    ChargeInfo {
        range: 3200,
        elevation: 1172,
        time_of_flight: 45.6,
    },
    ChargeInfo {
        range: 3300,
        elevation: 1154,
        time_of_flight: 45.2,
    },
    ChargeInfo {
        range: 3400,
        elevation: 1135,
        time_of_flight: 44.9,
    },
    ChargeInfo {
        range: 3500,
        elevation: 1116,
        time_of_flight: 44.5,
    },
    ChargeInfo {
        range: 3600,
        elevation: 1095,
        time_of_flight: 44.0,
    },
    ChargeInfo {
        range: 3700,
        elevation: 1074,
        time_of_flight: 43.6,
    },
    ChargeInfo {
        range: 3800,
        elevation: 1052,
        time_of_flight: 43.1,
    },
    ChargeInfo {
        range: 3900,
        elevation: 1028,
        time_of_flight: 42.5,
    },
    ChargeInfo {
        range: 4000,
        elevation: 1003,
        time_of_flight: 41.9,
    },
    ChargeInfo {
        range: 4100,
        elevation: 976,
        time_of_flight: 41.3,
    },
    ChargeInfo {
        range: 4200,
        elevation: 946,
        time_of_flight: 40.5,
    },
    ChargeInfo {
        range: 4300,
        elevation: 912,
        time_of_flight: 39.6,
    },
    ChargeInfo {
        range: 4400,
        elevation: 874,
        time_of_flight: 38.5,
    },
    ChargeInfo {
        range: 4500,
        elevation: 828,
        time_of_flight: 37.2,
    },
];

static CHARGE_5_LUT: [ChargeInfo; 24] = [
    ChargeInfo {
        range: 3000,
        elevation: 1271,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 3100,
        elevation: 1258,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 3200,
        elevation: 1244,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 3300,
        elevation: 1230,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 3400,
        elevation: 1216,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 3500,
        elevation: 1202,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 3600,
        elevation: 1187,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 3700,
        elevation: 1172,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 3800,
        elevation: 1156,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 3900,
        elevation: 1140,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 4000,
        elevation: 1024,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 4100,
        elevation: 1107,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 4200,
        elevation: 1089,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 4300,
        elevation: 1071,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 4400,
        elevation: 1052,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 4500,
        elevation: 1032,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 4600,
        elevation: 1011,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 4700,
        elevation: 989,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 4800,
        elevation: 966,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 4900,
        elevation: 941,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 5000,
        elevation: 913,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 5100,
        elevation: 883,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 5200,
        elevation: 850,
        time_of_flight: 0.0,
    },
    ChargeInfo {
        range: 5300,
        elevation: 809,
        time_of_flight: 0.0,
    },
];

struct Lut {
    points: &'static [ChargeInfo],
    range_inc: i32,
}

impl Lut {
    fn mils_linear(&self, distance: Distance) -> Option<f64> {
        let rounded = (distance.0 / self.range_inc) * self.range_inc;
        let rem = distance.0 % self.range_inc;
        let next = rounded + self.range_inc;
        let low = self.points.iter().find(|c| c.range == rounded)?;
        let high = self.points.iter().find(|c| c.range == next)?;
        let meters = high.range - low.range;
        let mils = low.elevation - high.elevation;
        let mils_per_meter = mils as f64 / meters as f64;
        Some(low.elevation as f64 - (rem as f64 * mils_per_meter))
    }

    fn mils_quadratic(&self, distance: Distance) -> Option<f64> {
        let rounded = (distance.0 / self.range_inc) * self.range_inc;
        let rem = distance.0 - rounded;
        let next = rounded + self.range_inc;
        let next2 = rounded + self.range_inc * 2;
        let prev = rounded - self.range_inc;
        let third = if rem > self.range_inc / 2 {
            next2
        } else {
            prev
        };
        let a = self.points.iter().find(|c| c.range == rounded)?;
        let b = self.points.iter().find(|c| c.range == next)?;
        let c = self.points.iter().find(|c| c.range == third)?;
        let points = [a, b, c];
        let mut sum = 0.0;
        for i in 0..3 {
            let mut prod = 1.0;
            for j in 0..3 {
                if i == j {
                    continue;
                }

                prod *= (distance.0 as f64 - points[j].range as f64)
                    / (points[i].range as f64 - points[j].range as f64);
            }
            sum += prod * points[i].elevation as f64;
        }
        Some(sum)
    }

    fn time_of_flight(&self, distance: Distance) -> Option<f64> {
        let rounded = (distance.0 / self.range_inc) * self.range_inc;
        let rem = distance.0 % self.range_inc;
        let next = rounded + self.range_inc;
        let low = self.points.iter().find(|c| c.range == rounded)?;
        let high = self.points.iter().find(|c| c.range == next)?;
        let meters = high.range - low.range;
        let seconds = low.time_of_flight - high.time_of_flight;
        let seconds_per_meter = seconds / meters as f64;
        Some(low.time_of_flight - (rem as f64 * seconds_per_meter))
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
                a: -6.01618E-05,
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
            Charge::One => Lut {
                points: &CHARGE_1_LUT,
                range_inc: 50,
            },
            Charge::Two => Lut {
                points: &CHARGE_2_LUT,
                range_inc: 50,
            },
            Charge::Three => unimplemented!(),
            Charge::Four => Lut {
                points: &CHARGE_4_LUT,
                range_inc: 100,
            },
            Charge::Five => Lut {
                points: &CHARGE_5_LUT,
                range_inc: 100,
            },
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
                    eprintln!("Invalid input for new fire mission. Format: n[ew] <name> <coords>");
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
                        "Invalid input for edit fire mission. Format: e[dit] <name> <coords>"
                    );
                }
            }
            "add" | "a" => {
                if let Err(_) = add(&mut state, i) {
                    eprintln!("Invalid input for add fire mission. Format: a[dd] <name> <coords>");
                }
            }
            "list" | "l" => list(&state),
            "reset" | "r" => {
                println!("");
                return Ok(true);
            }
            "quit" | "q" => return Ok(false),
            "help" | "h" => {
                println!("Commands:");
                println!("\tn[ew] <name> <coords>: Create a new fire mission with the given name and coordinates.");
                println!("\td[elete] <name>: Delete the fire mission with the given name.");
                println!("\te[dit] <name> <coords>: Edit the fire mission with the given name to have the new coordinates.");
                println!("\ta[dd] <name> <coords>: Add a firing position with the given name and coordinates.");
                println!("\tl[ist]: List all fire missions and firing positions.");
                println!("\tr[eset]: Clear all fire missions and firing positions.");
                println!("\tq[uit]: Exit the program.");
                println!("\th[elp]: Show this help message.");
            }
            _ => eprintln!(
                "Unknown command: {}\nValid commands: new, delete, edit, add, list, reset, quit",
                cmd
            ),
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
