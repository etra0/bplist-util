use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Song {
    pub key: String,
    pub hash: String,
    pub name: String,
    pub uploader: String
}

#[allow(non_snake_case, dead_code)]
#[derive(Deserialize)]
pub struct Bplist {
    playlistTitle: String,
    playlistAuthor: String,
    playlistDescription: String,
    syncURL: String,
    pub songs: Vec<Song>
}
