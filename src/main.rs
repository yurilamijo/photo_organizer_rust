use std::fs;
use std::path::Path;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let source_directory = "/Volumes/NO NAME";
    let destination_directory_photos = "/Volumes/NO NAME/photos";
    let destination_directory_videos = "/Volumes/NO NAME/videos";

    let start_time = Instant::now();
    move_files_to_destination_directories(
        source_directory,
        destination_directory_photos,
        destination_directory_videos,
    )?;
    let end_time = Instant::now();

    println!("Execution Time: {:?}", end_time.duration_since(start_time));

    Ok(())
}

fn move_files_to_destination_directories(
    source_directory: &str,
    destination_directory_photos: &str,
    destination_directory_videos: &str,
) -> std::io::Result<()> {
    let mut file_extension_map = HashMap::new();
    file_extension_map.insert("jpg", destination_directory_photos);
    file_extension_map.insert("jpeg", destination_directory_photos);
    file_extension_map.insert("png", destination_directory_photos);
    file_extension_map.insert("mov", destination_directory_videos);
    file_extension_map.insert("mp4", destination_directory_videos);
    file_extension_map.insert("avi", destination_directory_videos);
    file_extension_map.insert("mkv", destination_directory_videos);
    file_extension_map.insert("mpg", destination_directory_videos);

    let (tx, rx) = mpsc::channel();

    for entry in fs::read_dir(source_directory)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                let extension = extension.to_lowercase();
                if let Some(destination_directory) = file_extension_map.get(extension.as_str()) {
                    let destination_directory = destination_directory.to_string();
                    let source_path = path.to_path_buf();
                    let tx = tx.clone();

                    if should_move_file(&source_path, &destination_directory) {
                        thread::spawn(move || {
                            let _ = tx.send(move_file(source_path, &destination_directory));
                        });
                    }
                } else {
                    println!("Invalid file extension found: {:?}", extension);
                }
            }
        }
    }

    drop(tx);

    for result in rx {
        match result {
            Ok(_) => {}
            Err(e) => println!("Error moving file: {}", e),
        }
    }

    Ok(())
}

fn move_file(source_path: std::path::PathBuf, destination_directory: &str) -> std::io::Result<()> {
    if !Path::new(destination_directory).exists() {
        fs::create_dir_all(destination_directory)?;
    }

    if let Some(file_name) = source_path.file_name() {
        let destination_path = Path::new(destination_directory).join(file_name);
        fs::rename(&source_path, destination_path)?; // Move the file
        println!("Moved file: {:?}", source_path);
    }

    Ok(())
}

fn should_move_file(source_path: &Path, destination_directory: &str) -> bool {
    let file_name = match source_path.file_name() {
        Some(name) => name,
        None => return false,
    };
    let destination_file_path = Path::new(destination_directory).join(file_name);

    !destination_file_path.exists()
}
