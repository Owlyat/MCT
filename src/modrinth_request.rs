use inquire::Select;
const SEARCH_API_END_POINT: &str = "https://api.modrinth.com/v2/search";
const MOD_LOADERS: &[&str; 7] = &[
    "fabric",
    "forge",
    "quilt",
    "neoforge",
    "liteloader",
    "modloader",
    "rift",
];
const FILTERS: &[&str; 5] = &["relevance", "downloads", "follows", "newest", "updated"];
pub enum ModrinthSortingFilter {
    Relevance,
    Downloads,
    Follows,
    Newest,
    Updated,
}

impl ModrinthSortingFilter {
    pub fn with(maybe_filter: Option<impl Into<String>>) -> Option<Self> {
        match maybe_filter {
            Some(str) => {
                let filter: String = str.into();
                match filter.to_lowercase() {
                    value if value == FILTERS[0] => Some(ModrinthSortingFilter::Relevance),
                    value if value == FILTERS[1] => Some(ModrinthSortingFilter::Downloads),
                    value if value == FILTERS[2] => Some(ModrinthSortingFilter::Follows),
                    value if value == FILTERS[3] => Some(ModrinthSortingFilter::Newest),
                    value if value == FILTERS[4] => Some(ModrinthSortingFilter::Updated),
                    _ => None,
                }
            }
            None => None,
        }
    }
    fn get_filter(&self) -> &'static str {
        match self {
            ModrinthSortingFilter::Relevance => FILTERS[0],
            ModrinthSortingFilter::Downloads => FILTERS[1],
            ModrinthSortingFilter::Follows => FILTERS[2],
            ModrinthSortingFilter::Newest => FILTERS[3],
            ModrinthSortingFilter::Updated => FILTERS[4],
        }
    }
}

use std::path::PathBuf;

use serde::Serialize;
use serde_json::json;

use serde_json::Value;

#[derive(Serialize, Debug, Clone)]
pub struct ModrinthEntry {
    mod_name: Option<Box<str>>,
    mod_id: Option<Box<str>>,
    mod_version: Option<Box<str>>,
    mod_loader: Option<String>,
    response: Option<Value>,
    dependencies: Option<Value>,
}

impl ModrinthEntry {
    pub fn builder() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Use this if you want to search a modrinth mod
    pub async fn search_modrinth(
        &mut self,
        mod_name: impl Into<String>,
        mod_version: Option<impl Into<String>>,
        mod_loader: Option<impl Into<String>>,
        max_mod_number: Option<usize>,
        project_type: Option<String>,
        sorting: Option<ModrinthSortingFilter>,
        offset: Option<usize>,
        is_client_side: Option<bool>,
        is_server_side: Option<bool>,
    ) -> &mut Self {
        // this is just to call the function with Some("word")
        let target_mod_name: String = mod_name.into();

        // Creating the request with the mod name or mod id and if defined mod version, mod loader
        let query = format!("?query={}", target_mod_name);

        let mut facets: Vec<Value> = Vec::new();
        match project_type {
            Some(proj_type) => {
                facets.push(json!([format!("project_type:{}", proj_type)]));
            }
            None => {
                facets.push(json!([String::from("project_type:mod")]));
            }
        }

        // if any is empty, we don't push it to facets
        if mod_version.is_some() {
            facets.push(json!([format!("versions:{}", mod_version.unwrap().into())]));
        }
        if mod_loader.is_some() {
            let mod_loader: String = mod_loader.unwrap().into();
            MOD_LOADERS.iter().for_each(|loader| {
                if &mod_loader == loader {
                    facets.push(json!([format!("categories:{}", loader)]));
                }
            });
        }
        // setup the max number of mod to get
        let limit = format!(
            "&limit={}",
            if max_mod_number.is_some() {
                max_mod_number.unwrap()
            } else {
                10
            }
        );

        // setup the sorting filter
        let sorting = format!(
            "&index={}",
            if sorting.is_some() {
                sorting.unwrap().get_filter()
            } else {
                FILTERS[0]
            }
        );

        // Setup the offset number of mods displayed
        let offset = format!(
            "&offset={}",
            if offset.is_some() { offset.unwrap() } else { 0 }
        );

        // Filter client side mod required
        if is_client_side.is_some() {
            facets.push(json!([format!(
                "client_side:{}",
                if is_client_side.unwrap() {
                    "required"
                } else {
                    "optional"
                }
            )]));
        }

        // Filter server side mod required
        if is_server_side.is_some() {
            facets.push(json!([format!(
                "server_side:{}",
                if is_client_side.unwrap() {
                    "required"
                } else {
                    "optional"
                }
            )]));
        }

        // Building the URL to the API END POINT
        let url = {
            format!(
                "{}{}&facets={}{}{}{}",
                SEARCH_API_END_POINT,
                query,
                json!(facets),
                limit,
                sorting,
                offset,
            )
        };

        println!("{}", url);

        let response = reqwest::get(&url).await;

        // we keep track of the json answer in self
        match response {
            Ok(response) => {
                if let Ok(value) = response.json::<Value>().await {
                    self.response = Some(value)
                }
            }
            Err(e) => {
                println!("{}", e)
            }
        }
        self
    }
}

