use reqwest::Error;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

const FABRICMC_API_GAME_VERSIONS: &str = "https://meta.fabricmc.net/v2/versions/game";
const FABRICMC_API_LOADER_VERSIONS: &str = "https://meta.fabricmc.net/v2/versions/loader";
const FABRICMC_API_INSTALLER_VERSIONS: &str = "https://meta.fabricmc.net/v2/versions/installer";
const FABRICMC_API_DOWNLOAD: &str = "https://meta.fabricmc.net/v2/versions/loader";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct GameVersion {
    version: String,
    stable: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LoaderVersion {
    version: String,
    stable: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct InstallerVersion {
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FabricMCClient {
    project: String,
    game_version: Option<String>,
    loader_version: Option<String>,
    installer_version: Option<String>,
    download_url: Option<String>,
    server_path: Option<PathBuf>,
}

impl FabricMCClient {
    /// Creates a new instance of `FabricMCClient`
    pub fn build(server_path: Option<PathBuf>) -> Self {
        if server_path.is_some() {
            println!("Found : {}", server_path.clone().unwrap().to_string_lossy())
        }
        Self {
            project: "fabric".to_string(),
            game_version: None,
            loader_version: None,
            installer_version: None,
            download_url: None,
            server_path,
        }
    }

    /// Fetch available game versions
    async fn fetch_game_versions(&self) -> Result<Vec<GameVersion>, Error> {
        let response = reqwest::get(FABRICMC_API_GAME_VERSIONS)
            .await?
            .json::<Vec<GameVersion>>()
            .await?;
        Ok(response)
    }

    /// Fetch available Fabric Loader versions
    async fn fetch_loader_versions(&self) -> Result<Vec<LoaderVersion>, Error> {
        let response = reqwest::get(FABRICMC_API_LOADER_VERSIONS)
            .await?
            .json::<Vec<LoaderVersion>>()
            .await?;
        Ok(response)
    }

    /// Fetch available installer versions
    async fn fetch_installer_versions(&self) -> Result<Vec<InstallerVersion>, Error> {
        let response = reqwest::get(FABRICMC_API_INSTALLER_VERSIONS)
            .await?
            .json::<Vec<InstallerVersion>>()
            .await?;
        Ok(response)
    }

    /// Prompts the user to select a game version
    pub async fn select_game_version(&mut self, game_version: Option<String>) {
        let versions = self.fetch_game_versions().await.unwrap_or_default();
        if versions.is_empty() {
            println!("❌ No game versions found.");
            return;
        }

        if game_version.is_some() {
            let gv = game_version.unwrap();
            for entry in versions.iter() {
                if gv == entry.version {
                    self.game_version = Some(entry.version.clone());
                    println!(
                        "✅ Selected Game Version: {}",
                        self.game_version.as_ref().unwrap()
                    );
                    return;
                }
            }
        }

        println!("\n📌 Available Minecraft Versions:");
        for (index, entry) in versions.iter().enumerate().rev() {
            println!("{}: {}", index, entry.version);
        }

        loop {
            print!("\n➡️ Enter the number of the game version you want: ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            match input.trim().parse::<usize>() {
                Ok(num) if num < versions.len() => {
                    self.game_version = Some(versions[num].version.clone());
                    println!(
                        "✅ Selected Game Version: {}",
                        self.game_version.as_ref().unwrap()
                    );
                    break;
                }
                _ => println!("❌ Invalid input. Try again."),
            }
        }
    }

    /// Prompts the user to select a loader version
    pub async fn select_loader_version(&mut self) {
        let versions = self.fetch_loader_versions().await.unwrap_or_default();
        if versions.is_empty() {
            println!("❌ No loader versions found.");
            return;
        }

        println!("\n📌 Available Fabric Loader Versions:");
        for (index, entry) in versions.iter().enumerate().rev() {
            println!("{}: {}", index, entry.version);
        }

        loop {
            print!("\n➡️ Enter the number of the loader version you want: ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            match input.trim().parse::<usize>() {
                Ok(num) if num < versions.len() => {
                    self.loader_version = Some(versions[num].version.clone());
                    println!(
                        "✅ Selected Loader Version: {}",
                        self.loader_version.as_ref().unwrap()
                    );
                    break;
                }
                _ => println!("❌ Invalid input. Try again."),
            }
        }
    }

    /// Fetches the latest installer version
    pub async fn fetch_latest_installer_version(&mut self) {
        let versions = self.fetch_installer_versions().await.unwrap_or_default();
        if versions.is_empty() {
            println!("❌ No installer versions found.");
            return;
        }

        let latest_installer = &versions[0]; // Latest version is usually at index 0
        self.installer_version = Some(latest_installer.version.clone());
        println!("✅ Latest Installer Version: {}", latest_installer.version);
    }

    /// Constructs the download URL using the selected versions
    pub fn generate_download_url(&mut self) {
        if let (Some(ref game), Some(ref loader), Some(ref installer)) = (
            &self.game_version,
            &self.loader_version,
            &self.installer_version,
        ) {
            self.download_url = Some(format!(
                "{}/{}/{}/{}/server/jar",
                FABRICMC_API_DOWNLOAD, game, loader, installer
            ));
            println!("\n🔗 Download URL: {}", self.download_url.as_ref().unwrap());
        } else {
            println!("❌ Cannot generate download URL, missing values.");
            if let Some(ref game) = &self.game_version {
                println!("Game : {}", game)
            }
            if let Some(ref loader) = &self.loader_version {
                println!("Loader : {}", loader)
            }
            if let Some(ref installer) = &self.installer_version {
                println!("Installer : {}", installer)
            }
        }
    }

    /// Downloads the FabricMC server JAR
    pub async fn download_build(&self, server_path: PathBuf) {
        if let Some(url) = &self.download_url {
            println!("⬇️ Downloading FabricMC Server...");
            let response = reqwest::get(url).await;

            match response {
                Ok(res) => {
                    let content = res.bytes().await.unwrap();
                    let jar_path = server_path.join("fabric-server.jar");
                    fs::write(&jar_path, content).expect("❌ Failed to save the JAR file");
                    println!("✅ Downloaded: {}", jar_path.to_string_lossy());
                }
                Err(e) => println!("❌ Download failed: {}", e),
            }
        }
    }

    /// Starts the Fabric server
    pub fn start_server(&self, xmx: Option<String>, xms: Option<String>, is_gui: Option<bool>) {
        let mut java_args: Vec<String> = vec![];
        let mut startup_script = String::from("java ");
        match fs::write(
            self.server_path.clone().unwrap().join("eula.txt"),
            "eula=true",
        ) {
            Ok(_) => {
                println!("Eula are agreed !")
            }
            Err(_) => {
                let value: String = self
                    .server_path
                    .clone()
                    .unwrap()
                    .join("eula.txt")
                    .to_string_lossy()
                    .into_owned();
                let eula_path = Path::new(&value);
                if eula_path.is_file() {
                    println!("Eula already generated !");
                    match fs::read_to_string(eula_path) {
                        Ok(v) => {
                            let eula_str = v.replace("false", "true");
                            fs::write(Path::new(&value), eula_str).unwrap();
                        }
                        Err(_) => {
                            println!("Error reading eula file !")
                        }
                    }
                }
            }
        }

        let data_path = self.server_path.clone().unwrap().join("MCA.json");
        fs::write(
            data_path,
            serde_json::to_string_pretty(&self).unwrap().as_bytes(),
        )
        .unwrap();

        if let Some(xmx) = xmx {
            java_args.push(format!("-Xmx{}", xmx));
            startup_script.push_str(&format!("-Xmx{}", xmx));
        }
        if let Some(xms) = xms {
            java_args.push(format!("-Xms{}", xms));
            startup_script.push_str(&format!("-Xms{}", xms));
        }

        java_args.push("-jar".to_string());
        java_args.push("fabric-server.jar".to_string());
        startup_script.push_str("-jar fabric-server.jar");

        if is_gui.is_some() && !is_gui.unwrap() || is_gui.is_none() {
            java_args.push("-nogui".to_string());
            startup_script.push_str(" -nogui");
        }

        let path = self.server_path.as_ref().unwrap().join("start.bat");
        fs::write(&path, startup_script).unwrap();

        println!("🚀 Starting Fabric Server...");
        std::process::Command::new("java")
            .args(&java_args)
            .current_dir(self.server_path.as_ref().unwrap())
            .spawn()
            .expect("❌ Failed to start server.");
    }
}

impl FabricMCClient {
    pub fn get_version(&self) -> Option<String> {
        self.game_version.clone()
    }
    pub fn get_download_path(&self) -> Option<PathBuf> {
        if self.server_path.is_some() {
            Some(
                Path::new(&format!(
                    "{}\\mods",
                    self.server_path.clone().unwrap().to_string_lossy()
                ))
                .to_path_buf(),
            )
        } else {
            None
        }
    }
}

impl FabricMCClient {
    pub fn check_data(&mut self, path: Option<PathBuf>) -> Result<(), ()> {
        let path = if path.is_some() {
            path.unwrap()
        } else {
            if self.server_path.is_some() {
                self.server_path.clone().unwrap()
            } else {
                panic!("No Server Path Directory found !")
            }
        };
        let potential_data = path.join("MCA.json");
        match &mut fs::File::open(potential_data) {
            Ok(data) => {
                let mut input = String::new();
                data.read_to_string(&mut input).unwrap();
                let fabric: FabricMCClient = serde_json::from_str(&input).unwrap();
                self.project = fabric.project;
                self.game_version = fabric.game_version;
                self.loader_version = fabric.loader_version;
                self.installer_version = fabric.installer_version;
                self.download_url = fabric.download_url;

                Ok(())
            }
            Err(_) => Err(()),
        }
    }
}
