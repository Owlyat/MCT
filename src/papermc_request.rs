use core::panic;
use inquire::Select;
use std::{fs, io::Read, path::PathBuf, process::Stdio};

use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

const PAPERMC_API: &str = "https://api.papermc.io";
/// arg 1 : Game version
const PAPERMC_API_BUILDS: &[&str; 2] = &["/v2/projects/paper/versions/", "/builds"];
/// arg 1 : Project | arg 2 : game version | arg 3 : build | arg 4 : download (ex : paper-1.21.4-1.jar)
const PAPERMC_API_DOWNLOAD_BUILD: &[&str; 4] =
    &["/v2/projects/", "/versions/", "/builds/", "/downloads/"];
#[derive(Serialize, Deserialize)]
pub struct PaperMCRequest {
    project: Option<String>,
    game_version: Option<String>,
    build: Option<i64>,
    download: Option<String>,
    response: Option<Value>,
    server_path: Option<PathBuf>,
    jar_path: Option<PathBuf>,
}

impl PaperMCRequest {
    pub fn build() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn _get_version(&mut self) -> Option<String> {
        if self.game_version.is_some() {
            Some(self.game_version.clone().unwrap())
        } else {
            None
        }
    }
    pub fn _get_server_dir(&mut self) -> Option<PathBuf> {
        if self.server_path.is_some() {
            self.server_path.clone()
        } else {
            None
        }
    }
}

impl Default for PaperMCRequest {
    fn default() -> Self {
        Self {
            project: None,
            game_version: None,
            build: None,
            download: None,
            response: None,
            server_path: None,
            jar_path: None,
        }
    }
}
impl PaperMCRequest {
    pub async fn check_build(&mut self, game_version: Option<String>, build: Option<String>) {
        if game_version.is_none() {
            println!("❌ No game version provided !");
            return;
        }
        let url = if build.is_none() {
            format!(
                "{}{}{}{}",
                PAPERMC_API,
                PAPERMC_API_BUILDS[0],
                game_version.clone().unwrap(),
                PAPERMC_API_BUILDS[1]
            )
        } else {
            format!(
                "{}{}{}{}{}",
                PAPERMC_API,
                PAPERMC_API_BUILDS[0],
                game_version.clone().unwrap(),
                PAPERMC_API_BUILDS[1],
                build.clone().unwrap()
            )
        };
        println!("➡️ Fetching from : {}", url);
        let response = reqwest::get(url.clone()).await;

        match response {
            Ok(response) => {
                if let Ok(json) = response.json::<Value>().await {
                    if let Some(builds) = json["builds"].as_array() {
                        let options: Vec<(&Number, String)> = builds
                            .iter()
                            .rev()
                            .map(|b| {
                                (
                                    b["build"].as_number().unwrap(),
                                    b["downloads"].as_object().unwrap()["application"]
                                        .as_object()
                                        .unwrap()["name"]
                                        .as_str()
                                        .unwrap()
                                        .to_owned(),
                                )
                            })
                            .collect();
                        let selected_build = Select::new(
                            "➡️ Select Build Number",
                            options.iter().map(|b| b.1.clone()).collect(),
                        )
                        .prompt()
                        .unwrap();
                        let selected_build = builds.into_iter().find(|b| {
                            if b["downloads"].as_object().unwrap()["application"]
                                .as_object()
                                .unwrap()["name"]
                                .as_str()
                                .unwrap()
                                == selected_build
                            {
                                true
                            } else {
                                false
                            }
                        });
                        selected_build.iter().for_each(|b| {
                            self.build = Some(b["build"].as_number().unwrap().as_i64().unwrap());
                            self.project = Some(String::from("paper"));
                            self.game_version = Some(game_version.clone().unwrap());
                            self.download = Some(
                                b["downloads"].as_object().unwrap()["application"]
                                    .as_object()
                                    .unwrap()["name"]
                                    .as_str()
                                    .unwrap()
                                    .to_owned(),
                            );
                        });
                        return;
                    } else {
                        panic!(
                            "❌ No builds found from response !\n    ⏬ Response from url {} ⏬\n{:#?}",
                            url, json
                        );
                    }
                }
            }
            Err(e) => {
                println!("{}", e);
                return;
            }
        }
    }
}

