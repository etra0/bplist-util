use tokio::sync::Semaphore;
use std::{io::Write, sync::Arc};
use std::path::Path;
use clap::{Arg, App};

mod bplist;

use bplist::*;

static BASE_URL: &str = "https://api.beatsaver.com/download/key/";

/// This is the default destination path.
static DESTINATION_PATH: &str = r#"C:\Program Files (x86)\Steam\steamapps\common\Beat Saber\Beat Saber_Data\CustomLevels\"#;

async fn download_song(song: Song, sem: Arc<Semaphore>, client: reqwest::Client) -> Result<(), Box<dyn std::error::Error>> {
    let sacq = sem.acquire().await.unwrap();

    let final_url = format!("{}/{}", BASE_URL, song.key);
    println!("Downloading {}", final_url);
    let response = client.get(final_url).send().await?;

    let mut output_zip = tempfile::tempfile()?;

    let bytes = response.bytes().await?;
    output_zip.write(&bytes)?;
    output_zip.flush()?;
    drop(sacq);

    let path = Path::new(DESTINATION_PATH).join(&song.name);
    std::fs::create_dir(path)?;

    let mut zip = zip::read::ZipArchive::new(output_zip)?;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let filepath = Path::new(DESTINATION_PATH).join(&song.name).join(file.name());
        let mut output_file = std::fs::File::create(filepath)?;
        println!("Decompressing deez {}", file.name());
        std::io::copy(&mut file, &mut output_file)?;
    }

    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("BPlist downloader")
        .version(std::env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("BPLIST")
            .required(true)
            .help("The BPlist file from where to download songs"))
        .arg(Arg::with_name("duplicates")
            .help("Check if there are duplicated songs")
            .short("d"))
        .get_matches();

    let bplist_file = std::fs::read_to_string(matches.value_of("BPLIST").expect("The BPLIST is needed"))?;
    let bplist: Bplist = serde_json::from_str(&bplist_file)?;

    let client = reqwest::Client::new();

    // We limit the number of requests to 8 because the API doesn't like the same IP asking for too
    // much information at the same time.
    let sem = Arc::new(Semaphore::new(8));

    let futures = futures::stream::FuturesUnordered::new();
    for song in bplist.songs {
        let sem = sem.clone();
        let client = client.clone();
        futures.push(tokio::spawn(async {
            download_song(song, sem, client).await.expect("A song failed");
        }));
    }

    futures::future::join_all(futures).await;

    Ok(())
}
