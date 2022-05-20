use std::{
    cell::RefCell,
    fs::File,
    io::{BufRead, BufReader},
    time::Instant,
};

use flate2::read::GzDecoder;
use rand::{
    prelude::{IteratorRandom, SliceRandom},
    Rng,
};
use tfrecord::{ExampleWriter, Feature, RecordWriter};

#[derive(Debug)]
struct Color(f32, f32, f32, u8);

impl Color {
    pub fn from_hex(hex: &str) -> Option<Color> {
        let palette_index = match hex {
            "7EED56" => 0,
            "BE0039" => 1,
            "FF3881" => 2,
            "515252" => 3,
            "00CCC0" => 4,
            "FFD635" => 5,
            "811E9F" => 6,
            "FFA800" => 7,
            "D4D7D9" => 8,
            "DE107F" => 9,
            "FFB470" => 10,
            "94B3FF" => 11,
            "FFF8B8" => 12,
            "6D001A" => 13,
            "00756F" => 14,
            "3690EA" => 15,
            "B44AC0" => 16,
            "FF99AA" => 17,
            "FFFFFF" => 18,
            "6A5CFF" => 19,
            "898D90" => 20,
            "00A368" => 21,
            "9C6926" => 22,
            "6D482F" => 23,
            "000000" => 24,
            "FF4500" => 25,
            "51E9F4" => 26,
            "493AC1" => 27,
            "009EAA" => 28,
            "E4ABFF" => 29,
            "2450A4" => 30,
            "00CC78" => 31,
            _ => return None,
        };
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        Some(Color(r, g, b, palette_index))
    }
}

#[derive(Debug)]
struct Entry {
    color: Color,
    pos: (u16, u16),
}

impl Entry {
    pub fn from_line(line: &str) -> Option<Self> {
        let mut parts = line.split(',').skip(2);
        let color = parts.next()?;
        let pos1 = parts.next()?;
        let pos2 = parts.next()?;

        let color = Color::from_hex(&color[1..])?;

        let x = pos1[1..].parse().ok()?;
        let y = pos2[..pos2.len() - 1].parse().ok()?;
        let pos = (x, y);

        if parts.next().is_some() {
            return None;
        }

        Some(Self { color, pos })
    }
}

fn window<T: Copy>(arr: &[T], buf: &mut [T], x: isize, y: isize) {
    assert_eq!(buf.len() as isize, SIZE * SIZE * 3);
    let (starty, dy) = if y < 0 {
        (0, -y as usize)
    } else {
        (y as usize, 0)
    };
    let endy = y + SIZE;
    let endy = if endy >= IMAGE_SIZE { IMAGE_SIZE } else { endy } as usize;

    let (startx, dx) = if x < 0 {
        (0, -x as usize)
    } else {
        (x as usize, 0)
    };
    let endx = x + SIZE;
    let endx = if endx >= IMAGE_SIZE { IMAGE_SIZE } else { endx } as usize;

    let height = endy - starty;
    let width = endx - startx;

    for _y in 0..height {
        let y = starty + _y;
        let dy = dy + _y;
        let from_line = &arr[y * IMAGE_SIZEU * 3..(y + 1) * IMAGE_SIZEU * 3];
        let to_line = &mut buf[dy * SIZEU * 3..(dy + 1) * SIZEU * 3];

        let from_line = &from_line[startx * 3..(startx + width) * 3];
        let to_line = &mut to_line[dx * 3..(dx + width) * 3];
        to_line.copy_from_slice(from_line);
    }
}

const IMAGE_SIZE: isize = 2000;
const IMAGE_SIZEU: usize = IMAGE_SIZE as usize;

const MARGIN: isize = 50;
const MARGINU: usize = MARGIN as usize;
const SIZE: isize = MARGIN * 2 + 1;
const SIZEU: usize = SIZE as usize;

const OUTPUT: usize = 64000;

struct Example {
    image: [f32; SIZEU * SIZEU * 3],
    palette_index: i64,
}

