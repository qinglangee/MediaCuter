use tauri::{Emitter, Manager, WindowEvent, DragDropEvent};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::io::{BufRead, BufReader, Read as IoRead};
use regex::Regex;

#[derive(Serialize)]
struct StreamInfo {
    codec_type: String,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Serialize)]
struct MediaMetadata {
    duration_ms: u64,
    has_video: bool,
    has_audio: bool,
    streams: Vec<StreamInfo>,
}

#[derive(Serialize)]
struct WaveformData {
    peaks: Vec<f64>,
    sample_rate: u32,
    samples_per_peak: u32,
}

#[derive(Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
}

#[derive(Deserialize)]
struct FfprobeStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Deserialize)]
struct FfprobeOutput {
    format: Option<FfprobeFormat>,
    streams: Option<Vec<FfprobeStream>>,
}

#[tauri::command]
fn get_video_metadata(file_path: &str) -> Result<MediaMetadata, String> {
    let output = Command::new("ffprobe")
        .args(["-v", "quiet", "-print_format", "json", "-show_format", "-show_streams", file_path])
        .output()
        .map_err(|e| format!("ffprobe failed: {}", e))?;

    if !output.status.success() {
        return Err("ffprobe returned error".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let probe: FfprobeOutput = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse ffprobe output: {}", e))?;

    let mut duration_ms = 0u64;
    if let Some(ref format) = probe.format {
        if let Some(ref dur) = format.duration {
            if let Ok(d) = dur.parse::<f64>() {
                duration_ms = (d * 1000.0) as u64;
            }
        }
    }

    let mut has_video = false;
    let mut has_audio = false;
    let mut streams = Vec::new();

    if let Some(ref stream_list) = probe.streams {
        for s in stream_list {
            let ct = s.codec_type.clone().unwrap_or_default();
            if ct == "video" {
                has_video = true;
            }
            if ct == "audio" {
                has_audio = true;
            }
            streams.push(StreamInfo {
                codec_type: ct,
                codec_name: s.codec_name.clone(),
                width: s.width,
                height: s.height,
            });
        }
    }

    Ok(MediaMetadata {
        duration_ms,
        has_video,
        has_audio,
        streams,
    })
}

#[tauri::command]
fn get_waveform_data(file_path: &str, num_peaks: Option<u32>) -> Result<WaveformData, String> {
    let num_peaks = num_peaks.unwrap_or(4000);

    let dur_output = Command::new("ffprobe")
        .args(["-v", "quiet", "-show_entries", "format=duration", "-print_format", "json", file_path])
        .output()
        .map_err(|e| format!("ffprobe failed: {}", e))?;

    let dur_str = String::from_utf8_lossy(&dur_output.stdout);
    let duration: f64 = if let Ok(probe) = serde_json::from_str::<FfprobeOutput>(&dur_str) {
        probe.format
            .and_then(|f| f.duration)
            .and_then(|d| d.parse::<f64>().ok())
            .unwrap_or(30.0)
    } else {
        30.0
    };

    let sample_rate = 8000u32;
    let total_samples = (duration * sample_rate as f64) as u32;
    let samples_per_peak = if total_samples > num_peaks {
        total_samples / num_peaks
    } else {
        1
    };

    let ffmpeg_output = Command::new("ffmpeg")
        .args([
            "-i", file_path,
            "-ar", &sample_rate.to_string(),
            "-ac", "1",
            "-f", "u8",
            "-hide_banner",
            "-loglevel", "error",
            "-",
        ])
        .output()
        .map_err(|e| format!("ffmpeg failed: {}", e))?;

    let data = &ffmpeg_output.stdout;
    let mut peaks = Vec::new();
    let mut current_peak: u8 = 0;
    let mut sample_count = 0u32;

    for &sample in data {
        let s = if sample > 127 { sample - 128 } else { 128 - sample };
        if s > current_peak {
            current_peak = s;
        }
        sample_count += 1;
        if sample_count >= samples_per_peak {
            peaks.push(current_peak);
            current_peak = 0;
            sample_count = 0;
        }
    }

    let peaks: Vec<f64> = peaks.iter().map(|&p| p as f64 / 128.0).collect();

    Ok(WaveformData {
        peaks,
        sample_rate,
        samples_per_peak,
    })
}

#[tauri::command]
fn get_unique_output_path(dir: &str, filename: &str) -> Result<String, String> {
    let path = std::path::Path::new(dir).join(filename);
    if !path.exists() {
        return Ok(path.to_string_lossy().to_string());
    }

    let stem = path.file_stem().unwrap().to_string_lossy().to_string();
    let ext = path.extension().map(|e| e.to_string_lossy().to_string());

    let mut counter = 1u32;
    loop {
        let new_name = if let Some(ref ext) = ext {
            format!("{}_{}.{}", stem, counter, ext)
        } else {
            format!("{}_{}", stem, counter)
        };
        let new_path = std::path::Path::new(dir).join(&new_name);
        if !new_path.exists() {
            return Ok(new_path.to_string_lossy().to_string());
        }
        counter += 1;
    }
}

#[tauri::command]
async fn execute_ffmpeg(
    app: tauri::AppHandle,
    command_str: Vec<String>,
    output_path: String,
) -> Result<String, String> {
    if command_str.is_empty() {
        return Err("Empty command".to_string());
    }

    let program = &command_str[0];
    let args = &command_str[1..];

    let mut child = Command::new(program)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start ffmpeg: {}", e))?;

    let stderr = child.stderr.take().unwrap();
    let mut reader = BufReader::new(stderr);
    let time_re = Regex::new(r"time=(\d{2}):(\d{2}):(\d{2})\.(\d{2})").unwrap();

    let mut line = String::new();
    loop {
        line.clear();
        let mut buf = [0u8; 1];
        let mut line_bytes = Vec::new();
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(_) => {
                    if buf[0] == b'\n' || buf[0] == b'\r' {
                        if !line_bytes.is_empty() {
                            break;
                        }
                    } else {
                        line_bytes.push(buf[0]);
                    }
                }
                Err(_) => break,
            }
        }
        if line_bytes.is_empty() && reader.fill_buf().map(|b| b.is_empty()).unwrap_or(true) {
            break;
        }
        line = String::from_utf8_lossy(&line_bytes).to_string();

        if let Some(caps) = time_re.captures(&line) {
            let h: u64 = caps[1].parse().unwrap_or(0);
            let m: u64 = caps[2].parse().unwrap_or(0);
            let s: u64 = caps[3].parse().unwrap_or(0);
            let cs: u64 = caps[4].parse().unwrap_or(0);
            let ms = h * 3600000 + m * 60000 + s * 1000 + cs * 10;
            let _ = app.emit("ffmpeg-progress", serde_json::json!({ "time_ms": ms }));
        }
    }

    let status = child.wait().map_err(|e| format!("Failed to wait: {}", e))?;

    if status.success() {
        let _ = app.emit("ffmpeg-progress", serde_json::json!({ "completed": true }));
        Ok(output_path)
    } else {
        Err(format!("ffmpeg exited with status: {}", status))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_video_metadata,
            get_waveform_data,
            execute_ffmpeg,
            get_unique_output_path,
        ])
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();
            let app_handle = app.handle().clone();
            main_window.on_window_event(move |event| {
                if let WindowEvent::DragDrop(drag_event) = event {
                    if let DragDropEvent::Drop { paths, .. } = drag_event {
                        if let Some(path) = paths.first() {
                            let _ = app_handle.emit("file-dropped", path.to_string_lossy().to_string());
                        }
                    }
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
