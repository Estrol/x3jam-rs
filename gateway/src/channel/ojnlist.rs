use byteorder_lite::{ByteOrder as _, LittleEndian};
use tokio::io::AsyncReadExt as _;

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

pub async fn load_ojn_list(path: &str) -> std::io::Result<Vec<Header>> {
    let mut current_dir = std::env::current_dir()?;
    current_dir.push(path);

    current_dir = current_dir.canonicalize()?;

    let file = tokio::fs::File::open(current_dir).await?;
    let mut headers = Vec::new();

    let mut reader = tokio::io::BufReader::new(file);

    let mut count_bytes = [0u8; 4];
    reader.read_exact(&mut count_bytes).await?;

    let count = LittleEndian::read_u32(&count_bytes);

    for _ in 0..count {
        let mut header = std::mem::MaybeUninit::<Header>::uninit();
        let header_slice = unsafe {
            std::slice::from_raw_parts_mut(
                header.as_mut_ptr() as *mut u8,
                std::mem::size_of::<Header>(),
            )
        };

        reader.read_exact(header_slice).await?;
        headers.push(unsafe { header.assume_init() });
    }

    Ok(headers)
}
