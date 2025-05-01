use rayon::prelude::*;
use std::{
    env, fs,
    io::{self, Read},
    sync::atomic::{AtomicUsize, Ordering},
};

const MAGIC: [u8; 4] = [0xf9, 0xbe, 0xb4, 0xd9];

fn main() {
    let datadir = match env::consts::OS {
        "macos" => "Library/Application Support/Bitcoin",
        "windows" => "AppData\\Local\\Bitcoin",
        "linux" => ".bitcoin",
        _ => panic!("Unknown OS"),
    };

    let blocks_path: std::path::PathBuf = env::home_dir().unwrap().join(datadir).join("blocks");

    let paths = fs::read_dir(&blocks_path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();

    let xor_path: std::path::PathBuf = blocks_path.join("xor.dat");
    if !fs::exists(&xor_path).unwrap() {
        println!("No xor.dat file. Make sure you are running Bitcoin Core v28 or higher.");
        return;
    }

    println!("Xor'ing blocks dir. Do not start bitcoind until finished!");

    let key: [u8; 8] = fs::read(&xor_path).unwrap().try_into().unwrap();
    let key = if key == [0u8; 8] { rand::random() } else { key };

    fs::write(xor_path, key).unwrap();

    let total = paths.len();
    let done = AtomicUsize::new(0);

    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .unwrap();
    paths.into_par_iter().for_each(|path| {
        if !path.extension().is_some_and(|f| f == "dat") || path.iter().last().unwrap() == "xor.dat"
        {
            return;
        }

        let mut file = fs::File::open(&path).unwrap();
        let mut buf = [0u8; 4];
        file.read_exact(&mut buf).unwrap();
        buf.iter_mut()
            .enumerate()
            .for_each(|(i, b)| *b ^= key[i % key.len()]);

        if buf != MAGIC {
            let mut block = fs::read(&path).unwrap();
            block
                .iter_mut()
                .enumerate()
                .for_each(|(i, b)| *b ^= key[i % key.len()]);

            fs::write(&path, block).unwrap();
        }

        let done = done.fetch_add(1, Ordering::Relaxed);
        if done % 100 == 0 {
            println!("Xor'd {done} / {total} files");
        }
    });

    println!("Done! Blocksdir is now xor'd.");
}
