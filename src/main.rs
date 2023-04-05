use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    // Open the video file
    let mut file = File::open("example.mp4")?;

    // Read the entire video file into memory
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Split the video into small segments
    let segment_size = 10_000_000; // 10 MB segments
    let mut segments = Vec::new();
    let mut offset = 0;
    while offset < buffer.len() {
        let end = std::cmp::min(offset + segment_size, buffer.len());
        let segment = &buffer[offset..end];
        segments.push(segment.to_vec());
        offset += segment_size;
    }

    // Generate the HLS playlist file
    let mut playlist = String::new();
    playlist.push_str("#EXTM3U\n");
    for (i, segment) in segments.iter().enumerate() {
        playlist.push_str(&format!("#EXTINF:{},\n", 10.0)); // 10 second duration
        playlist.push_str(&format!("http://example.com/segment{}.ts\n", i));
    }

    // Write the HLS playlist file to disk
    let mut playlist_file = File::create("example.m3u8")?;
    playlist_file.write_all(playlist.as_bytes())?;

    Ok(())
}

