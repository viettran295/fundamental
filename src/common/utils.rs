use log::{debug, error, warn};
use std::{fs, io, path};
use tokio::task;
use zip::ZipArchive;

pub async fn decompress_zip_file(file_path: &str) -> io::Result<()> {
    let own_zip_path = file_path.to_owned();
    let result = task::spawn_blocking(move || -> io::Result<()> {
        let zip_path = path::Path::new(&own_zip_path);
        debug!("Starting to decompress {:?}", zip_path);
        // Extract and create output directory
        let unzip_dir = zip_path
            .parent()
            .unwrap_or_else(|| path::Path::new(""))
            .join(zip_path.file_stem().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid zip file name")
            })?);
        if unzip_dir.exists() && unzip_dir.is_dir() {
            fs::remove_dir_all(&unzip_dir)?;
        }
        fs::create_dir_all(&unzip_dir)?;
        // Read and loop through all zip files
        let zip_file = fs::File::open(zip_path)?;
        let mut archive = ZipArchive::new(zip_file)?;
        for i in 0..archive.len() {
            // Get files in .zip and combine with output directory path
            let mut compressed_file = archive.by_index(i)?;
            let compressed_file_path = match compressed_file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };
            let output_path = unzip_dir.join(&compressed_file_path);
            // Decompress file (not directory) in .zip
            if !compressed_file.name().ends_with('/') {
                let mut output_file = match fs::File::create(&output_path) {
                    Ok(file) => file,
                    Err(e) => {
                        warn!("Error creating file: {:?} - {}", &output_path, e);
                        continue;
                    }
                };
                if let Err(e) = io::copy(&mut compressed_file, &mut output_file) {
                    warn!("Error decompressing file: {:?} - {}", output_file, e);
                    continue;
                };
            }
        }
        debug!("Finish to decompress {:?}", zip_path);
        Ok(())
    })
    .await;
    match result {
        Ok(result) => result,
        Err(e) => {
            error!("Error decompressing zip file: {}", file_path);
            Err(io::Error::new(io::ErrorKind::Other, e))
        }
    }
}
