use std::{
    fs,
    io::{BufReader, Read},
    path::Path,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PackageInToml {
    name: String,
    license_file_url: String,
}

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub license_file_content: String,
}

#[derive(Debug, Deserialize)]
struct LicenseFileMap {
    package: Vec<PackageInToml>,
}

pub fn load_license_file_map<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Package>> {
    let mut file_content = String::new();
    fs::File::open(path)
        .map(BufReader::new)?
        .read_to_string(&mut file_content)?;
    Ok(toml::from_str::<LicenseFileMap>(&file_content)?
        .package
        .iter()
        .map(|p| {
            reqwest::blocking::get(&p.license_file_url)
                .and_then(|body| body.text())
                .map(|text| Package {
                    name: p.name.to_owned(),
                    license_file_content: text,
                })
        })
        .collect::<Result<Vec<_>, _>>()?)
}
