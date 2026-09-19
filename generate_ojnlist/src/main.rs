use std::io::{Read as _, Write as _};

pub fn main() {
    if std::env::args().any(|x| x == "--help" || x == "-h") {
        println!("Usage: generate_ojnlist --path <path> --output <output>");
        std::process::exit(0);
    }

    if std::mem::size_of::<Header>() != 300 {
        println!("Header size is not 300 bytes, please check the Header struct definition.");
        std::process::exit(1);
    }

    let args = std::env::args().collect::<Vec<String>>();

    // required args: --path <path> --output <output>

    let path_index = args.iter().position(|x| x == "--path");
    let output_index = args.iter().position(|x| x == "--output");

    if path_index.is_none() || output_index.is_none() {
        println!("Usage: generate_ojnlist --path <path> --output <output>");
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

    for header in &headers {
        let header_slice = unsafe {
            std::slice::from_raw_parts(
                header as *const Header as *const u8,
                std::mem::size_of::<Header>(),
            )
        };

        writer.write_all(header_slice).unwrap();
    }

    #[allow(dead_code)]
    struct ExtraInfo {
        songid: i32,
        value1: i32,
        new_or_top: i32,
        value2: i32,
    }

    writer.write_all(&length.to_le_bytes()).unwrap();

    for header in &headers {
        let extra_info = ExtraInfo {
            songid: header.songid,
            value1: 0,
            new_or_top: 1,
            value2: 0,
        };

        let extra_info_slice = unsafe {
            std::slice::from_raw_parts(
                &extra_info as *const ExtraInfo as *const u8,
                std::mem::size_of::<ExtraInfo>(),
            )
        };

        writer.write_all(extra_info_slice).unwrap();
    }

    #[allow(dead_code)]
    struct ExtraInfo2 {
        songid: i32,
        value1: i32,
        value2: i32,
        value3: i32,
    }

    // writer.write_all(&0i32.to_le_bytes()).unwrap();

    for header in &headers {
        let extra_info2 = ExtraInfo2 {
            songid: header.songid,
            value1: 0,
            value2: 0,
            value3: 0,
        };

        let extra_info2_slice = unsafe {
            std::slice::from_raw_parts(
                &extra_info2 as *const ExtraInfo2 as *const u8,
                std::mem::size_of::<ExtraInfo2>(),
            )
        };

        writer.write_all(extra_info2_slice).unwrap();
    }

    #[allow(dead_code)]
    struct ExtraInfo3 {
        songid: i32,
        data: [std::ffi::c_char; 24], // format: %d %d %d
    }

    writer.write_all(&0i32.to_le_bytes()).unwrap();

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
