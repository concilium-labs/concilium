use chrono::Utc;
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};
use std::path::Path;
use concilium_error::Error;

async fn save(log: &str, log_type: &str, file_name: &str) -> Result<(), Error> {
    if ! Path::new("logs").is_dir() {
        fs::create_dir("logs").await?;
    }

    let mut file = OpenOptions::new()
    .write(true)
    .append(true)
    .create(true)
    .open(format!("logs/{}.log", file_name))
    .await?;

    file.write_all(format!("[{}] {} {}\n", Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(), log_type, log).as_bytes()).await?;

    Ok(())
}

pub async fn error(log: &str) -> Result<(), Error> {
    save(log, "ERROR", "concilium").await?;

    Ok(())
}

pub async fn warn(log: &str) -> Result<(), Error> {
    save(log, "WARN", "concilium").await?;

    Ok(())
}

pub async fn trace(log: &str) -> Result<(), Error> {
    save(log, "TRACE", "concilium").await?;

    Ok(())
}

pub async fn debug(log: &str) -> Result<(), Error> {
    save(log, "DEBUG", "concilium").await?;

    Ok(())
}

pub async fn info(log: &str) -> Result<(), Error> {
    save(log, "INFO", "concilium").await?;

    Ok(())
}
