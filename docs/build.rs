use std::{
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use flate2::read::GzDecoder;
use tar::Archive;

const EXPECTED_BOOK_TAR_GZ: &str = "book.tar.gz";

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn untar_gz_file(path: &Path, dest: &Path) -> std::io::Result<()> {
    // Open the .tar.gz file
    let tar_gz = File::open(path)?;
    let tar_gz_reader = BufReader::new(tar_gz);

    // Decode the gzip layer
    let tar = GzDecoder::new(tar_gz_reader);

    // Create a new archive from the tar file
    let mut archive = Archive::new(tar);

    // Unpack the archive into the specified destination directory
    archive.unpack(dest)?;

    Ok(())
}

/// fetch .tar.gz of kinode book for docs app
fn get_kinode_book(cwd: &Path) -> anyhow::Result<()> {
    p!("fetching kinode book .tar.gz");
    let book_dir = cwd.parent().unwrap().parent().unwrap().join("docs").join("pkg").join("ui");
    if book_dir.exists() {
        fs::remove_dir_all(&book_dir)?;
    }
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let releases = kit::boot_fake_node::fetch_releases("kinode-dao", "kinode-book")
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        if releases.is_empty() {
            return Err(anyhow::anyhow!("couldn't retrieve kinode-book releases"));
        }
        let release = &releases[0];
        if release.assets.is_empty() {
            return Err(anyhow::anyhow!(
                "most recent kinode-book release has no assets"
            ));
        }
        let maybe_book_tar_index = release.assets.iter().position(|item| item.name == EXPECTED_BOOK_TAR_GZ);
        let book_name = if let Some(book_tar_index) = maybe_book_tar_index {
            release.assets[book_tar_index].name.clone()
        } else {
            p!("Couldn't find {EXPECTED_BOOK_TAR_GZ} in release assets; YOLOing...");
            EXPECTED_BOOK_TAR_GZ.to_string()
        };
        let release_url = format!(
            "https://github.com/kinode-dao/kinode-book/releases/download/{}/{book_name}",
            release.tag_name
        );
        fs::create_dir_all(&book_dir)?;
        let book_tar_path = book_dir.join("book.tar.gz");
        kit::build::download_file(&release_url, &book_tar_path)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        untar_gz_file(&book_tar_path, &book_dir)?;
        fs::remove_file(book_tar_path)?;
        Ok(())
    })
}

fn main() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    get_kinode_book(&cwd)?;
    Ok(())
}
