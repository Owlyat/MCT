use core::panic;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

const PAPERMC_API: &str = "https://api.papermc.io";
/// arg 1 : Game version
const PAPERMC_API_BUILDS: &[&str; 2] = &["/v2/projects/paper/versions/", "/builds"];
/// arg 1 : Project | arg 2 : game version | arg 3 : build | arg 4 : download (ex : paper-1.21.4-1.jar)
const PAPERMC_API_DOWNLOAD_BUILD: &[&str; 4] =
    &["/v2/projects/", "/versions/", "/builds/", "/downloads/"];
#[derive(Serialize, Deserialize)]
pub struct PaperMCBuild {
    project: Option<String>,
    game_version: Option<String>,
    build: Option<i64>,
    download: Option<String>,
    response: Option<Value>,
    server_path: Option<PathBuf>,
}

impl PaperMCBuild {
    pub fn build() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn get_version(&mut self) -> Option<String> {
        if self.game_version.is_some() {
            Some(self.game_version.clone().unwrap())
        } else {
            None
        }
    }
    pub fn get_server_dir(&mut self) -> Option<PathBuf> {
        if self.server_path.is_some() {
            self.server_path.clone()
        } else {
            None
        }
    }
}

impl Default for PaperMCBuild {
    fn default() -> Self {
        Self {
            project: None,
            game_version: None,
            build: None,
            download: None,
            response: None,
            server_path: None,
        }
    }
}
impl PaperMCBuild {
    pub async fn check_build(&mut self, game_version: String, build: Option<String>) {
        let url = if build.is_none() {
            format!(
                "{}{}{}{}",
                PAPERMC_API, PAPERMC_API_BUILDS[0], game_version, PAPERMC_API_BUILDS[1]
            )
        } else {
            format!(
                "{}{}{}{}{}",
                PAPERMC_API,
                PAPERMC_API_BUILDS[0],
                game_version,
                PAPERMC_API_BUILDS[1],
                build.clone().unwrap()
            )
        };
        let response = reqwest::get(url).await;

        match response {
            Ok(response) => {
                if let Ok(json) = response.json::<Value>().await {
                    if let Some(builds) = json["builds"].as_array() {
                        for (index, build) in builds.iter().enumerate() {
                            println!("index : {}", index);
                            //println!("{:#?}", build);
                            if let Some(build_num) = build["build"].as_number() {
                                println!("Build : {}", build_num);
                            }
                            if let Some(download) = build["downloads"].as_object() {
                                if let Some(application) = download["application"].as_object() {
                                    if let Some(name) = application["name"].as_str() {
                                        println!("{}", name);
                                    }
                                }
                            }
                            println!()
                        }
                        loop {
                            println!("Please select a build : ");
                            let mut input = String::new();
                            std::io::stdin().read_line(&mut input).unwrap();
                            let selected = match input.trim().parse::<usize>() {
                                Ok(v) => v,
                                Err(_) => continue,
                            };
                            for (index, build) in builds.iter().enumerate() {
                                if selected == index {
                                    self.project = Some("paper".to_owned());
                                    self.game_version = Some(game_version.clone());
                                    if let Some(build_num) = build["build"].as_number() {
                                        self.build = Some(build_num.as_i64().unwrap());
                                    }
                                    if let Some(download) = build["downloads"].as_object() {
                                        if let Some(application) =
                                            download["application"].as_object()
                                        {
                                            if let Some(name) = application["name"].as_str() {
                                                self.download = Some(name.to_owned());
                                                println!("You choose : {}", name);
                                            }
                                        }
                                    }
                                    return;
                                }
                            }
                        }
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

impl PaperMCBuild {
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
                                    Err(e) => panic!("{}", e),
                                };
                                match fs::write(
                                    server_path.join(self.download.clone().unwrap()),
                                    content,
                                ) {
                                    Ok(_) => {
                                        println! {"Downloaded : {}", server_path.join(self.download.clone().unwrap()).to_string_lossy()}
                                        self.server_path =
                                            Some(server_path.join(self.download.clone().unwrap()));
                                    }
                                    Err(_) => todo!(),
                                }
                            }
                            Err(e) => panic!("{}", e),
                        }
                    } else {
                        todo!("Missing Download")
                    }
                } else {
                    todo!("Missing Build")
                }
            } else {
                todo!("Missing Game Version")
            }
        } else {
            todo!("Missing Project")
        }
    }
}

impl PaperMCBuild {
    pub fn start_server(&mut self, xmx: Option<String>, xms: Option<String>, is_gui: Option<bool>) {
        let mut java_args: Vec<String> = vec![];
        let mut startup_script = String::from("java ");
        if let Some(xmx) = &xmx {
            java_args.push(format!("-Xmx{}", xmx));
            startup_script.push_str(&format!("-Xmx{}", xmx));
        }
        if let Some(xms) = xms {
            java_args.push(format!("-Xms{}", xms));
            startup_script.push_str(&format!("-Xms{}", xms));
        }

        match fs::write(
            self.server_path
                .clone()
                .unwrap()
                .to_string_lossy()
                .replace(&self.download.clone().unwrap(), "eula.txt"),
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
                    .to_string_lossy()
                    .replace(&self.download.clone().unwrap(), "eula.txt");
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

        java_args.push(format!("-jar"));
        java_args.push(format!("{}", self.download.clone().unwrap()));
        startup_script.push_str(&format!("-jar {}", self.download.clone().unwrap()));
        if is_gui.is_some() && !is_gui.unwrap() || is_gui.is_none() {
            java_args.push(format!("-nogui"));
            startup_script.push_str(" -nogui");
        }

        let path = self
            .server_path
            .clone()
            .unwrap()
            .to_string_lossy()
            .replace(&self.download.clone().unwrap(), "start.bat");
        fs::write(Path::new(&path), startup_script.clone()).unwrap();
        let data_path = self
            .server_path
            .clone()
            .unwrap()
            .to_string_lossy()
            .replace(&self.download.clone().unwrap(), "MCA.json");
        fs::write(
            data_path,
            serde_json::to_string_pretty(&self).unwrap().as_bytes(),
        )
        .unwrap();
        match std::process::Command::new("java")
            .args(java_args.clone())
            .current_dir(PathBuf::from(format!(
                "{}",
                self.server_path
                    .clone()
                    .unwrap()
                    .to_string_lossy()
                    .replace(&self.download.clone().unwrap(), "")
            )))
            .spawn()
        {
            Ok(_) => {}
            Err(e) => {
                println!("{:#?}", java_args);
                panic!("{}", e);
            }
        };
    }
}

impl PaperMCBuild {
    pub fn check_data(&mut self, path: PathBuf) -> Result<(), ()> {
        let potential_data = path.join("MCA.json");
        match &mut fs::File::open(potential_data) {
            Ok(data) => {
                let mut input = String::new();
                data.read_to_string(&mut input).unwrap();
                let paper: PaperMCBuild = serde_json::from_str(&input).unwrap();
                self.project = paper.project;
                self.game_version = paper.game_version;
                self.build = paper.build;
                self.download = paper.download;
                self.server_path = paper.server_path;
                Ok(())
            }
            Err(_) => Err(()),
        }
    }
}
