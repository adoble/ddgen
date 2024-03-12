use std::default;

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
    embedded_hal: Dependency,
    #[serde(rename = "embedded-hal-mock")]
    embedded_hal_mock: Dependency,
}

#[derive(Serialize)]
struct Dependency {
    version: semver::Version,

    features: Option<Vec<String>>,
}

pub fn generate(name: &str, version: &semver::Version) -> String {
    let package = Package {
        name: name.to_string().to_lowercase(),
        version: version.to_string(),
        edition: "2021".to_string(),
    };

    let embedded_hal_dependency = Dependency {
        version: semver::Version::parse("1.0.0").unwrap(),
        features: None,
    };

    let embedded_hal_mock_dependency = Dependency {
        version: semver::Version::parse("0.10.0").unwrap(),
        features: Some(vec!["eh1".to_string()]),
    };
    let dependencies = Dependencies {
        embedded_hal: embedded_hal_dependency,
        embedded_hal_mock: embedded_hal_mock_dependency,
    };

    let cargo_toml = CargoToml {
        package,
        dependencies,
    };

    toml::to_string(&cargo_toml).expect("Unable to generate Cargo.toml")
}
