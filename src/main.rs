use std::{
    env, fs,
    io::{self, Read},
    time::{Duration, Instant},
};

const MAGIC: [u8; 4] = [0xf9, 0xbe, 0xb4, 0xd9];

fn main() {
    let start = Instant::now();

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
    let mut done = 0;
    let mut timer = Instant::now();
    let duration = Duration::from_secs(5);

    paths.into_iter().for_each(|path| {
        if !path.extension().is_some_and(|f| f == "dat") || path.iter().last().unwrap() == "xor.dat"
        {
            return;
        }

        let mut file = fs::File::open(&path).unwrap();
        let mut buf = [0u8; 4];
        if let Err(e) = file.read_exact(&mut buf) {
            println!(
                "Error reading file {}: {e:?}",
                path.iter().last().unwrap().to_str().unwrap()
            );
            return;
        };

        if buf == MAGIC {
            let Ok(mut block) = fs::read(&path) else {
                println!(
                    "Error reading file {}",
                    path.iter().last().unwrap().to_str().unwrap()
                );
                return;
            };
            block
                .iter_mut()
                .enumerate()
                .for_each(|(i, b)| *b ^= key[i % key.len()]);

            let mut tmp_path = path.as_os_str().to_owned();
            tmp_path.push(".tmp");
            fs::write(&tmp_path, block).unwrap();
            fs::rename(&tmp_path, &path).unwrap();
        }

        done += 1;
        if timer.elapsed() > duration {
            println!("Xor'd {done} / {total} files");
            timer = Instant::now();
        }
    });

    println!(
        "Done in {} seconds! Blocksdir is now xor'd.",
        start.elapsed().as_secs()
    );
}
