use std::{
    fs,
    io::{BufReader, Read},
    path::Path,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PackageInToml {
    name: String,
    license_file_url: Option<String>,
    license: Option<String>,
}

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub license_file_content: Option<String>,
    pub license: Option<String>,
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
        .map(
            |p| match (p.license_file_url.as_ref(), p.license.as_ref()) {
                (Some(url), None) => reqwest::blocking::get(url)
                    .and_then(|body| body.text())
                    .map(|text| Package {
                        name: p.name.to_owned(),
                        license_file_content: Some(text),
                        license: None,
                    }),
                (None, Some(license)) => Ok(Package {
                    name: p.name.to_owned(),
                    license_file_content: None,
                    license: Some(license.to_owned()),
                }),
                _ => panic!("Either license_file_url or license must be specified"),
            },
        )
        .collect::<Result<Vec<_>, _>>()?)
}
