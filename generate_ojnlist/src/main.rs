use std::io::{Read as _, Write as _};

pub fn main() {
    if std::env::args().any(|x| x == "--help" || x == "-h") {
        println!("Usage: generate_ojnlist --path <path> --output <output>");
        std::process::exit(0);
    }

    if std::mem::size_of::<Header>() != 300 {
        eprintln!("Header size is not 300 bytes, please check the Header struct definition.");
        std::process::exit(1);
    }

    let args = std::env::args().collect::<Vec<String>>();

    // required args: --path <path> --output <output>

    let path_index = args.iter().position(|x| x == "--path");
    let output_index = args.iter().position(|x| x == "--output");

    if path_index.is_none() || output_index.is_none() {
        eprintln!("Usage: generate_ojnlist --path <path> --output <output>");
        std::process::exit(1);
    }

    let path = args[path_index.unwrap() + 1].clone();
    let output = args[output_index.unwrap() + 1].clone();

    println!("Scanning path: {}", path);

    let mut headers = Vec::new();
    // let limit = 1000;
    // let mut iter = 0;

    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() && path.extension().map(|x| x == "ojn").unwrap_or(false) {
            let mut file = std::fs::File::open(&path).unwrap();
            let mut header = std::mem::MaybeUninit::<Header>::uninit();
            let header_slice = unsafe {
                std::slice::from_raw_parts_mut(
                    header.as_mut_ptr() as *mut u8,
                    std::mem::size_of::<Header>(),
                )
            };

            file.read_exact(header_slice).unwrap();

            headers.push(unsafe { header.assume_init() });

            // iter += 1;

            // if iter > limit {
            //     println!("Reached limit of {} files, stopping scan.", limit);
            //     break;
            // }
        }
    }

    // sort by songid
    headers.sort_by_key(|h| h.songid);

    println!("Processing {} .ojn files", headers.len());

    let mut writer = std::fs::File::create(output).unwrap();

    let length = headers.len() as u32;
    writer.write_all(&length.to_le_bytes()).unwrap();

    for header in headers {
        let header_slice = unsafe {
            std::slice::from_raw_parts(
                &header as *const Header as *const u8,
                std::mem::size_of::<Header>(),
            )
        };

        writer.write_all(header_slice).unwrap();
    }

    println!("Done!");
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub songid: i32,
    pub signature: [u8; 4],
    pub encode_version: f32,
    pub genre: i32,
    pub bpm: f32,
    pub level: [i16; 4],
    pub event_count: [i32; 3],
    pub note_count: [i32; 3],
    pub measure_count: [i32; 3],
    pub package_count: [i32; 3],
    pub old_encode_version: i16,
    pub old_songid: i16,
    pub old_genre: [u8; 20],
    pub bmp_size: i32,
    pub old_file_version: i32,
    pub title: [u8; 64],
    pub artist: [u8; 32],
    pub noter: [u8; 32],
    pub ojm_file: [u8; 32],
    pub cover_size: i32,
    pub time: [i32; 3],
    pub note_offset: [i32; 3],
    pub cover_offset: i32,
}
