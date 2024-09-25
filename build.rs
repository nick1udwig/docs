/// fetch .tar.gz of kinode book for docs app
fn get_kinode_book(cwd: &Path) -> anyhow::Result<()> {
    p!("fetching kinode book .tar.gz");
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
        let release_url = format!(
            "https://github.com/kinode-dao/kinode-book/releases/download/{}/{}",
            release.tag_name, release.assets[0].name,
        );
        let book_dir = cwd.join("docs").join("pkg").join("ui");
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
