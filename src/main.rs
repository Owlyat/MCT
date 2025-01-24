mod modrinth_request;
use modrinth_request::{MCMod, ModrinthSortingFilter};
use std::io::stdin;
mod modname;
use clap::{Arg, Command};
use reqwest::Error;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let commands = Command::new("Modrinth CLI")
        .version("1.0")
        .author("Owlyat")
        .about("Manages mods")
        .subcommand(
            Command::new("search_mod")
                .short_flag('s')
                .alias("sm")
                .about("Search for a mod")
                .arg(
                    Arg::new("mod_name")
                        .short('n')
                        .aliases(["name", "n"])
                        .visible_aliases(["name", "n"])
                        .long("modname")
                        .help("Search term for mod")
                        .required(true),
                )
                .arg(
                    Arg::new("mod_version")
                        .long("mod_version")
                        .alias("mv")
                        .visible_alias("mv")
                        .required(false)
                        .help("Filter results by a specific mod version"),
                )
                .arg(
                    Arg::new("with_loader")
                        .long("with_loader")
                        .alias("wl")
                        .visible_alias("wl")
                        .required(false)
                        .help("Filter results by loader"),
                )
                .arg(
                    Arg::new("mods_number")
                        .long("mods_number")
                        .aliases(["mn", "max"])
                        .visible_aliases(["mn", "max"])
                        .help("Specifies the max number of mods to be displayed")
                        .value_parser(clap::value_parser!(usize))
                        .required(false),
                )
                .arg(
                    Arg::new("offset")
                        .long("offset")
                        .alias("off")
                        .visible_alias("off")
                        .short('o')
                        .help("Number of mods that will be skipped in the search")
                        .required(false)
                        .value_parser(clap::value_parser!(usize)),
                )
                .arg(
                    Arg::new("Sorting")
                        .long("Sorting")
                        .short('f')
                        .aliases(["filter", "sort"])
                        .visible_aliases(["filter", "sort"])
                        .help("Sort results by relevance|downloads|follows|newest|updated")
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("download_mod")
                .short_flag('d')
                .alias("dm")
                .about("Download a mod by ID")
                .arg(
                    Arg::new("mod_id")
                        .aliases(["id", "modid", "mod_id", "ID", "Id", "ModId"])
                        .long("download_mod")
                        .help("Mod ID to download")
                        .required_unless_present("mod_name"),
                )
                .arg(
                    Arg::new("mod_name")
                        .alias("mn")
                        .short('n')
                        .long("mod_name")
                        .help("Mod Name to download")
                        .required_unless_present("mod_id"),
                )
                .arg(
                    Arg::new("mod_version")
                        .alias("mv")
                        .help("Mod Version to download")
                        .required(false),
                )
                .arg(
                    Arg::new("download_path")
                        .long("download_path")
                        .alias("p")
                        .visible_alias("p")
                        .required(false)
                        .help("Download mod to path given (not required)"),
                ),
        )
        .get_matches();

    match commands.subcommand() {
        Some(("search_mod", sub_commands)) => {
            let mod_name = sub_commands.get_one::<String>("mod_name").unwrap();
            let version = sub_commands.get_one::<String>("mod_version");
            let loader = sub_commands.get_one::<String>("with_loader");
            let max_mods_number = sub_commands.get_one::<usize>("mods_number");
            let offset = sub_commands.get_one::<usize>("offset");
            let sorting = sub_commands.get_one::<String>("Sorting");

            let mut modrinth_mod = MCMod::builder();
            modrinth_mod
                .search_modrinth_mod(
                    mod_name,
                    version,
                    loader,
                    max_mods_number.cloned(),
                    ModrinthSortingFilter::with(sorting),
                    offset.cloned(),
                )
                .await;

            modrinth_mod.display_mods();
        }
        Some(("download_mod", sub_commands)) => {
            let mod_id = sub_commands.get_one::<String>("mod_id");
            let version = sub_commands.get_one::<String>("mod_version");
            let mod_name = sub_commands.get_one::<String>("mod_name");
            let download_path = sub_commands.get_one::<String>("download_path");

            let mut modrinth_req = MCMod::builder();
            modrinth_req
                .download_mod(
                    &mut mod_id.cloned(),
                    mod_name.cloned(),
                    version.cloned(),
                    verify_download_path(download_path.cloned()),
                )
                .await;
        }
        _ => (),
    }

    Ok(())
}

fn get_items(json: &Value, value: &str) -> Option<Vec<Value>> {
    if let Some(hits) = json.get("hits") {
        if let Some(projects) = hits.as_array() {
            let mut result = vec![];
            projects
                .iter()
                .map(|project| {
                    result.push(match project.get(value) {
                        Some(v) => v.clone(),
                        None => {
                            panic!("No value {} in {:#?}", value, project)
                        }
                    })
                })
                .count();
            return Some(result);
        }
    }
    None
}

async fn download_mod(
    mod_id: &str,
    version: Option<&str>,
    loader: Option<&str>,
) -> Result<(), reqwest::Error> {
    if loader.is_none() {
        println!("no loader in the function call");
    }
    if version.is_none() {
        println!("no version in the function call")
    }
    let url = format!("https://api.modrinth.com/v2/project/{}/version", mod_id);
    println!("Fetching versions from: {}", url);

    let response = reqwest::get(&url).await?;
    let data = response.json::<Value>().await?;

    if let Some(json) = data.as_array() {
        let mut good_builds = vec![];
        // find the corresponding build with the right version and loader
        json.iter()
            .filter(|object| {
                if let Some(game_versions) = object["game_versions"].as_array() {
                    if let Some(_loaders) = object["loaders"].as_array() {
                        if version.is_some()
                            && game_versions
                                .contains(&serde_json::Value::String(version.unwrap().to_owned()))
                        {
                            if loader.is_some() {
                                true
                            } else {
                                if loader.is_none() {
                                    true
                                } else {
                                    false
                                }
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .map(|v| {
                good_builds.push(v);
            })
            .count();
        if good_builds.len() > 1 {
            good_builds.iter().for_each(|v| {
                if let Some(files) = v["files"].as_array() {
                    files.iter().for_each(|v| {
                        if let Some(url) = v["url"].as_str() {
                            println!("    => {}", url)
                        }
                    });
                }
            });
        }
    }
    Ok(())
}

use std::path::{Path, PathBuf};

/// Verifies if the given download path is valid.
/// Returns `Some(&Path)` if the path exists and is a directory, otherwise `None`.
fn verify_download_path(download_path: Option<String>) -> Option<PathBuf> {
    if download_path.is_some() {
        let dlpath = download_path.unwrap();
        let path = Path::new(&dlpath);

        // Check if the path exists and is a directory
        if path.exists() && path.is_dir() {
            Some(path.to_path_buf())
        } else {
            println!(
                "Invalid download path: '{}'. Path does not exist or is not a directory.",
                dlpath
            );
            None
        }
    } else {
        None
    }
}
