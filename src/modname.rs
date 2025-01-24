use std::str::FromStr;

use super::download_mod;
use crate::stdin;

use serde_json::Value;

use super::get_items;

use super::MCMod;

// pub(crate) async fn prompt_mod() -> Result<(), reqwest::Error> {
//     let _mod_name = loop {
//         println!("Enter mod name : ");
//         let mut input = String::new();

//         prompt::<String>("Enter a mod name : ").unwrap();

//         match stdin().read_line(&mut input) {
//             Ok(_v) => {
//                 let modrinth_mod = MCMod::builder();
//                 modrinth_mod.search_modrinth_mod(input.trim(), , , , , )
//                 let found = search_modrinth_mod(input.trim(), None, None).await?;
//                 let (titles, project_ids, descriptions, versions) = (
//                     get_items(&found, "title"),
//                     get_items(&found, "project_id"),
//                     get_items(&found, "description"),
//                     get_items(&found, "versions"),
//                 );

//                 let (titles, project_ids, descriptions, versions) = (
//                     titles.unwrap(),
//                     project_ids.unwrap(),
//                     descriptions.unwrap(),
//                     versions.unwrap(),
//                 );
//                 titles
//                     .iter()
//                     .zip(project_ids.iter())
//                     .zip(descriptions)
//                     .zip(versions)
//                     .enumerate()
//                     .for_each(|(index, (((title, _id), _description), _versionn))| {
//                         println!("{}:{}", index, title.as_str().unwrap());
//                     });
//                 println!("Enter mod number : ");
//                 let pid: Value;

//                 input.clear();
//                 match stdin().read_line(&mut input) {
//                     Ok(_) => match input.trim().parse::<usize>() {
//                         Ok(v) => {
//                             // V can be compared to index of a project
//                             if titles.len() < v {
//                                 println!("Mod Number out of bound !");
//                                 continue;
//                             } else {
//                                 // V is valid and we take the project id
//                                 println!("Selected : {}", titles[v].as_str().unwrap());
//                                 pid = project_ids[v].clone();
//                             }
//                         }
//                         Err(e) => {
//                             println!("{}", e);
//                             continue;
//                         }
//                     },
//                     Err(e) => {
//                         println!("{}", e);
//                         continue;
//                     }
//                 }

//                 let url = format!(
//                     "https://api.modrinth.com/v2/project/{}",
//                     pid.as_str().unwrap()
//                 );
//                 println!("{url}");
//                 let result = reqwest::get(url).await?;

//                 let mut available_game_versions: Vec<Value> = vec![];
//                 let mut available_loader: Vec<Value> = vec![];
//                 let mut json: Value = Value::Null;
//                 if result.status().is_success() {
//                     json = result.json::<Value>().await?;
//                     if let Some(gv) = json["game_versions"].as_array() {
//                         gv.iter().for_each(|game_version| {
//                             println!("{}", game_version.as_str().unwrap())
//                         });
//                         available_game_versions.append(&mut gv.clone());
//                     }
//                     if let Some(loader) = json["loaders"].as_array() {
//                         available_loader = loader.clone();
//                     }
//                 }
//                 let mut selected_version = String::new();
//                 println!("Select version : ");
//                 input.clear();
//                 match stdin().read_line(&mut input) {
//                     Ok(_) => {
//                         for gv in available_game_versions {
//                             if input.trim() == gv.as_str().unwrap() {
//                                 // versions corresponds, check loader !
//                                 selected_version = gv.clone().as_str().unwrap().into();
//                             }
//                         }
//                     }
//                     Err(_) => {
//                         continue;
//                     }
//                 }
//                 println!("{} version {}", pid, selected_version);
//                 let mut selected_loader: String = String::new();
//                 available_loader
//                     .iter()
//                     .enumerate()
//                     .for_each(|(index, loader)| {
//                         println!("{} : {}", index, loader.as_str().unwrap())
//                     });
//                 input.clear();
//                 match stdin().read_line(&mut input) {
//                     Ok(_) => {
//                         for loader in &available_loader {
//                             println!("{:#?}", loader);
//                             if loader.as_str().unwrap() == input.trim() {
//                                 // Loader correspond, proceed to download
//                                 selected_loader = loader.clone().as_str().unwrap().into();
//                             }
//                         }
//                     }
//                     Err(e) => {
//                         println!("{e}");
//                         continue;
//                     }
//                 }
//                 // WE CAN DOWNLOAD HERE
//                 //println!("{:#?}", json);
//                 download_mod(
//                     pid.as_str().unwrap(),
//                     Some(&selected_version),
//                     Some(&selected_loader),
//                 )
//                 .await?;
//                 return Ok(());
//             }
//             Err(_) => {
//                 continue;
//             }
//         }
//     };
// }

/// Just a function to prompt user with a specified prompt sentence and retrieve the parsed infered type
fn prompt<T: FromStr>(prompt: impl Into<String>) -> Result<T, Box<dyn std::error::Error>> {
    let prompt = prompt.into();
    loop {
        println!("{}", prompt);
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        match input.trim().parse::<T>() {
            Ok(value) => break Ok(value),
            Err(_e) => {
                continue;
            }
        }
    }
}
