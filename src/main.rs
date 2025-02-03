mod fabric_request;
mod modrinth_request;
mod papermc_request;
use clap::{Arg, Command};
use fabric_request::FabricMCRequest;
use modrinth_request::{ModrinthEntry, ModrinthSortingFilter};
use papermc_request::PaperMCRequest;
use reqwest::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let commands = Command::new("MCT [MINECRAFT TOOL]")
        .version("1.0")
        .author("Owlyat")
        .about("Minecraft Tool")
        .subcommand(
            Command::new("Search")
                .short_flag('s').visible_short_flag_alias('s')
                .alias("s")
                .about("Search on modrinth")
                .arg(
                    Arg::new("Name")
                        .long("name")
                        .short('n').visible_short_alias('n')
                        .aliases(["name", "n"])
                        .visible_aliases(["name", "n"])
                        .help("Search term")
                        .required(true),
                )
                .arg(
                    Arg::new("Project_Version")
                        .short('v')
                        .long("project_version")
                        .alias("pv")
                        .visible_alias("pv")
                        .help("Filter results by a specific game version")
                        .required(false),
                )
                .arg(
                    Arg::new("With_Loader")
                        .long("with_loader")
                        .alias("wl")
                        .visible_alias("wl")
                        .help("Filter results by loader\nex : fabric | neoforge | quilt | forge | ...")
                        .required(false),
                )
                .arg(Arg::new("Project_Type").long("project_type").short('t').alias("pt").help("The type of project you seek for, default value : mod\nex: mod | modpack | resourcepack | ...").required(false))
                .arg(
                    Arg::new("Result_Number")
                        .long("result_number")
                        .short('l').visible_short_alias('l')
                        .aliases(["mn", "max", "limit"])
                        .visible_aliases(["mn", "max"])
                        .value_parser(clap::value_parser!(usize))
                        .help("Specifies the max number of mods to be displayed")
                        .required(false),
                )
                .arg(
                    Arg::new("Offset")
                        .long("offset")
                        .short('o').visible_short_alias('o')
                        .alias("off")
                        .visible_alias("off")
                        .value_parser(clap::value_parser!(usize))
                        .help("Number of mods that will be skipped in the search")
                        .required(false),
                )
                .arg(
                    Arg::new("Sorting")
                        .long("sorting")
                        .short('f').visible_short_alias('f')
                        .aliases(["filter", "sort"])
                        .visible_aliases(["filter", "sort"])
                        .help("Sort results by relevance|downloads|follows|newest|updated")
                        .required(false),
                )
                .arg(
                    Arg::new("Client_Side")
                        .short('C').visible_short_alias('C')
                        .long("client_side")
                        .alias("client")
                        .visible_alias("client")
                        .value_parser(clap::value_parser!(bool))
                        .help("Filters Client side mods required")
                        .required(false),
                )
                .arg(
                    Arg::new("Server_Side")
                        .long("server_side")
                        .short('S').visible_short_alias('S')
                        .alias("server")
                        .visible_alias("server")
                        .value_parser(clap::value_parser!(bool))
                        .help("Filters Server side mods required")
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("Download_Entry")
                .long_flag("download_entry")
                .short_flag('d').visible_short_flag_alias('d')
                .alias("dw")
                .about("Download an entry on modrinth")
                .arg(
                    Arg::new("Id")
                        .long("entry_id")
                        .short('i').visible_short_alias('i')
                        .aliases(["id","Id","iD","ID"])
                        .help("Entry ID to download"),
                )
                .arg(
                    Arg::new("Name")
                        .long("name")
                        .short('n').visible_short_alias('n')
                        .help("Entry Name to download")
                        .required_unless_present("Id"),
                )
                .arg(
                    Arg::new("Version")
                        .long("version")
                        .short('v').visible_short_alias('v')
                        .alias("ev")
                        .help("Entry Version to download")
                        .required(false),
                )
                .arg(
                    Arg::new("Download_Path")
                        .long("download_path")
                        .short('p').visible_short_alias('p')
                        .help("Download entry to path given (not required)")
                        .required(false),
                )
                .arg(
                    Arg::new("With_Dependencies")
                        .long("with_dependencies")
                        .short('d').visible_short_alias('d')
                        .aliases(["with_dep", "wd"])
                        .visible_aliases(["with_dep", "wd"])
                        .value_parser(clap::value_parser!(bool))
                        .help("Downloads dependencies for the entry if found")
                        .required(false),
                )
                .arg(
                    Arg::new("For_Loader")
                        .long("for_loader")
                        .short('l')
                        .alias("fl")
                        .visible_alias("fl")
                        .help("Download entry for specified loader"),
                ).arg(Arg::new("For_Server")
                    .long("for_server")
                    .short('f').visible_short_alias('f')
                    .help("Download mod for fabric server, provide server root directory to this ar&gument")
                    .required(false)),
        ).subcommand(Command::new("Create_Server")
            .short_flag('c')
            .about("Create a directory with a Minecraft server")
            .arg(
                Arg::new("Path")
                    .long("Path")
                    .short('p')
                    .help("Server path Directory")
                    .required(false))
            .arg(
                Arg::new("Platform")
                    .long("Platform")
                    .short('c')
                    .help("ex : Paper")
                    .required(false))
            .arg(
                Arg::new("Gui")
                    .long("Gui")
                    .short('g')
                    .value_parser(clap::value_parser!(bool))
                    .help("Shows the server graphic user interface ex : true | false")
                    .required(false))
            .arg(
                Arg::new("Max_Ram")
                    .long("max_ram")
                    .alias("Xmx")
                    .visible_alias("Xmx")
                    .help("Max Amount of ram ex: 1024k | 512m | 8g")
                    .required(false))
            .arg(
                Arg::new("Min_Ram")
                    .long("min_ram")
                    .alias("Xms")
                    .visible_alias("Xms")
                    .help("Initial amount of ram ex: 1024k | 512m | 8g")
                    .required(false))
            .arg(
                Arg::new("Game_Version")
                    .long("game_version")
                    .short('v')
                    .help("Minecraft version ex: 1.20.1")
                    .required(false))
            .arg(
                Arg::new("Build")
                    .long("build")
                    .short('B')
                    .help("Build number ex: 23")
                    .required(false))
            .arg(
                Arg::new("Public_IP")
                    .long("public_ip")
                    .short('I')
                    .aliases(["pip","broadcastip","p_ip","bcast"])
                    .visible_aliases(["pip","broadcastip","p_ip","bcast"])
                    .value_parser(clap::value_parser!(bool))
                    .help("Broadcast your server with a network tunnel using a service depending on the online services (Serveo, Bore.pub, Tunnelto, Ownserver,...) ex : True | False")
                    .required(false)))
        .get_matches();

    match commands.subcommand() {
        Some(("Search", sub_commands)) => {
            let name = sub_commands.get_one::<String>("Name").unwrap();
            let version = sub_commands.get_one::<String>("Project_Version");
            let loader = sub_commands.get_one::<String>("With_Loader");
            let max_mods_number = sub_commands.get_one::<usize>("Result_Number");
            let offset = sub_commands.get_one::<usize>("Offset");
            let sorting = sub_commands.get_one::<String>("Sorting");
            let is_cliend_side = sub_commands.get_one::<bool>("Client_Side");
            let is_server_side = sub_commands.get_one::<bool>("Server_Side");
            let project_type = sub_commands.get_one::<String>("Project_Type");

            let mut modrinth_mod = ModrinthEntry::builder();
            modrinth_mod
                .search_modrinth(
                    name,
                    version,
                    loader,
                    max_mods_number.cloned(),
                    project_type.cloned(),
                    ModrinthSortingFilter::with(sorting),
                    offset.cloned(),
                    is_cliend_side.cloned(),
                    is_server_side.cloned(),
                )
                .await;

            modrinth_mod.display_entries();
        }
        Some(("Download_Entry", sub_commands)) => {
            let for_server = sub_commands.get_one::<String>("For_Server");
            let id = sub_commands.get_one::<String>("Id");
            let version = sub_commands.get_one::<String>("Version");
            let name = sub_commands.get_one::<String>("Name");
            let download_path = sub_commands.get_one::<String>("Download_Path");
            let do_download_dependencies = sub_commands.get_one::<bool>("With_Dependencies");
            let for_loader = sub_commands.get_one::<String>("For_Loader");

            if for_server.is_some() {
                let server_path = verify_path(for_server.cloned());
                let mut fabric_server = FabricMCRequest::build(server_path);
                match fabric_server.check_data(None) {
                    Ok(_) => {
                        let mut modrinth_entry = ModrinthEntry::builder();
                        modrinth_entry
                            .download_server_mod(
                                &mut id.cloned(),
                                name.cloned(),
                                Some("fabric".to_owned()),
                                fabric_server.get_version(),
                                fabric_server.get_download_path(),
                                Some(true),
                            )
                            .await;
                    }
                    Err(_) => {}
                }
            } else {
                let mut modrinth_entry = ModrinthEntry::builder();
                modrinth_entry
                    .download_mod(
                        &mut id.cloned(),
                        name.cloned(),
                        for_loader.cloned(),
                        version.cloned(),
                        verify_path(download_path.cloned()),
                        do_download_dependencies.cloned(),
                    )
                    .await;
            }
        }
        Some(("Create_Server", sub_commands)) => {
            let path = sub_commands.get_one::<String>("Path");
            let game_version = sub_commands.get_one::<String>("Game_Version");
            let build = sub_commands.get_one::<String>("Build");
            let platform = sub_commands.get_one::<String>("Platform");
            let xmx = sub_commands.get_one::<String>("Max_Ram");
            let xms = sub_commands.get_one::<String>("Min_Ram");
            let is_gui = sub_commands.get_one::<bool>("Gui");
            let open_with_public_ip = sub_commands.get_one::<bool>("Public_IP");

            let path = match check_server_path(path.cloned()) {
                Ok(p) => p,
                Err(e) => {
                    panic!("❌ Error while checking the server path\n    ➡️ {}", e)
                }
            };
            match platform {
                Some(p) if p.to_lowercase() == "paper" => {
                    let mut paper_server = PaperMCRequest::build();
                    match paper_server.check_data(path.clone()) {
                        Ok(_) => {
                            match open_with_public_ip {
                                Some(do_open) => {
                                    if *do_open {
                                        std::process::Command::new("cmd")
                                            .args([
                                                "/C",
                                                "ssh",
                                                "-R",
                                                "0:localhost:25565",
                                                "serveo.net",
                                            ])
                                            .spawn()
                                            .unwrap();
                                    }
                                }
                                None => {}
                            }
                            paper_server.start_server(xmx.cloned(), xms.cloned(), is_gui.cloned());
                        }
                        Err(_) => {
                            paper_server
                                .check_build(game_version.cloned(), build.cloned())
                                .await;
                            paper_server.download_build(path).await;

                            match open_with_public_ip {
                                Some(do_open) => {
                                    if *do_open {
                                        std::process::Command::new("cmd")
                                            .args([
                                                "/C",
                                                "ssh",
                                                "-R",
                                                "0:localhost:25565",
                                                "serveo.net",
                                            ])
                                            .spawn()
                                            .unwrap();
                                    }
                                }
                                None => {}
                            }
                            paper_server.start_server(xmx.cloned(), xms.cloned(), is_gui.cloned());
                        }
                    }
                }
                Some(p) if p.to_lowercase() == "fabric" => {
                    let mut fabric_server = FabricMCRequest::build(Some(path.clone()));
                    match fabric_server.check_data(Some(path.clone())) {
                        Ok(_) => {
                            match open_with_public_ip {
                                Some(do_open_public) => {
                                    if *do_open_public {
                                        std::process::Command::new("cmd")
                                            .args([
                                                "/C",
                                                "ssh",
                                                "-R",
                                                "0:localhost:25565",
                                                "serveo.net",
                                            ])
                                            .spawn()
                                            .unwrap();
                                    }
                                }
                                None => (),
                            }
                            fabric_server.start_server(xmx.cloned(), xms.cloned(), is_gui.cloned());
                        }
                        Err(_) => {
                            fabric_server
                                .select_game_version(game_version.cloned())
                                .await;
                            fabric_server.select_loader_version().await;
                            fabric_server.fetch_latest_installer_version().await;
                            fabric_server.generate_download_url();
                            fabric_server.download_build(path).await;
                            match open_with_public_ip {
                                Some(do_open_public) => {
                                    if *do_open_public {
                                        std::process::Command::new("cmd")
                                            .args([
                                                "/C",
                                                "ssh",
                                                "-R",
                                                "0:localhost:25565",
                                                "serveo.net",
                                            ])
                                            .spawn()
                                            .unwrap();
                                    }
                                }
                                None => (),
                            }
                            fabric_server.start_server(xmx.cloned(), xms.cloned(), is_gui.cloned());
                        }
                    }
                }
                _ => {
                    let mut paper_server = PaperMCRequest::build();
                    match paper_server.check_data(path.clone()) {
                        Ok(_) => {
                            println!("✅ MCA.json Found !");
                            match open_with_public_ip {
                                Some(do_open) => {
                                    if *do_open {
                                        std::process::Command::new("cmd")
                                            .args([
                                                "/C",
                                                "ssh",
                                                "-R",
                                                "0:localhost:25565",
                                                "serveo.net",
                                            ])
                                            .spawn()
                                            .unwrap();
                                    }
                                }
                                None => {}
                            }
                            paper_server.start_server(xmx.cloned(), xms.cloned(), is_gui.cloned());
                        }
                        Err(_) => {
                            println!("➡️ No MCA.json Found");
                            paper_server
                                .check_build(game_version.cloned(), build.cloned())
                                .await;
                            paper_server.download_build(path).await;

                            match open_with_public_ip {
                                Some(do_open) => {
                                    if *do_open {
                                        std::process::Command::new("cmd")
                                            .args([
                                                "/C",
                                                "ssh",
                                                "-R",
                                                "0:localhost:25565",
                                                "serveo.net",
                                            ])
                                            .spawn()
                                            .unwrap();
                                    }
                                }
                                None => {}
                            }
                            paper_server.start_server(xmx.cloned(), xms.cloned(), is_gui.cloned());
                        }
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}

use std::{
    fs,
    path::{Path, PathBuf},
};

/// Verifies if the given download path is valid.
/// Returns `Some(&Path)` if the path exists and is a directory, otherwise `None`.
fn verify_path(path: Option<String>) -> Option<PathBuf> {
    if path.is_some() {
        let dlpath = path.unwrap();
        let path = Path::new(&dlpath);

        // Check if the path exists and is a directory
        if path.exists() && path.is_dir() {
            println!("✅ Download path : '{}'", path.to_string_lossy());
            Some(path.to_path_buf())
        } else {
            println!(
                "❌ Invalid path: '{}'. Path does not exist or is not a directory.",
                dlpath
            );
            None
        }
    } else {
        println!("➡️ No download path provided");
        None
    }
}

// Check if the provided path is valid, else try to create a default path else returns an error
fn check_server_path(path: Option<String>) -> Result<PathBuf, std::io::Error> {
    if path.clone().is_some() {
        if Path::new(&path.clone().unwrap()).exists() {
            // Path already exists
            println!("✅ Server directory Found");
            Ok(Path::new(&path.unwrap()).to_path_buf())
        } else {
            // Created Dir
            match fs::create_dir(Path::new(&path.clone().unwrap())) {
                Ok(_) => {
                    println!("✅ Server directory created Successfully");
                    Ok(Path::new(&path.unwrap()).to_path_buf())
                }
                Err(e) => Err(e),
            }
        }
    } else {
        // No path provided try default path if it does not work return error
        let default_path = "./MCT Server";

        if Path::new(default_path).exists() {
            println!("✅ Default Server directory Found");
            Ok(Path::new(default_path).to_path_buf())
        } else {
            match fs::create_dir(default_path) {
                Ok(_) => {
                    println!(
                        "✅ Default Server directory created Successfully at {}",
                        Path::new(default_path).to_string_lossy()
                    );
                    Ok(Path::new(default_path).to_path_buf())
                }
                Err(e) => Err(e),
            }
        }
    }
}
