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
pub struct MCMod {
    mod_name: Option<Box<str>>,
    mod_id: Option<Box<str>>,
    mod_version: Option<Box<str>>,
    mod_loader: Option<Box<str>>,
    response: Option<Value>,
}

impl MCMod {
    pub fn builder() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Use this if you want to search a modrinth mod
    pub async fn search_modrinth_mod(
        &mut self,
        mod_name: impl Into<String>,
        mod_version: Option<impl Into<String>>,
        mod_loader: Option<impl Into<String>>,
        max_mod_number: Option<usize>,
        sorting: Option<ModrinthSortingFilter>,
        offset: Option<usize>,
    ) -> &mut Self {
        // this is just to call the function with Some("word")
        let target_mod_name: String = mod_name.into();

        // Creating the request with the mod name or mod id and if defined mod version, mod loader
        let query = format!("?query={}", target_mod_name);

        let mut facets: Vec<Value> = Vec::new();

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

        // Building the URL to the API END POINT
        let url = format!(
            "{}{}{}{}{}{}",
            SEARCH_API_END_POINT,
            query,
            json!(facets),
            limit,
            sorting,
            offset,
        );

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

impl Default for MCMod {
    fn default() -> Self {
        Self {
            mod_name: None,
            mod_id: None,
            mod_version: None,
            mod_loader: None,
            response: None,
        }
    }
}

impl MCMod {
    pub fn display_mods(&self) {
        if let Some(response) = &self.response {
            if let Some(mods) = response.get("hits").and_then(|hits| hits.as_array()) {
                println!("Found {} mods:", mods.len());
                for (index, mod_entry) in mods.iter().enumerate() {
                    let mod_name = mod_entry
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown");
                    let mod_id = mod_entry
                        .get("project_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown");
                    let mod_version = mod_entry
                        .get("latest_version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown");
                    let mod_desc = mod_entry
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
                        "\n{}. Mod Name: {}\n   ID: {}\n   Latest Version: {}\n   Game Versions: {}\n   Description: {}",
                        index + 1, mod_name, mod_id, mod_version, game_versions, mod_desc
                    );
                }
            } else {
                println!("No mods found in the response!");
            }
        } else {
            println!("No response to display mods from!");
        }
    }
}

impl MCMod {
    pub async fn download_mod(
        &mut self,
        mod_id: &mut Option<String>,
        mod_name: Option<String>,
        version: Option<String>,
        download_path: Option<PathBuf>,
    ) {
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
                format!(
                    "https://api.modrinth.com/v2/search?query=\"{}\"&facets=[[\"project_type:mod\"]]",
                    mod_id_or_name
                )
            };

            println!("Fetching mod information from: {}", url);

            // Fetch the data
            let response = reqwest::get(&url).await;
            match response {
                Ok(res) => {
                    let data: Value = match res.json().await {
                        Ok(json) => json,
                        Err(e) => {
                            println!("Failed to parse JSON response: {}", e);
                            return;
                        }
                    };

                    if mod_id.is_none() {
                        // If searching by name and multiple results are found
                        if let Some(mods) = data["hits"].as_array() {
                            if mods.len() > 1 {
                                println!("Multiple mods found. Please select one:");
                                for (index, mod_entry) in mods.iter().enumerate() {
                                    let name = mod_entry["title"].as_str().unwrap_or("Unknown Mod");
                                    let author =
                                        mod_entry["author"].as_str().unwrap_or("Unknown Author");
                                    println!("{}: {} by {}", index + 1, name, author);
                                }
                                let mut input = String::new();
                                std::io::stdin()
                                    .read_line(&mut input)
                                    .expect("Failed to read input");
                                let choice: usize = match input.trim().parse() {
                                    Ok(num) if num > 0 && num <= mods.len() => num,
                                    _ => {
                                        println!("Invalid selection.");
                                        return;
                                    }
                                };
                                if let Some(selected_mod) = mods.get(choice - 1) {
                                    mod_id_or_name = selected_mod["project_id"]
                                        .as_str()
                                        .expect("Mod ID not found")
                                        .to_string();
                                    *mod_id = Some(mod_id_or_name.clone());
                                    continue; // Restart the loop with the selected mod ID
                                }
                            }
                        }
                    }

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
                                            Ok(_) => println!(
                                                "Downloaded: {}",
                                                full_path.to_string_lossy()
                                            ),
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

    async fn extract_files(&self, data: &Value, version: Option<String>) -> Option<Vec<Value>> {
        if let Some(versions) = data.as_array() {
            for version_data in versions {
                if let Some(version_str) = &version {
                    let game_versions = version_data["game_versions"].as_array()?;
                    if !game_versions
                        .iter()
                        .any(|v| v.as_str() == Some(version_str))
                    {
                        continue;
                    }
                }
                if let Some(files) = version_data["files"].as_array() {
                    return Some(files.clone());
                }
            }
        }
        None
    }
}