fn choose_multiple<R, I: Iterator + Sized, O, F: FnMut(I::Item) -> O>(
    mut iterator: I,
    rng: &mut R,
    amount: usize,
    mut map: F,
) -> Vec<O>
where
    R: Rng + ?Sized,
{
    let mut reservoir = Vec::with_capacity(amount);
    reservoir.extend(iterator.by_ref().take(amount).map(|a| map(a)));

    // Continue unless the iterator was exhausted
    //
    // note: this prevents iterators that "restart" from causing problems.
    // If the iterator stops once, then so do we.
    if reservoir.len() == amount {
        for (i, elem) in iterator.enumerate() {
            let k = gen_index(rng, i + 1 + amount);
            if let Some(slot) = reservoir.get_mut(k) {
                *slot = map(elem);
            }
        }
    } else {
        // Don't hang onto extra memory. There is a corner case where
        // `amount` was much less than `self.len()`.
        reservoir.shrink_to_fit();
    }
    reservoir
}

fn gen_index<R: Rng + ?Sized>(rng: &mut R, ubound: usize) -> usize {
    if ubound <= (core::u32::MAX as usize) {
        rng.gen_range(0..ubound as u32) as usize
    } else {
        rng.gen_range(0..ubound)
    }
}

fn main() {
    let f = File::open("../sorted.csv.gz").unwrap();
    let f = GzDecoder::new(f);
    let f = BufReader::new(f);
    println!("Reading file");

    let take_size = 156353085;
    //let take_size = 1000000;

    //156353085: 160353085 total entries - 2000*2000 white pixels
    let entries = f
        .lines()
        .flatten()
        .map(|line| Entry::from_line(&line))
        .flatten()
        .take(take_size);

    let mut canvas = (0..IMAGE_SIZE * IMAGE_SIZE * 3)
        .map(|_| 1f32)
        .collect::<Vec<_>>();
    let canvas = RefCell::new(canvas);

    let mut writer: ExampleWriter<_> = RecordWriter::create("dataset.tfrecord").unwrap();

    let start = Instant::now();

    let mut i = 0;
    let example_iterator = entries.map(|entry| {
        i += 1;
        if i % (take_size / 1000) == 0 {
            let elapsed = start.elapsed();
            let fract = i as f32 / take_size as f32;
            let eta = elapsed.mul_f32((1.0 - fract) / fract);
            println!(
                "Decoding {}/{}. {:.2}% Elapsed: {:?} ETA: {:?}",
                i,
                take_size,
                fract * 100.0,
                elapsed,
                eta
            )
        }
        let (x, y) = entry.pos;

        let mut canvas = canvas.borrow_mut();
        let index = (entry.pos.0 as usize + entry.pos.1 as usize * IMAGE_SIZE as usize) * 3;
        let r = canvas[index];
        let g = canvas[index + 1];
        let b = canvas[index + 2];
        canvas[index] = entry.color.0;
        canvas[index + 1] = entry.color.1;
        canvas[index + 2] = entry.color.2;

        (x, y, r, g, b, entry.color.3)
    });

    let mut rng = rand::thread_rng();
    let mut examples = choose_multiple(
        example_iterator,
        &mut rng,
        OUTPUT,
        |(x, y, r, g, b, palette_index)| {
            let canvas = canvas.borrow();
            let mut o = [1f32; SIZEU * SIZEU * 3];
            window(&canvas, &mut o, x as isize - MARGIN, y as isize - MARGIN);
            let index = (SIZEU * MARGINU + MARGINU) * 3;
            o[index] = r;
            o[index + 1] = g;
            o[index + 2] = b;
            Example {
                image: o,
                palette_index: palette_index as i64,
            }
        },
    );

    examples.shuffle(&mut rng); //Shuffle results

    println!("Elapsed: {:?}", start.elapsed());
    println!("Generating file");
    let start = Instant::now();

    for (i, example) in examples.into_iter().enumerate() {
        if i % (OUTPUT / 100) == 0 {
            println!("Element {}/{} - {}%", i, OUTPUT, i * 100 / OUTPUT);
        }
        let window_feature = Feature::from_f32_iter(example.image.into_iter());
        let index_feature = Feature::from_i64_list(vec![example.palette_index]);

        let example = vec![
            ("window".into(), window_feature),
            ("index".into(), index_feature),
        ]
        .into_iter()
        .collect::<tfrecord::Example>();

        writer.send(example).unwrap();
    }

    println!("Elapsed: {:?}", start.elapsed());
}
