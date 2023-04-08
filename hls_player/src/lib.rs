use js_sys::{ArrayBuffer, Uint8Array};
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    console,
    Document, 
    Element,
    Event,
    EventTarget,
    HtmlMediaElement,
    HtmlVideoElement,
    MediaSource,
    Request, 
    RequestInit, 
    RequestMode, 
    Response,
    SourceBuffer,
    Url,
    window, MediaSourceReadyState,
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Debug)]
pub enum Playlist {
    Master(MasterPlaylist),
    Media(MediaPlaylist),
}

#[derive(Debug)]
pub struct MasterPlaylist {
    variants: Vec<Variant>,
}

#[derive(Debug)]
pub struct Variant {
    uri: String,
    bandwidth: u32,
}

#[derive(Clone, Debug)]
pub struct MediaPlaylist {
    segments: Vec<MediaSegment>,
}

#[derive(Clone, Debug)]
pub struct MediaSegment {
    uri: String,
    duration: f64,
}

fn parse_playlist(content: &str) -> Result<Playlist, &str> {
    if content.contains("#EXTM3U") {
        if content.contains("#EXT-X-STREAM-INF") {
            parse_master_playlist(content)
        } else if content.contains("#EXTINF") {
            parse_media_playlist(content)
        } else {
            Err("Invalid playlist format.")
        }
    } else {
        Err("Invalid HLS playlist.")
    }
}

fn parse_master_playlist(content: &str) -> Result<Playlist, &str> {
    let mut variants = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut line_iter = lines.iter();
    while let Some(line) = line_iter.next() {
        if line.starts_with("#EXT-X-STREAM-INF:") {
            let bandwidth = parse_bandwidth(line)?;
            if let Some(uri_line) = line_iter.next() {
                let uri = uri_line.to_string();
                let variant = Variant { uri, bandwidth };
                variants.push(variant);
            } else {
                return Err("Invalid master playlist format.");
            }
        }
    }
    Ok(Playlist::Master(MasterPlaylist { variants } ))
}

fn parse_media_playlist(content: &str) -> Result<Playlist, &str> {
    let mut segments = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut line_iter = lines.iter();
    while let Some(line) = line_iter.next() {
        if line.starts_with("#EXTINF:") {
            let duration = parse_duration(line)?;
            if let Some(uri_line) = line_iter.next() {
                let uri = uri_line.to_string();
                let segment = MediaSegment { uri, duration };
                segments.push(segment);
            } else {
                return Err("Invalid media playlist format.");
            }
        }
    }
    Ok(Playlist::Media(MediaPlaylist { segments }))
}

fn parse_bandwidth(line: &str) -> Result<u32, &str> {
    let parts: Vec<&str> = line.split(',').collect();
    for part in parts {
        if part.starts_with("BANDWIDTH=") {
            let bandwidth_str = &part["BANDWIDTH=".len()..];
            return bandwidth_str.parse().map_err(|_| "Invalid bandwidth value");

        }
    }
    Err("Bandwidth not found")
}

fn parse_duration(line: &str) -> Result<f64, &str> {
    let colon_index = line.find(':').ok_or("Invalid duration format")?;
    let comma_index = line.find(',').ok_or("Invalid duration format")?;
    let duration_str = &line[(colon_index + 1)..comma_index];
    duration_str.parse().map_err(|_| "Invalid duration value")
}

async fn fetch_playlist(url: &str) -> Result<String, JsValue> {
    let window = window().ok_or("No window object, oopsie!")?;
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::NoCors);
    let url = "/localhost.m3u8".to_owned();
    let request = Request::new_with_str_and_init(&url, &opts)?;
    request.headers().set("Accept", "application/vnd.apple.mpegurl")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    let text = JsFuture::from(resp.text()?).await?;
    Ok(text.as_string().ok_or("JsValue is invalid String")?)
}

