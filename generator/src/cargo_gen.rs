/// Generates a `Cargo.toml` file for the generated driver
use serde::Serialize;

#[derive(Serialize)]
struct CargoToml {
    package: Package,
    dependencies: Dependencies,
    #[serde(rename = "dev-dependencies")]
    dev_dependencies: DevDependencies,
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
}

#[derive(Serialize)]
#[serde(rename = "dev-dependencies")]
struct DevDependencies {
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
        version: semver::Version::parse("0.11.1").unwrap(),
        features: Some(vec!["eh1".to_string()]),
    };
    let dependencies = Dependencies {
        embedded_hal: embedded_hal_dependency,
    };

    let dev_dependencies = DevDependencies {
        embedded_hal_mock: embedded_hal_mock_dependency,
    };

    let cargo_toml = CargoToml {
        package,
        dependencies,
        dev_dependencies,
    };

    toml::to_string(&cargo_toml).expect("Unable to generate Cargo.toml")
}