impl PaperMCRequest {
    pub async fn download_build(&mut self, server_path: PathBuf) {
        if self.project.is_some() {
            if self.game_version.is_some() {
                if self.build.is_some() {
                    if self.download.is_some() {
                        let download_url = format!(
                            "{}{}{}{}{}{}{}{}{}",
                            PAPERMC_API,
                            PAPERMC_API_DOWNLOAD_BUILD[0],
                            self.project.clone().unwrap(),
                            PAPERMC_API_DOWNLOAD_BUILD[1],
                            self.game_version.clone().unwrap(),
                            PAPERMC_API_DOWNLOAD_BUILD[2],
                            self.build.clone().unwrap(),
                            PAPERMC_API_DOWNLOAD_BUILD[3],
                            self.download.clone().unwrap(),
                        );

                        let response = reqwest::get(download_url).await;
                        match response {
                            Ok(res) => {
                                let content = match res.bytes().await {
                                    Ok(bytes) => bytes,
                                    Err(e) => panic!("❌ Error while writting bytes : {}", e),
                                };
                                self.server_path = Some(server_path);
                                self.jar_path = Some(
                                    self.server_path
                                        .clone()
                                        .unwrap()
                                        .join(self.download.clone().unwrap()),
                                );
                                match fs::write(self.jar_path.clone().unwrap(), content) {
                                    Ok(_) => {
                                        println! {"✅ Downloaded : {}", self.jar_path.clone().unwrap().to_string_lossy().replace("\\", "/")}
                                    }
                                    Err(e) => panic!("❌ Error while writting file : {}", e),
                                }
                            }
                            Err(e) => panic!("❌ Error with the web request : {}", e),
                        }
                    } else {
                        panic!("❌ Missing Download!")
                    }
                } else {
                    panic!("❌ Missing Build!")
                }
            } else {
                panic!("❌ Missing Game Version!")
            }
        } else {
            panic!("❌ Missing Project!")
        }
    }
}

impl PaperMCRequest {
    pub fn start_server(&mut self, xmx: Option<String>, xms: Option<String>, is_gui: Option<bool>) {
        let mut java_args: Vec<String> = vec!["-jar".to_owned()];
        java_args.push(self.download.clone().unwrap());
        if let Some(xms) = xms {
            java_args.insert(0, format!("-Xms{}", xms));
        }
        if let Some(xmx) = &xmx {
            java_args.insert(0, format!("-Xmx{}", xmx));
        }
        // Ensure the EULA is accepted
        let eula_path = self.server_path.as_ref().unwrap().join("eula.txt");

        if let Err(_) = fs::write(&eula_path, "eula=true") {
            if let Ok(eula_content) = fs::read_to_string(&eula_path) {
                let updated_eula = eula_content.replace("false", "true");
                let _ = fs::write(&eula_path, updated_eula);
            }
        }

        if is_gui.is_none() || !is_gui.unwrap() {
            java_args.push("-nogui".to_owned());
        }

        let data_path = self.server_path.clone().unwrap().join("MCA.json");
        fs::write(
            data_path,
            serde_json::to_string_pretty(&self).unwrap().as_bytes(),
        )
        .unwrap();
        match std::process::Command::new("java")
            .args(&java_args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .current_dir(self.server_path.clone().unwrap())
            .spawn()
        {
            Ok(_) => {}
            Err(e) => {
                println!("➡️ Command: java {:?}", java_args);
                panic!("    ❌➡️{}", e);
            }
        };
    }
}
/// Check MCA.json and sets the Paper server values if found
impl PaperMCRequest {
    pub fn check_data(&mut self, path: PathBuf) -> Result<(), ()> {
        let potential_data = path.join("MCA.json");
        match &mut fs::File::open(potential_data) {
            Ok(data) => {
                let mut input = String::new();
                data.read_to_string(&mut input).unwrap();
                let paper: PaperMCRequest = serde_json::from_str(&input).unwrap();
                self.project = paper.project;
                self.game_version = paper.game_version;
                self.build = paper.build;
                self.download = paper.download;
                self.server_path = paper.server_path;
                self.jar_path = paper.jar_path;
                Ok(())
            }
            Err(_) => Err(()),
        }
    }
}
