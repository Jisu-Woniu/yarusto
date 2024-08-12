use std::{
    borrow::Cow,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use async_walkdir::WalkDir;
use tempfile::TempDir;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tokio_stream::StreamExt;
use zip::ZipArchive;

use crate::model::{cases_config::CasesConfig, config::Config, raw::config1::RawConfig1};

pub struct Converter {
    config_paths: Vec<PathBuf>,
    temp_dir: TempDir,
}

impl Converter {
    pub async fn build(input_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let zip_file = find_zip_file(input_path).await?;

        let temp_dir = TempDir::new()?;

        let config_paths = extract_config_file(&zip_file, &temp_dir).await?;

        Ok(Self {
            config_paths,
            temp_dir,
        })
    }

    pub async fn rename(&self) -> crate::error::Result<&Self> {
        fn map_ext(orig: &str) -> Cow<'_, str> {
            match orig {
                "in" => Cow::Borrowed(orig),
                _ => Cow::Owned(String::from("out")),
            }
        }

        let mut entries = WalkDir::new(&self.temp_dir);
        while let Some(entry) = entries.try_next().await? {
            let path = entry.path();
            let filename = path
                .file_name()
                .expect("full path should have filename")
                .to_str()
                .expect("filename contains invalid UTF-8");

            if let Some((basename, ext @ ("in" | "out" | "ans"))) = filename.rsplit_once(".") {
                let digits = {
                    let pos = basename
                        .as_bytes()
                        .iter()
                        .rposition(|b| !b.is_ascii_digit())
                        .map_or(0, |p| p + 1);
                    &basename[pos..]
                };

                let new_path = path.with_file_name(format!("{}.{}", digits, map_ext(ext)));

                fs::rename(path, new_path).await?;
            }
        }
        Ok(self)
    }

    pub async fn convert(&self) -> anyhow::Result<&Self> {
        for config_path in self.config_paths.iter() {
            let reader = File::open(&config_path).await?;
            let raw: RawConfig1 = serde_yaml::from_reader(reader.into_std().await)?;
            let config: Box<dyn Config> = Box::new(raw);
            let target = CasesConfig::try_from(config)?;
            let parent_dir = config_path.parent().expect("No parent directory");
            let toml_path = parent_dir.join("config.toml");
            let mut toml_file = File::create(&toml_path).await?;

            toml_file
                .write_all(toml::to_string(&target)?.as_bytes())
                .await?;
        }

        Ok(self)
    }

    pub async fn tar(&self, output_path: impl AsRef<Path>) -> anyhow::Result<&Self> {
        let tar_file = output_path.as_ref().join("config.tar.zst");
        fs::create_dir_all(&output_path).await?;
        let file = File::create(&tar_file).await?;
        let encoder = zstd::Encoder::new(file.into_std().await, 1)?.auto_finish();
        let mut tar_builder = tar::Builder::new(encoder);
        tar_builder.append_dir_all("config", &self.temp_dir)?;
        tar_builder.finish()?;

        Ok(self)
    }
}

async fn find_zip_file(input_path: impl AsRef<Path>) -> anyhow::Result<String> {
    for mut entry in fs::read_dir(input_path).await.into_iter() {
        let path = entry
            .next_entry()
            .await?
            .expect("Should have an entry")
            .path();
        if let Some(extension) = path.extension() {
            if extension == "zip" {
                return Ok(path.to_string_lossy().into_owned());
            }
        }
    }

    anyhow::bail!("No .zip file found in the directory")
}

async fn extract_config_file(
    zip_file: impl AsRef<Path>,
    temp_dir: impl AsRef<Path>,
) -> anyhow::Result<Vec<PathBuf>> {
    let file = fs::File::open(zip_file).await?;
    ZipArchive::new(file.into_std().await)?.extract(&temp_dir)?;

    let mut yaml_files = Vec::new();

    let mut entries = WalkDir::new(&temp_dir);
    while let Some(entry) = entries.try_next().await? {
        let path = entry.path();
        if path.file_name() == Some(OsStr::new("config.yaml"))
            || path.file_name() == Some(OsStr::new("config.yml"))
        {
            yaml_files.push(path);
        }
    }

    eprintln!("Found {} YAML files", yaml_files.len());

    Ok(yaml_files)
}
