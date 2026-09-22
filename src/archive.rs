use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use flate2::write::GzEncoder;
use flate2::Compression;
use sha2::{Digest, Sha256};
use tar::Builder;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Creates a `.zip` archive containing the specified list of (source_path, archive_entry_name)
/// atomically via a temporary file.
pub fn create_zip(target_zip: &Path, files: &[(&Path, &str)]) -> Result<(), std::io::Error> {
    if let Some(parent) = target_zip.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp_path = target_zip.with_extension("zip.tmp");
    let file = File::create(&tmp_path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let res = (|| -> Result<(), std::io::Error> {
        for &(src_path, entry_name) in files {
            if src_path.is_file() {
                zip.start_file(entry_name, options)?;
                let mut f = File::open(src_path)?;
                std::io::copy(&mut f, &mut zip)?;
            }
        }
        zip.finish()?;
        Ok(())
    })();

    if let Err(e) = res {
        let _ = fs::remove_file(&tmp_path);
        return Err(e);
    }

    if target_zip.exists() {
        let _ = fs::remove_file(target_zip);
    }
    fs::rename(&tmp_path, target_zip)?;
    Ok(())
}

/// Creates a `.tar.gz` archive containing the specified list of (source_path, archive_entry_name),
/// ensuring POSIX executable permissions (0o755) are set on binary files, atomically via a temporary file.
pub fn create_tar_gz(target_tar_gz: &Path, files: &[(&Path, &str)]) -> Result<(), std::io::Error> {
    if let Some(parent) = target_tar_gz.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp_path = target_tar_gz.with_extension("tar.gz.tmp");
    let file = File::create(&tmp_path)?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut tar = Builder::new(enc);

    let res = (|| -> Result<(), std::io::Error> {
        for &(src_path, entry_name) in files {
            if src_path.is_file() {
                let mut f = File::open(src_path)?;
                let metadata = f.metadata()?;

                let mut header = tar::Header::new_gnu();
                header.set_size(metadata.len());

                // Set executable permission for Unix targets
                header.set_mode(0o755);
                header.set_cksum();

                tar.append_data(&mut header, entry_name, &mut f)?;
            }
        }
        tar.finish()?;
        Ok(())
    })();

    if let Err(e) = res {
        let _ = fs::remove_file(&tmp_path);
        return Err(e);
    }

    if target_tar_gz.exists() {
        let _ = fs::remove_file(target_tar_gz);
    }
    fs::rename(&tmp_path, target_tar_gz)?;
    Ok(())
}

/// Computes SHA-256 hash of a file, returned as a lowercase hex string.
pub fn compute_sha256(path: &Path) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash_result = hasher.finalize();
    let hex = hash_result
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Ok(hex)
}

/// Generates a standard `checksums.txt` file listing SHA-256 hashes for all release files.
pub fn generate_checksums_file(
    output_dir: &Path,
    archive_files: &[PathBuf],
) -> Result<PathBuf, std::io::Error> {
    fs::create_dir_all(output_dir)?;
    let checksums_path = output_dir.join("checksums.txt");
    let mut lines = Vec::new();

    for path in archive_files {
        if path.is_file() {
            let hash = compute_sha256(path)?;
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            lines.push(format!("{}  {}", hash, filename));
        }
    }

    fs::write(&checksums_path, lines.join("\n") + "\n")?;
    Ok(checksums_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_zip_and_sha256() {
        let dir = tempdir().unwrap();
        let src_file = dir.path().join("mycli.exe");
        fs::write(&src_file, "fake binary content").unwrap();

        let zip_file = dir.path().join("mycli.zip");
        create_zip(&zip_file, &[(&src_file, "mycli.exe")]).unwrap();

        assert!(zip_file.is_file());
        let hash = compute_sha256(&zip_file).unwrap();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_create_tar_gz_and_checksums_file() {
        let dir = tempdir().unwrap();
        let src_file = dir.path().join("mycli");
        fs::write(&src_file, "fake unix binary").unwrap();

        let tar_file = dir.path().join("mycli.tar.gz");
        create_tar_gz(&tar_file, &[(&src_file, "mycli")]).unwrap();

        assert!(tar_file.is_file());
        let checksums_file = generate_checksums_file(dir.path(), std::slice::from_ref(&tar_file)).unwrap();
        assert!(checksums_file.is_file());

        let content = fs::read_to_string(&checksums_file).unwrap();
        assert!(content.contains("mycli.tar.gz"));
    }
}
