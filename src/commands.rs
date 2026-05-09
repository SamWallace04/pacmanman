use std::process::Command;

use crate::shared::ListItems;
#[derive(Clone)]
pub struct PackageVersionInfo {
    pub name: String,
    details: Option<PackageDetails>,
    pub package_type: PackageType,
}

impl PartialEq for PackageVersionInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl ListItems for PackageVersionInfo {}

impl PackageVersionInfo {
    pub fn get_details(&mut self) -> PackageDetails {
        match &self.details {
            Some(d) => d.clone(),
            None => {
                let details = get_package_details("pacman", &self.name);
                self.details = Some(details.clone());
                details
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct PackageDetails {
    pub name: String,
    pub version: String,
    pub description: String,
    pub url: String,
    pub depends_on: Vec<String>,
    pub optional_dependencies: Vec<String>,
    pub required_by: Vec<String>,
    pub optional_for: Vec<String>,
    pub installed_size: String,
    pub installed_reason: String,
}

#[derive(Clone, Default)]
pub struct CloudPackage {
    pub name: String,
    pub version: String,
    pub description: String,
    pub source: String,
    pub is_installed: bool,
}

impl ListItems for CloudPackage {}

#[derive(Clone, PartialEq)]
pub enum PackageType {
    Explicit,
    Orphan,
    Foreign,
}

// TODO: Is there a better way than running the commands manually??
// i.e. reading the info from a file.
pub fn get_all_packages(package_manager: &str) -> Vec<PackageVersionInfo> {
    let mut list = get_explicit_packages(package_manager);
    let orphans = get_orphan_packages(package_manager);
    let foreign = get_foreign_packages(package_manager);

    let mut dedupe: Vec<PackageVersionInfo> = list
        .iter_mut()
        .map(|p| {
            // Getting explicit packages also gets foreign ones.
            // Update the explicit type to foreign if it's in the foreign list.
            if foreign.contains(p) {
                p.package_type = PackageType::Foreign
            }
            p.clone()
        })
        .collect();

    dedupe.extend(orphans);
    dedupe.sort_by_key(|i| i.name.clone());
    dedupe
}

pub fn get_explicit_packages(package_manager: &str) -> Vec<PackageVersionInfo> {
    let out = run_command(package_manager, vec!["-Qe"]);

    parse_version_list(&out, PackageType::Explicit)
}

pub fn get_orphan_packages(package_manager: &str) -> Vec<PackageVersionInfo> {
    let out = run_command(package_manager, vec!["-Qdt"]);

    parse_version_list(&out, PackageType::Orphan)
}

pub fn get_foreign_packages(package_manager: &str) -> Vec<PackageVersionInfo> {
    let out = run_command(package_manager, vec!["-Qm"]);

    parse_version_list(&out, PackageType::Foreign)
}

fn get_package_details(package_manager: &str, package_name: &str) -> PackageDetails {
    let out = run_command(package_manager, vec!["-Qi", package_name]);

    parse_details_list(&out)
}

pub fn search_packages(package_manager: &str, search_string: &str) -> Vec<CloudPackage> {
    let out = run_command(package_manager, vec!["-Ss", search_string]);

    parse_cloud_package_list(&out)
}

fn parse_version_list(input: &str, package_type: PackageType) -> Vec<PackageVersionInfo> {
    let list = input.split("\n");

    let mut version_list = vec![];
    for l in list {
        let split: Vec<&str> = l.split(" ").collect();
        if split.len() < 2 {
            continue;
        }

        version_list.push(PackageVersionInfo {
            name: split[0].to_string(),
            details: None,
            package_type: package_type.clone(),
        });
    }
    version_list
}

fn parse_details_list(input: &str) -> PackageDetails {
    let lines: Vec<&str> = input.split("\n").collect();
    let mut details: PackageDetails = PackageDetails::default();
    for line in lines {
        let split_line: Vec<&str> = line.split(":").collect();
        let key = split_line[0].to_lowercase().replace(" ", "");

        match key.as_ref() {
            "name" => details.name = split_line[1].to_owned(),
            "version" => details.version = split_line[1].to_owned(),
            "description" => details.description = split_line[1].to_owned(),
            "url" => details.url = split_line[1].to_owned(),
            "dependson" => details.depends_on = vec![split_line[1].split("\n").collect()],
            "optionaldeps" => {
                details.optional_dependencies = vec![split_line[1].split("\n").collect()]
            }
            "requiredby" => details.required_by = vec![split_line[1].split(" ").collect()],
            "optionalfor" => details.optional_for = vec![split_line[1].split(" ").collect()],
            "installedsize" => details.installed_size = split_line[1].to_owned(),
            "installreason" => details.installed_reason = split_line[1].to_owned(),
            _ => {}
        }
    }

    details
}

fn parse_cloud_package_list(input: &str) -> Vec<CloudPackage> {
    let lines: Vec<&str> = input.split("\n").collect();
    let mut packages: Vec<CloudPackage> = vec![];

    let mut line_num = 0;
    while line_num < lines.len() - 1 {
        let split1: Vec<&str> = lines[line_num].split("/").collect();
        let split2: Vec<&str> = split1[1].split(" ").collect();

        let package: CloudPackage = CloudPackage {
            name: split2[0].to_owned(),
            description: lines[line_num + 1].to_owned(),
            version: split2[1].to_owned(),
            source: split1[0].to_owned(),
            is_installed: split2.len() > 2,
        };
        packages.extend(vec![package]);

        line_num += 2;
    }

    // Remove this to sort by repo -> name
    packages.sort_by(|a, b| a.name.cmp(&b.name));
    packages
}

fn run_command(package_manager: &str, args: Vec<&str>) -> String {
    let output = Command::new(package_manager)
        .args(args)
        .output()
        .expect("Failed to execute command.");

    String::from_utf8(output.stdout).unwrap()
}