impl Default for ModrinthEntry {
    fn default() -> Self {
        Self {
            mod_name: None,
            mod_id: None,
            mod_version: None,
            mod_loader: None,
            response: None,
            dependencies: None,
        }
    }
}

impl ModrinthEntry {
    pub fn display_entries(&self) {
        if let Some(response) = &self.response {
            if let Some(mods) = response.get("hits").and_then(|hits| hits.as_array()) {
                println!("Found {} mods:", mods.len());
                for (index, mod_entry) in mods.iter().enumerate() {
                    let name = mod_entry
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown");
                    let id = mod_entry
                        .get("project_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown");
                    let project_type = mod_entry
                        .get("project_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown");
                    let desc = mod_entry
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("No description");
                    let game_versions = mod_entry
                        .get("versions")
                        .and_then(|v| v.as_array())
                        .map(|versions| {
                            versions
                                .iter()
                                .filter_map(|ver| ver.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_else(|| "Unknown".to_string());

                    println!(
                        "\n{}. Name: {}\n   ID: {}\n   Type: {}\n   Game Versions: {}\n   Description: {}",
                        index + 1, name, id, project_type, game_versions, desc.replace('\n',"" )
                    );
                }
            } else {
                println!("Nothing found in the response!");
            }
        } else {
            println!("No response to display from!");
        }
    }
}

impl ModrinthEntry {
    pub async fn download_mod(
        &mut self,
        mod_id: &mut Option<String>,
        mod_name: Option<String>,
        mod_loader: Option<String>,
        version: Option<String>,
        download_path: Option<PathBuf>,
        dependencies: Option<bool>,
    ) {
        let mut mod_id_or_name = mod_id
            .clone()
            .or_else(|| mod_name.clone())
            .expect("Either mod_id or mod_name must be provided");

        // If the first request fails, retry it once.
        let mut counter = 0;
        loop {
            // Construct the API endpoint
            let url = if mod_id.is_some() {
                format!(
                    "https://api.modrinth.com/v2/project/{}/version",
                    mod_id_or_name
                )
            } else {
                match (mod_loader.is_some(), version.is_some()) {
                    (true, true) => format!("{}?query=\"{}\"&facets=[[\"project_type:mod\"],[\"categories:{}\"],[\"versions:{}\"]]",SEARCH_API_END_POINT,mod_id_or_name,mod_loader.clone().unwrap(),version.clone().unwrap()),
                    (true, false) => format!("{}?query=\"{}\"&facets=[[\"project_type:mod\"],[\"categories:{}\"]]",SEARCH_API_END_POINT,mod_id_or_name,mod_loader.clone().unwrap()),
                    (false, true) => format!("{}?query=\"{}\"&facets=[[\"project_type:mod\"],[\"versions:{}\"]]",SEARCH_API_END_POINT,mod_id_or_name,version.clone().unwrap()),
                    (false, false) => format!("{}?query=\"{}\"&facets=[[\"project_type:mod\"]]",SEARCH_API_END_POINT,mod_id_or_name),
                }
            };

            println!("Fetching mod information from: {}", url);

            // Fetch the data
            let response = reqwest::get(&url).await;
            match response {
                Ok(res) => {
                    let data: Value = match res.json().await {
                        Ok(json) => {
                            if mod_loader.is_some() && self.mod_loader.is_none() {
                                self.mod_loader = mod_loader.clone();
                            }
                            json
                        }
                        Err(e) => {
                            println!("Failed to parse JSON response: {}", e);
                            if counter == 0 {
                                counter += 1;
                                continue;
                            }
                            return;
                        }
                    };

                    if self.mod_id.is_none() {
                        // If searching by name and multiple results are found
                        if let Some(mods) = data["hits"].as_array() {
                            if mods.len() > 1 {
                                let options: Vec<String> = mods
                                    .iter()
                                    .map(|m| {
                                        format!(
                                            "{}\nAuthor: {}\nDescription: {}",
                                            m["title"].as_str().unwrap(),
                                            m["author"].as_str().unwrap(),
                                            m["description"].as_str().unwrap(),
                                        )
                                    })
                                    .collect();
                                let selected_mod: String =
                                    Select::new("Please Select a mod ➡️", options)
                                        .prompt()
                                        .unwrap();

                                mods.iter().for_each(|m| {
                                    if selected_mod.starts_with(m["title"].as_str().unwrap())
                                        && selected_mod.contains(m["author"].as_str().unwrap())
                                        && selected_mod.contains(m["description"].as_str().unwrap())
                                    {
                                        mod_id_or_name = m["project_id"]
                                            .as_str()
                                            .expect("Mod ID not found")
                                            .to_string();

                                        self.mod_id = Some(mod_id_or_name.clone().into());
                                        *mod_id = Some(mod_id_or_name.clone());
                                    }
                                });
                                continue;
                            }
                        }
                    }
                    // Mod ID Selected
                    if let Some(files) = self.extract_files(&data, version.clone()).await {
                        for file in files {
                            let download_url = file.get("url").and_then(|u| u.as_str());
                            if let Some(download_url) = download_url {
                                println!("Downloading from: {}", download_url);

                                match reqwest::get(download_url).await {
                                    Ok(res) => {
                                        let content = match res.bytes().await {
                                            Ok(bytes) => bytes,
                                            Err(e) => {
                                                println!("Failed to read file content: {}", e);
                                                continue;
                                            }
                                        };

                                        let file_name = file
                                            .get("filename")
                                            .and_then(|f| f.as_str())
                                            .unwrap_or("mod_file.zip");

                                        // Determine the full path
                                        let full_path = download_path
                                            .clone()
                                            .unwrap_or_else(|| std::env::current_dir().unwrap())
                                            .join(file_name);

                                        // Save the file
                                        match std::fs::write(&full_path, content) {
                                            Ok(_) => {
                                                println!(
                                                    "Downloaded: {}",
                                                    full_path.to_string_lossy()
                                                );
                                                if dependencies.is_some() {
                                                    let do_download_dependencies =
                                                        dependencies.unwrap();
                                                    if do_download_dependencies {
                                                        self.verify_dependencies(
                                                            download_path.clone(),
                                                        )
                                                        .await;
                                                    }
                                                }
                                            }
                                            Err(e) => println!(
                                                "Failed to save file to {}: {}",
                                                full_path.to_string_lossy(),
                                                e
                                            ),
                                        }
                                    }
                                    Err(e) => println!("Failed to download file: {}", e),
                                }
                            }
                        }
                    } else {
                        println!("No suitable mod files found for the given criteria.");
                    }
                    return;
                }
                Err(e) => {
                    println!("Failed to fetch mod information: {}", e);
                    if counter == 0 {
                        counter += 1;
                        continue;
                    }
                    return;
                }
            }
        }
    }
}

impl ModrinthEntry {
    pub async fn download_server_mod(
        &mut self,
        mod_id: &mut Option<String>,
        mod_name: Option<String>,
        mod_loader: Option<String>,
        version: Option<String>,
        download_path: Option<PathBuf>,
        dependencies: Option<bool>,
    ) {
        let version = if let Some(v) = self.mod_version.clone() {
            Some(format!("{}", v))
        } else {
            version
        };
        let mut mod_id_or_name = mod_id
            .clone()
            .or_else(|| mod_name.clone())
            .expect("Either mod_id or mod_name must be provided");

        loop {
            // Construct the API endpoint
            let url = if mod_id.is_some() {
                format!(
                    "https://api.modrinth.com/v2/project/{}/version",
                    mod_id_or_name
                )
            } else {
                match (mod_loader.is_some(), version.is_some()) {
                    (true, true) => format!("{}?query={}&facets=[[\"project_type:mod\"],[\"categories:{}\"],[\"versions:{}\"]]",SEARCH_API_END_POINT,mod_id_or_name,mod_loader.clone().unwrap(),version.clone().unwrap()),
                    (true, false) => format!("{}?query={}&facets=[[\"project_type:mod\"],[\"categories:{}\"]]",SEARCH_API_END_POINT,mod_id_or_name,mod_loader.clone().unwrap()),
                    (false, true) => format!("{}?query={}&facets=[[\"project_type:mod\"],[\"versions:{}\"]]",SEARCH_API_END_POINT,mod_id_or_name,version.clone().unwrap()),
                    (false, false) => format!("{}?query={}&facets=[[\"project_type:mod\"]]",SEARCH_API_END_POINT,mod_id_or_name),
                }
            };

            println!("Fetching mod information from: {}", url);

            // Fetch the data
            let response = reqwest::get(&url).await;
            match response {
                Ok(res) => {
                    let data: Value = match res.json().await {
                        Ok(json) => {
                            if mod_loader.is_some() && self.mod_loader.is_none() {
                                self.mod_loader = mod_loader.clone();
                            }
                            json
                        }
                        Err(e) => {
                            println!("Failed to parse JSON response: {}", e);
                            return;
                        }
                    };

                    if self.mod_id.is_none() {
                        // If searching by name and multiple results are found
                        if let Some(mods) = &mut data["hits"].as_array() {
                            let mut filtered_mods: Vec<Value> = vec![];
                            mods.iter().for_each(|m| {
                                if let Some(m) = m.as_object() {
                                    if m.contains_key("server_side") {
                                        if matches!(m["server_side"].as_str().unwrap(), "required")
                                        {
                                            // do not return as the mod is pushed to the filtered_mods
                                        } else {
                                            if matches!(
                                                m["server_side"].as_str().unwrap(),
                                                "optional"
                                            ) {
                                                loop {
                                                    let options = vec![
                                                        String::from("Yes"),
                                                        String::from("No"),
                                                    ];
                                                    let do_download: String = Select::new(
                                                        &format!(
                                                            "➡️ Do you want this optional Server Side mod ? {}",
                                                            m.get("slug")
                                                                .unwrap()
                                                                .as_str()
                                                                .unwrap()
                                                        ),
                                                        options,
                                                    )
                                                    .prompt()
                                                    .unwrap();
                                                    if &do_download == "Yes" {
                                                        break; // break to push mod into filtered mods
                                                    } else {
                                                        return;
                                                    }
                                                }
                                                // do not return as the mod is pushed
                                            } else {
                                                return;
                                            }
                                        }
                                    }
                                }
                                filtered_mods.push(m.clone());
                            });
                            if filtered_mods.len() > 1 {
                                let options: Vec<String> = filtered_mods
                                    .iter()
                                    .map(|m| m["title"].as_str().unwrap().to_owned())
                                    .collect();
                                let selected_mod: String =
                                    Select::new("➡️ Please select a mod", options)
                                        .prompt()
                                        .unwrap();
                                filtered_mods
                                    .iter()
                                    .find(|m| {
                                        if m["title"].as_str().unwrap().to_owned() == selected_mod {
                                            true
                                        } else {
                                            false
                                        }
                                    })
                                    .iter()
                                    .for_each(|m| {
                                        mod_id_or_name =
                                            m["project_id"].as_str().unwrap().to_string();
                                        self.mod_id = Some(mod_id_or_name.clone().into());
                                        *mod_id = Some(mod_id_or_name.clone())
                                    });
                                continue;
                            }
                        }
                    }
                    // Mod ID Selected
                    if let Some(files) = self.extract_files(&data, version.clone()).await {
                        for file in files {
                            let download_url = file.get("url").and_then(|u| u.as_str());
                            if let Some(download_url) = download_url {
                                println!("Downloading from: {}", download_url);

                                match reqwest::get(download_url).await {
                                    Ok(res) => {
                                        let content = match res.bytes().await {
                                            Ok(bytes) => bytes,
                                            Err(e) => {
                                                println!("Failed to read file content: {}", e);
                                                continue;
                                            }
                                        };

                                        let file_name = file
                                            .get("filename")
                                            .and_then(|f| f.as_str())
                                            .unwrap_or("mod_file.zip");

                                        // Determine the full path
                                        let full_path = download_path
                                            .clone()
                                            .unwrap_or_else(|| std::env::current_dir().unwrap())
                                            .join(file_name);

                                        // Save the file
                                        match std::fs::write(&full_path, content) {
                                            Ok(_) => {
                                                println!(
                                                    "Downloaded: {}",
                                                    full_path.to_string_lossy()
                                                );
                                                if dependencies.is_some() {
                                                    let do_download_dependencies =
                                                        dependencies.unwrap();
                                                    if do_download_dependencies {
                                                        self.verify_server_dependencies(
                                                            download_path.clone(),
                                                        )
                                                        .await;
                                                    }
                                                }
                                            }
                                            Err(e) => println!(
                                                "Failed to save file to {}: {}",
                                                full_path.to_string_lossy(),
                                                e
                                            ),
                                        }
                                    }
                                    Err(e) => println!("Failed to download file: {}", e),
                                }
                            }
                        }
                    } else {
                        println!("No suitable mod files found for the given criteria.");
                    }
                    return;
                }
                Err(e) => {
                    println!("Failed to fetch mod information: {}", e);
                    return;
                }
            }
        }
    }

    /// Check if the version (if provided) matches the data and returns the json for file[] which has the download url
    async fn extract_files(&mut self, data: &Value, version: Option<String>) -> Option<Vec<Value>> {
        if let Some(projects) = data.as_array() {
            for project in projects {
                let loaders = project["loaders"].as_array().unwrap();
                if self.mod_loader.is_some() {
                    if !loaders
                        .iter()
                        .any(|l| l.as_str() == Some(self.mod_loader.clone().unwrap().as_str()))
                    {
                        continue;
                    }
                }
                if let Some(version_str) = version.clone() {
                    let game_versions = project["game_versions"].as_array().unwrap();
                    if game_versions
                        .iter()
                        .any(|v| v.as_str().unwrap() != &version_str)
                    {
                        continue;
                    }
                }
                if let Some(files) = project["dependencies"].as_array() {
                    self.dependencies = Some(json!(files));
                }
                if let Some(files) = project["files"].as_array() {
                    return Some(files.clone());
                }
            }
        }
        None
    }
}

impl ModrinthEntry {
    async fn verify_dependencies(&mut self, download_path: Option<PathBuf>) {
        // If the request fails retry it once.
        let mut project_ids: Vec<String> = vec![];
        if self.dependencies.is_some() {
            let dep = self.dependencies.clone().unwrap();
            if let Some(dependencies) = dep.as_array() {
                for dependency in dependencies {
                    if let Some(required) = dependency["dependency_type"].as_str() {
                        if required == "required" {
                            if let Some(dep_proj_id) = dependency["project_id"].as_str() {
                                project_ids.push(dep_proj_id.to_owned());
                            }
                        }
                    }
                }
            }
        }
        if project_ids.is_empty() {
            return;
        }
        for project_id in project_ids {
            let mod_loader_copy = self.mod_loader.clone();
            let mod_loader_target = if let Some(mod_loader) = mod_loader_copy {
                Some(format!("{}", mod_loader))
            } else {
                None
            };

            let target_id: &mut Option<String> = &mut Some(project_id.clone());
            // Box the future to allow recursion
            let boxed_future = Box::pin(self.download_mod(
                target_id,
                None,
                mod_loader_target,
                None,
                download_path.clone(),
                Some(true),
            ));
            boxed_future.await;
        }
    }
    async fn verify_server_dependencies(&mut self, download_path: Option<PathBuf>) {
        // If the request fails retry it once.
        let mut project_ids: Vec<String> = vec![];
        if self.dependencies.is_some() {
            let dep = self.dependencies.clone().unwrap();
            if let Some(dependencies) = dep.as_array() {
                for dependency in dependencies {
                    if let Some(required) = dependency["dependency_type"].as_str() {
                        if required == "required" {
                            if let Some(dep_proj_id) = dependency["project_id"].as_str() {
                                project_ids.push(dep_proj_id.to_owned());
                            }
                        }
                    }
                }
            }
        }
        if project_ids.is_empty() {
            return;
        }
        for project_id in project_ids {
            let mod_loader_copy = self.mod_loader.clone();
            let mod_loader_target = if let Some(mod_loader) = mod_loader_copy {
                Some(format!("{}", mod_loader))
            } else {
                None
            };

            let target_id: &mut Option<String> = &mut Some(project_id.clone());
            // Box the future to allow recursion
            let boxed_future = Box::pin(self.download_server_mod(
                target_id,
                None,
                mod_loader_target,
                match self.mod_version.clone() {
                    Some(v) => Some(format!("{}", v)),
                    None => None,
                },
                download_path.clone(),
                Some(true),
            ));
            boxed_future.await;
        }
    }
}
