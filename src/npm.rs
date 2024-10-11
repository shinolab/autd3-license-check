use std::{
    collections::HashMap,
    fs,
    io::{BufReader, Read},
    path::Path,
};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct PackageLockModuleJson {
    pub dev: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct PackageLockJson {
    pub packages: HashMap<String, PackageLockModuleJson>,
}

#[derive(Deserialize, Debug)]
struct PackageJson {
    pub name: String,
    pub version: String,
    pub repository: serde_json::Value,
    pub license: String,
}

#[derive(Debug)]
pub struct NodeDependencyDetails {
    pub name: String,
    pub version: String,
    pub repository: String,
    pub license: String,
}

pub fn glob_node_modules<P1, P2>(
    node_modules_path: P1,
    package_lock_json_path: P2,
) -> anyhow::Result<Vec<NodeDependencyDetails>>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let mut package_lock_json = String::new();
    fs::File::open(package_lock_json_path)
        .map(BufReader::new)?
        .read_to_string(&mut package_lock_json)?;
    let dev_packages =
        if let Ok(package_lock) = serde_json::from_str::<PackageLockJson>(&package_lock_json) {
            package_lock
                .packages
                .into_iter()
                .filter_map(|(name, module)| {
                    if module.dev.unwrap_or(false) {
                        Some(name.replace("node_modules/", ""))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

    let mut details = Vec::new();
    for entry in glob::glob(&format!(
        "{}/{}",
        node_modules_path.as_ref().to_str().unwrap(),
        "**/package.json"
    ))? {
        let entry = entry?;
        let mut file_content = String::new();
        fs::File::open(&entry)
            .map(BufReader::new)?
            .read_to_string(&mut file_content)?;
        if let Ok(package) = serde_json::from_str::<PackageJson>(&file_content) {
            if dev_packages.contains(&package.name) {
                continue;
            }
            details.push(NodeDependencyDetails {
                name: package.name,
                version: package.version,
                repository: match package.repository {
                    serde_json::Value::String(rep) => rep,
                    serde_json::Value::Object(map) => {
                        if let Some(rep) = map.get("url") {
                            rep.as_str().unwrap().to_owned()
                        } else {
                            return Err(anyhow::anyhow!("invalid repository type"));
                        }
                    }
                    _ => return Err(anyhow::anyhow!("invalid repository type")),
                },
                license: package.license,
            });
        }
    }

    Ok(details)
}
