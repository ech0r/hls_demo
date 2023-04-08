use std::fs::{self, File};
use std::io::{self, prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::process::Command;
use hls_demo::ThreadPool;

fn main() -> io::Result<()> {
    // generate HLS playlist and segments
    //generate_hls_segments("demo_video.mp4", "segments", 10)?;

    let listener = TcpListener::bind("0.0.0.0:8080")?;
    let pool = ThreadPool::new(4);
    println!("Server listening on port 8080");

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_client(stream);
        });
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..]);
    let response = match &request {
        r if r.contains(".m3u8") => {
            let file = File::open("./segments/hls_demo.m3u8").unwrap();
            let mut reader = BufReader::new(file);
            let mut contents = Vec::new();
            reader.read_to_end(&mut contents).unwrap();
            let headers = format!(
                "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: {}\r\n\r\n", 
                contents.len()
            ).as_bytes().to_vec();
            let mut response = headers;
            response.extend(contents);
            response
        },
        r if r.contains(".mp4") => {
            let segment_file = &request.split("/").nth(1).unwrap().split(" ").nth(0).unwrap();
            match File::open(format!("./segments/{}", segment_file)) {
                Ok(file) => {
                    let mut reader = BufReader::new(file);
                    let mut contents = Vec::new();
                    reader.read_to_end(&mut contents).unwrap();
                    let headers = format!(
                        "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: video/mp4\r\nContent-Length: {}\r\n\r\n", 
                        contents.len()
                    ).as_bytes().to_vec();
                    let mut response = headers;
                    response.extend(contents);
                    response      
                }, 
                Err(_err) => {
                    format!("HTTP/1.1 404 NOT FOUND\r\n\r\n").as_bytes().to_vec()
                }
            }
        },
        r if r.contains(".js") => {
            let file_name = &request.split("/").nth(1).unwrap().split(" ").nth(0).unwrap();
            match File::open(format!("./static/{}", file_name)) {
                Ok(file) => {
                    let mut reader = BufReader::new(file);
                    let mut contents = Vec::new();
                    reader.read_to_end(&mut contents).unwrap();
                    let headers = format!(
                        "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/javascript\r\nContent-Length: {}\r\n\r\n", 
                        contents.len()
                    ).as_bytes().to_vec();
                    let mut response = headers;
                    response.extend(contents);
                    response      
                }, 
                Err(_err) => {
                    format!("HTTP/1.1 404 NOT FOUND\r\n\r\n").as_bytes().to_vec()
                }
            }
        },
        r if r.contains(".wasm") => {
            let file_name = &request.split("/").nth(1).unwrap().split(" ").nth(0).unwrap();
            match File::open(format!("./static/{}", file_name)) {
                Ok(file) => {
                    let mut reader = BufReader::new(file);
                    let mut contents = Vec::new();
                    reader.read_to_end(&mut contents).unwrap();
                    let headers = format!(
                        "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/wasm\r\nContent-Length: {}\r\n\r\n", 
                        contents.len()
                    ).as_bytes().to_vec();
                    let mut response = headers;
                    response.extend(contents);
                    response      
                }, 
                Err(_err) => {
                    format!("HTTP/1.1 404 NOT FOUND\r\n\r\n").as_bytes().to_vec()
                }
            }
        },
        r if r.starts_with("GET /") => {
            let file = File::open("./static/index.html").unwrap();
            let mut reader = BufReader::new(file);
            let mut contents = Vec::new();
            reader.read_to_end(&mut contents).unwrap();
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                contents.len()
            ).as_bytes().to_vec();
            let mut response = headers;
            response.extend(contents);
            response
        },
        _=> format!("HTTP/1.1 404 NOT FOUND\r\n\r\n").as_bytes().to_vec()
    };
    stream.write(&response).unwrap();
    stream.flush().unwrap();
}

fn generate_hls_segments(input_file: &str, output_dir: &str, segment_duration: u32) -> std::io::Result<()> {
    // Create the output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(input_file)
        .arg("-codec:v")
        .arg("copy")
        .arg("-codec:a")
        .arg("copy")
        .arg("-map")
        .arg("0")
        .arg("-f")
        .arg("segment")
        .arg("-segment_time")
        .arg("10")
        .arg("-segment_format")
        .arg("mp4")
        .arg("-segment_list")
        .arg("segments/hls_demo.m3u8")
        .arg("-segment_list_type")
        .arg("m3u8")
        .arg("segments/")
        .arg("%04d.mp4")
        .status()?;

    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "FFmpeg command failed",
        ));
    }

    Ok(())
}
