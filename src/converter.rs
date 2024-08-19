use std::{
    borrow::Cow,
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
    str,
};

use async_walkdir::WalkDir;
use tempfile::TempDir;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tokio_stream::StreamExt;
use zip::ZipArchive;

use crate::model::{
    cases_config::CasesConfig, config::Config, raw::config1::ConfigData as Config1,
};

pub struct Converter {
    config_paths: Vec<PathBuf>,
    temp_dir: TempDir,
}

impl Converter {
    pub async fn with_input_path(input_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        async fn inner(input_path: &Path) -> anyhow::Result<Converter> {
            let zip_file = find_zip_file(input_path).await?;

            if let Some(path) = zip_file {
                let temp_dir = TempDir::new()?;

                let config_paths = extract_config_file(path, &temp_dir).await?;

                Ok(Converter {
                    config_paths,
                    temp_dir,
                })
            } else {
                Err(io::Error::from(io::ErrorKind::NotFound))?
            }
        }
        inner(input_path.as_ref()).await
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

            if let (Some(stem), Some(ext)) = (path.file_stem(), path.extension()) {
                if let (stem, ext @ (b"in" | b"out" | b"ans")) =
                    (stem.as_encoded_bytes(), ext.as_encoded_bytes())
                {
                    let digits = if let Some(pos) = stem.iter().rposition(|b| !b.is_ascii_digit()) {
                        &stem[pos..]
                    } else {
                        stem
                    };

                    // SAFETY: `digits` is composed of ASCII digits only
                    let new_stem = unsafe { str::from_utf8_unchecked(digits) };

                    // SAFETY: `ext` is one of `"in"` / `"out"` / `"ans"`, which are all ASCII-only
                    let new_ext = map_ext(unsafe { str::from_utf8_unchecked(ext) });
                    let new_path = path.with_file_name(format!("{new_stem}.{new_ext}"));
                    fs::rename(&path, new_path).await?;
                }
            }
        }
        Ok(self)
    }

    pub async fn convert(&self) -> anyhow::Result<&Self> {
        for config_path in self.config_paths.iter() {
            let reader = File::open(&config_path).await?;

            // TODO: Erase the concrete type here.
            let raw: Config1 = serde_yml::from_reader(reader.into_std().await)?;
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

    pub async fn tar(&self, output_path: impl AsRef<Path>) -> anyhow::Result<()> {
        async fn inner(temp_dir: &TempDir, output_path: &Path) -> anyhow::Result<()> {
            let tar_file = output_path.join("config.tar.zst");
            fs::create_dir_all(&output_path).await?;
            let file = File::create(&tar_file).await?;
            let encoder = zstd::Encoder::new(file.into_std().await, 1)?.auto_finish();
            let mut tar_builder = tar::Builder::new(encoder);
            tar_builder.append_dir_all("config", temp_dir)?;
            tar_builder.finish()?;

            Ok(())
        }
        inner(&self.temp_dir, output_path.as_ref()).await
    }
}

async fn find_zip_file(path: impl AsRef<Path>) -> io::Result<Option<PathBuf>> {
    async fn inner(path: &Path) -> io::Result<Option<PathBuf>> {
        while let Some(entry) = fs::read_dir(path).await?.next_entry().await? {
            if let Some(ext) = entry.path().extension() {
                if ext == "zip" {
                    return Ok(Some(path.to_path_buf()));
                }
            }
        }
        Ok(None)
    }

    inner(path.as_ref()).await
}

async fn extract_config_file(
    zip_path: impl AsRef<Path>,
    temp_dir: impl AsRef<Path>,
) -> anyhow::Result<Vec<PathBuf>> {
    async fn inner(zip_path: &Path, temp_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
        let file = fs::File::open(zip_path).await?;
        ZipArchive::new(file.into_std().await)?.extract(temp_dir)?;

        let mut yaml_files = Vec::new();

        let mut entries = WalkDir::new(temp_dir);
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
    inner(zip_path.as_ref(), temp_dir.as_ref()).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
