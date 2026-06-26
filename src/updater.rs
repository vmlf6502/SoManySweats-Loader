use regex::Regex;
use self_update::backends::github;
use self_update::update::Release;
use std::fs::{create_dir_all, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use zip::ZipArchive;

const REPO_OWNER: &str = "vmlf6502";
const REPO_NAME: &str = "SoManySweats";
const JAR_PATTERN: &str = r"SoManySweats-v\d+\.\d+\.\d+\.jar";

pub enum VersionStatus {
    NotFound,
    OutOfDate,
    UpToDate,
}

fn mod_dir() -> PathBuf {
    let dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".lunarclient/offline/multiver/somanysweats");

    create_dir_all(&dir).expect("Failed to create somanysweats directory.");

    dir
}

pub struct Updater {
    pub status: VersionStatus,
    pub current_ver: String,
    pub release_ver: String,
    pub release: Option<Release>,
}

impl Default for Updater {
    fn default() -> Self {
        Self::new()
    }
}

impl Updater {
    pub fn new() -> Self {
        match Self::try_new() {
            Ok(u) => u,
            Err(e) => {
                eprintln!("Failed to initialize updater: {e}");
                Self {
                    current_ver: "Unknown".to_string(),
                    release_ver: "Latest".to_string(),
                    status: VersionStatus::NotFound,
                    release: None,
                }
            }
        }
    }
    fn try_new() -> Result<Self, Box<dyn std::error::Error>> {
        let release = github::ReleaseList::configure()
            .repo_owner(REPO_OWNER)
            .repo_name(REPO_NAME)
            .build()?
            .fetch()?
            .into_iter()
            .next()
            .ok_or("No releases found")?;

        let release_ver = release.version.clone();
        let mut current_ver = "Unknown".to_string();

        let status = match Self::find_existing_jar() {
            None => VersionStatus::NotFound,
            Some(jar_path) => {
                current_ver = Self::get_jar_version(&jar_path)?;
                if current_ver == release_ver {
                    VersionStatus::UpToDate
                } else {
                    VersionStatus::OutOfDate
                }
            }
        };

        Ok(Self {
            status,
            current_ver,
            release_ver,
            release: Some(release),
        })
    }

    pub fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pattern = Regex::new(JAR_PATTERN)?;
        let release = self.release.as_ref().ok_or("No release available")?;
        let asset = release
            .assets
            .iter()
            .find(|a| pattern.is_match(&a.name))
            .ok_or("No matching asset found in latest release")?;

        match Self::find_existing_jar() {
            None => {
                Self::download_jar(&release.version, &asset.download_url, &asset.name)?;
            }
            Some(jar_path) => {
                let current = Self::get_jar_version(&jar_path)?;
                if current != release.version {
                    std::fs::remove_file(&jar_path)?;
                    Self::download_jar(&release.version, &asset.download_url, &asset.name)?;
                }
            }
        }

        Ok(())
    }

    fn find_existing_jar() -> Option<PathBuf> {
        let pattern = Regex::new(JAR_PATTERN).unwrap();
        std::fs::read_dir(mod_dir())
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| pattern.is_match(n))
                    .unwrap_or(false)
            })
    }

    fn get_jar_version(jar_path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
        let mut archive = ZipArchive::new(File::open(jar_path)?)?;
        let manifest = archive.by_name("META-INF/MANIFEST.MF")?;

        for line in BufReader::new(manifest).lines() {
            let line = line?;
            if let Some(version) = line.strip_prefix("Implementation-Version: ") {
                return Ok(version.trim().to_string());
            }
        }

        Ok("Unknown".to_string())
    }

    fn download_jar(
        _version: &str,
        asset_url: &str,
        asset_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dest = mod_dir().join(asset_name);
        let mut file = File::create(&dest)?;

        self_update::Download::from_url(asset_url)
            .set_header(reqwest::header::ACCEPT, "application/octet-stream".parse()?)
            .download_to(&mut file)?;

        Ok(())
    }
}
