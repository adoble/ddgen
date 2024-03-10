/// Generates a `Cargo.toml` file for the generated driver
use serde::Serialize;

#[derive(Serialize)]
struct CargoToml {
    package: Package,
    dependencies: Dependencies,
}

#[derive(Serialize)]
struct Package {
    name: String,
    version: String,
    edition: String,
}

#[derive(Serialize)]
struct Dependencies {
    #[serde(rename = "embedded-hal")]
    embedded_hal: String,
    #[serde(rename = "embedded-hal-mock")]
    embedded_hal_mock: String,
}

pub fn generate(name: &str, version: &semver::Version) -> String {
    let package = Package {
        name: name.to_string().to_lowercase(),
        version: version.to_string(),
        edition: "2021".to_string(),
    };
    let dependencies = Dependencies {
        embedded_hal: "0.2.7".to_string(),
        embedded_hal_mock: "0.7.2".to_string(),
    };

    let cargo_toml = CargoToml {
        package,
        dependencies,
    };

    toml::to_string(&cargo_toml).expect("Unable to generate Cargo.toml")
}
