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




fn window(arr: &[u8], buf: &mut [u8], x: isize, y: isize) {
    assert_eq!(buf.len() as isize, SIZE*SIZE*3);
    let (starty, dy) = if y < 0 {
        (0, -y as usize)
    }else{(y as usize, 0)};
    let endy = y+SIZE;
    let endy = if endy >= IMAGE_SIZE {
        IMAGE_SIZE
    }else{endy} as usize;

    let (startx, dx) = if x < 0 {
        (0, -x as usize)
    }else{(x as usize, 0)};
    let endx = x+SIZE;
    let endx = if endx >= IMAGE_SIZE {
        IMAGE_SIZE
    }else{endx} as usize;

    let height = endy-starty;
    let width = endx-startx;

    for _y in 0..height {
        let y = starty+_y;
        let dy = dy+_y;
        let from_line = &arr[y*IMAGE_SIZEU*3..(y+1)*IMAGE_SIZEU*3];
        let to_line = &mut buf[dy*SIZEU*3..(dy+1)*SIZEU*3];
        let x = startx;
        let dx = dx;
        let from_line = &from_line[x*3..(x+width)*3];
        let to_line = &mut to_line[dx*3..(dx+width)*3];
        to_line.copy_from_slice(from_line);
    }

}

const IMAGE_SIZE: isize = 2000;
const IMAGE_SIZEU: usize = IMAGE_SIZE as usize;

const MARGIN: isize = 50;
const SIZE: isize = MARGIN*2+1;
const SIZEU: usize = SIZE as usize;

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


    let mut canvas = (0..IMAGE_SIZE*IMAGE_SIZE*3).map(|_|255u8).collect::<Vec<_>>();
    
    for entry in entries {
        let (x, y) = entry.pos;
        let index = (entry.pos.0 as usize + entry.pos.1 as usize*IMAGE_SIZE as usize)*3;
        canvas[index  ] = entry.color.0;
        canvas[index+1] = entry.color.1;
        canvas[index+2] = entry.color.2;
        
        
        let mut o = [255u8;(SIZE*SIZE*3) as usize];
        window(&canvas, &mut o, x as isize-MARGIN, y as isize-MARGIN);
    }
    

    println!("Elapsed: {:?}", start.elapsed());

}
