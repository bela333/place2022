use std::{fs::File, io::{BufReader, BufRead, Write}, time::Instant};

use flate2::read::GzDecoder;


#[derive(Debug)]
struct Color(u8, u8, u8);

impl Color{
    pub fn from_hex(hex: &str) -> Option<Color>{
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Color(r, g, b))
    }
}


#[derive(Debug)]
struct Entry{
    color: Color,
    pos: (u16, u16)
}

impl Entry{


    pub fn from_line(line: &str) -> Option<Self>{
        let mut parts = line.split(',').skip(2);
        let color = parts.next()?;
        let pos1 = parts.next()?;
        let pos2 = parts.next()?;

        let color = Color::from_hex(&color[1..])?;

        let x = pos1[1..].parse().ok()?;
        let y = pos2[..pos2.len()-1].parse().ok()?;
        let pos = (x, y);

        if parts.next().is_some() {
            return None;
        }

        Some(Self{
            color, pos
        })

    }
}

fn main() {

    let start = Instant::now();
    
    let f = File::open("../sorted.csv.gz").unwrap();
    let f = GzDecoder::new(f);
    let f = BufReader::new(f);
    println!("Reading file");
    //156353085: 160353085 total entries - 2000*2000 white pixels
    let entries = f.lines().flatten().map(|line|Entry::from_line(&line)).flatten()
        //.take(156353085);
        .take(1000000);


    let mut canvas = (0..2000*2000*3).map(|_|255u8).collect::<Vec<_>>();

    for entry in entries {
        let index = (entry.pos.0 as usize + entry.pos.1 as usize*2000)*3;
        canvas[index  ] = entry.color.0;
        canvas[index+1] = entry.color.1;
        canvas[index+2] = entry.color.2;
    }

    File::create("output.bin").unwrap().write_all(&canvas).unwrap();

    println!("Elapsed: {:?}", start.elapsed());

}
