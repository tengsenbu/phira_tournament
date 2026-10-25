use anyhow::{Context, Result};
use macroquad::prelude::load_file;
use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub static STEP: AtomicUsize = AtomicUsize::new(0);
pub static PENDING_SONG: Mutex<Option<(crate::page::ChartItem, String)>> = Mutex::new(None);

pub fn current() -> usize {
    STEP.load(Ordering::Relaxed)
}

pub fn advance() {
    STEP.fetch_add(1, Ordering::Relaxed);
}

pub fn reset() {
    STEP.store(0, Ordering::Relaxed);
}

pub fn take_pending_song() -> Option<(crate::page::ChartItem, String)> {
    PENDING_SONG.lock().unwrap().take()
}

pub async fn import_pez(name: &str) -> Result<String> {
    use crate::{get_data_mut, save_data};
    use crate::data::LocalChart;
    use prpr::config::Mods;

    let bytes = load_file(name).await.with_context(|| format!("failed to load {name}"))?;
    let (dir_path, uuid) = crate::scene::gen_custom_dir()?;
    let local_path = format!("custom/{uuid}");
    let dir = prpr::dir::Dir::new(dir_path.clone())?;
    prpr::ext::unzip_into(Cursor::new(bytes), &dir, true)?;

    let mut fs = prpr::fs::fs_from_file(&dir_path)?;
    let info = prpr::fs::load_info(fs.as_mut()).await?;
    let chart = LocalChart {
        info: info.into(),
        local_path: local_path.clone(),
        record: None,
        mods: Mods::default(),
        played_unlock: false,
    };
    get_data_mut().charts.push(chart);
    save_data()?;
    Ok(local_path)
}