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

/// Moves files from a source directory to separate destination directories based on their file extensions.//+
/////+
/// This function reads all files from the source directory, determines their file extensions, and moves them to//+
/// the corresponding destination directory (photos or videos). It uses multithreading to improve performance.//+
/////+
/// # Parameters//+
/////+
/// * `source_directory` - A reference to a `&str` representing the source directory where the files are located.//+
/// * `destination_directory_photos` - A reference to a `&str` representing the destination directory for photos.//+
/// * `destination_directory_videos` - A reference to a `&str` representing the destination directory for videos.//+
/////+
/// # Return Value//+
/////+
/// Returns a `Result` indicating whether the file movement was successful.//+
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

/// Moves a file from the source path to the destination directory.
///
/// # Parameters
///
/// * `source_path` - A `PathBuf` representing the source path of the file to be moved.
/// * `destination_directory` - A `&str` representing the destination directory where the file should be moved.
///
/// # Return Value
///
/// Returns a `Result` indicating whether the file was successfully moved.
///
/// * `Ok(())` - If the file was moved successfully.
/// * `Err(e)` - If an error occurred during the file movement, where `e` is the associated error.
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

/// Determines whether a file should be moved based on its existence in the destination directory.
///
/// # Parameters
///
/// * `source_path` - A reference to a `Path` representing the source path of the file to be checked.
/// * `destination_directory` - A reference to a `&str` representing the destination directory where the file should be moved.
///
/// # Return Value
///
/// Returns a `bool` indicating whether the file should be moved.
///
/// * `true` - If the file does not exist in the destination directory.
/// * `false` - If the file already exists in the destination directory.
fn should_move_file(source_path: &Path, destination_directory: &str) -> bool {
    let file_name = match source_path.file_name() {
        Some(name) => name,
        None => return false,
    };
    let destination_file_path = Path::new(destination_directory).join(file_name);

    !destination_file_path.exists()
}
