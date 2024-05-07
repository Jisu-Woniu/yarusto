use crate::model::CasesConfig;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::ZipArchive;

pub struct Converter {
    pub configs: Vec<CasesConfig>,
}

const TMP_DIR: &str = "./tmp";

impl Converter {
    pub fn build(input_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let zip_file = find_zip_file(input_path)?;

        let config_path = extract_config_file(&zip_file)?;

        let mut configs = Vec::new();

        for path in config_path {
            let reader = fs::File::open(&path)?;
            let config = serde_yaml::from_reader(reader)?;
            eprintln!("{:?}", config);
            configs.push(config);
        }

        Ok(Self { configs })
    }

    pub fn rename(&self) -> anyhow::Result<&Self> {
        Ok(self)
    }

    pub fn tar(&self, output_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let tar_file = output_path.as_ref().join("config.tar");
        tar::Builder::new(fs::File::create(tar_file)?).append_dir_all(TMP_DIR, &output_path)?;
        Ok(())
    }
}

fn find_zip_file(input_path: impl AsRef<Path>) -> anyhow::Result<String> {
    let entries = fs::read_dir(input_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension == "zip" {
                return Ok(path.to_string_lossy().into_owned());
            }
        }
    }

    anyhow::bail!("No .zip file found in the directory")
}

fn extract_config_file(zip_file: impl AsRef<Path>) -> anyhow::Result<Vec<PathBuf>> {
    let file = fs::File::open(zip_file)?;
    ZipArchive::new(file)?.extract(TMP_DIR)?;

    let mut yaml_files = Vec::new();

    let _ = WalkDir::new(TMP_DIR)
        .into_iter()
        .filter_map(Result::ok)
        .find(|entry| {
            let path = entry.path();
            path.extension()
                .map_or(false, |ext| ext == "yaml" || ext == "yml")
        })
        .map(|entry| entry.into_path())
        .ok_or_else(|| anyhow::anyhow!("config.yaml not found in the zip file"))
        .map(|path| yaml_files.push(path));

    eprintln!("Found {} YAML files", yaml_files.len());
    eprintln!("{:?}", yaml_files);

    Ok(yaml_files)
}