async fn fetch_data_as_array_buffer(url: &str) -> Result<ArrayBuffer, JsValue> {
    let window = window().ok_or("No window object, oopsie!")?;
    let mut request_init = RequestInit::new();
    request_init.method("GET");
    request_init.mode(RequestMode::NoCors);
    let request = Request::new_with_str_and_init(url, &request_init)?;
    let response = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: Response = response.dyn_into()?;
    if !response.ok() {
        return Err(JsValue::from_str("Failed to fetch data"));
    }
    let array_buffer = JsFuture::from(response.array_buffer()?).await?;
    let array_buffer: js_sys::ArrayBuffer = array_buffer.dyn_into()?;
    Ok(array_buffer)
}

async fn fetch_segment(url: &str) -> Result<ArrayBuffer, JsValue> {
    let response = fetch_data_as_array_buffer(url).await?;
    Ok(response)
}

pub async fn load_playlist(url: &str) -> Result<MediaPlaylist, String> {
    let content = fetch_playlist(url).await.map_err(|err| err.as_string().unwrap_or_else(|| "Error fetching playlist".to_string()))?;
    let playlist = parse_playlist(&content).map_err(|err| err.to_string());
    let media_playlist = match playlist {
        Ok(Playlist::Media(media_playlist)) => {
            media_playlist
            //play_video(&video_element, media_playlist).await.map_err(|err| format!("Error playing video: {:?}", err))?;
        }
        _ => return Err("Master playlist handling not implemented yet.".to_owned())
    };
    Ok(media_playlist)
}

pub async fn get_video_element() -> HtmlVideoElement {
    // DOM query to get video element
    let window = window().expect("No window object, oopsie!");
    let video_element = window.document()
        .expect("No DOM, oopsie!")
        .get_element_by_id("video")
        .expect("No video element, oopsie!");
    let video: HtmlVideoElement = video_element.dyn_into::<HtmlVideoElement>().expect("Invalid video element, woops!");
    video
}

fn log_array_buffer(array_buffer: &ArrayBuffer) {
    let uint8_array = Uint8Array::new(array_buffer);
    let contents = uint8_array.to_vec();
    let contents_str = format!("{:?}", contents);
    console_log!("{:?}", contents_str);
}

fn source_buffer_updated() {
    console_log!("SourceBuffer updated!");
}

#[wasm_bindgen(start)]
pub async fn run() -> Result<(), JsValue> {
    log("HOLA");

    // closure to run on MediaSource sourceopen
    let media_source_closure = Closure::wrap(Box::new(move |event: Event| {
        spawn_local(async move {
            let video = get_video_element().await;
            let media_playlist: MediaPlaylist = load_playlist("/localhost.m3u8").await.expect("Failed to load media playlist");
            let media_source = event.target().unwrap().dyn_into::<MediaSource>().unwrap();
            let mime_codec = "video/mp4; codecs=\"avc1.640028, mp4a.40.2\"";
            let source_buffer: SourceBuffer = media_source.add_source_buffer(&mime_codec).expect("Failed to add source buffer");  
            console_log!("MEDIA TYPE SUPPORTED!");
            let segments = media_playlist.segments.clone();
            let buffer_update_closure = Closure::wrap(Box::new(move |event: Event| {
                console_log!("SourceBuffer Updated!");
            }) as Box<dyn FnMut(Event)>);
            let new_segment = fetch_segment(&segments[0].uri).await.expect("Failed to get first segment.");
            source_buffer.append_buffer_with_array_buffer(&new_segment).expect(&format!("Failed to append segment: {}", &segments[0].uri));
            source_buffer.add_event_listener_with_callback("updateend", buffer_update_closure.as_ref().unchecked_ref()).expect("Failed to add source buffer event listener.");
        })
    }) as Box<dyn FnMut(Event)>);

    let video: HtmlVideoElement = get_video_element().await;
    // Construct media source
    let media_source = MediaSource::new().expect("Failed to initialize new MediaSource.");
    let media_source_url = Url::create_object_url_with_source(&media_source).expect("Error creating media source url.");
    // set src url on video element
    video.set_src(&media_source_url);
    // set up event listener once source is open
    let media_source_event_target: EventTarget = media_source.clone().unchecked_into();
    media_source_event_target.add_event_listener_with_callback("sourceopen", media_source_closure.as_ref().unchecked_ref()).expect("Failed to add media source event listener.");
    Ok(())
}
















