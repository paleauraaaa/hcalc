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
    for (n, p) in state.firing_positions.iter() {
        let dist = distance(*p, c);
        let bearing = bearing(*p, c);
        let charge = Charge::for_distance(dist);
        let mils = charge.map(|c| c.quadratic().solve(dist.0 as f64));
        let mils_s = if let Some(m) = mils {
            format!("{:06.1}mrad ({})", m, charge.unwrap())
        } else {
            "OUT OF RANGE".to_string()
        };
        println!(
            "\t\"{}\": distance: {}, bearing: {}, mils: {}",
            n, dist, bearing, mils_s
        );
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
    for (n, p) in state.firing_positions.iter() {
        let dist = distance(*p, c);
        let bearing = bearing(*p, c);
        let charge = Charge::for_distance(dist);
        let mils = charge.map(|c| c.quadratic().solve(dist.0 as f64));
        let mils_s = if let Some(m) = mils {
            format!("{:06.1}mrad ({})", m, charge.unwrap())
        } else {
            "OUT OF RANGE".to_string()
        };
        println!(
            "\t\"{}\": distance: {}, bearing: {}, mils: {}",
            n, dist, bearing, mils_s
        );
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
        for (n, p) in state.firing_positions.iter() {
            let dist = distance(*p, f.coords);
            let bearing = bearing(*p, f.coords);
            let charge = Charge::for_distance(dist);
            let mils = charge.map(|c| c.quadratic().solve(dist.0 as f64));
            let mils_s = if let Some(m) = mils {
                format!("{:06.1}mrad", m)
            } else {
                "OUT OF RANGE".to_string()
            };
            println!(
                "\t\t\"{}\": distance: {}, bearing: {}, mils: {}",
                n, dist, bearing, mils_s
            );
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
