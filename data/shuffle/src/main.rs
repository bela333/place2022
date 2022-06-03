use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    time::Instant,
};


use flate2::read::GzDecoder;
use rand::{
    prelude::{IteratorRandom, SliceRandom, ThreadRng},
};
use tfrecord::{ExampleWriter, Feature, RecordWriter};

use crate::{
    entry::Entry,
    sample::{Reservoir, Sample},
};

mod color;
mod entry;
mod sample;

/// Copies a window of values from `arr` to `buf`
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

const OUTPUT: usize = 32000;

/// Contains examples, that are to be exported
struct Example {
    /// Window, with the current pixel in the middle
    pub image: [f32; SIZEU * SIZEU * 3],
    /// Palette index of the current pixel
    pub palette_index: i64,
}

fn main() {
    let mut rng = rand::thread_rng();

    //Files are gzip encoded
    let f = File::open("../sorted.csv.gz").unwrap();
    let f = GzDecoder::new(f);
    let f = BufReader::new(f);
    println!("Reading file");

    let take_size = 156353085;
    //let take_size = 1000000;

    //156353085: 160353085 total entries - 2000*2000 white pixels at the end
    let entries = f
        .lines()
        .flatten().filter_map(|line| Entry::from_line(&line))
        .take(take_size);

    //Choose OUTPUT pixels randomly, for investigation
    let samples =
        Sample::iterate(IMAGE_SIZE as _, IMAGE_SIZE as _).choose_multiple(&mut rng, OUTPUT);
    println!("Samples: {}", samples.len());

    //Create one reservoir for each sample
    let mut examples: HashMap<Sample, Reservoir<Example, ThreadRng>> = HashMap::new();
    for sample in samples {
        examples.insert(sample, Reservoir::new(rng.clone()));
    }
    println!("Examples: {}", examples.len());

    let mut canvas = (0..IMAGE_SIZE * IMAGE_SIZE * 3)
        .map(|_| 1f32)
        .collect::<Vec<_>>();

    let start = Instant::now();

    for (i, entry) in entries.enumerate() {
        if i % (take_size / 1000) == 0 {
            let elapsed = start.elapsed();
            let fract = (i + 1) as f32 / take_size as f32;
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

        //Update reservoir
        if let Some(reservoir) = examples.get_mut(&Sample(x, y)) {
            if let Some(handle) = reservoir.get_handle() {
                let mut o = [1f32; SIZEU * SIZEU * 3];
                window(&canvas, &mut o, x as isize - MARGIN, y as isize - MARGIN);

                *handle = Some(Example {
                    palette_index: entry.color.3 as i64,
                    image: o,
                })
            }
        }

        //Update canvas
        let index = (entry.pos.0 as usize + entry.pos.1 as usize * IMAGE_SIZE as usize) * 3;

        canvas[index] = entry.color.0;
        canvas[index + 1] = entry.color.1;
        canvas[index + 2] = entry.color.2;
    }

    let mut examples: Vec<&Example> = examples
        .iter().filter_map(|(_, reservoir)| reservoir.get_current())
        .collect();

    println!("Example count: {}", examples.len());

    examples.shuffle(&mut rng); //Shuffle results

    println!("Elapsed: {:?}", start.elapsed());
    println!("Generating file");
    let start = Instant::now();

    let mut writer: ExampleWriter<_> = RecordWriter::create("dataset.tfrecord").unwrap();

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
