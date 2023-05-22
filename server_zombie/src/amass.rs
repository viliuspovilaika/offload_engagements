#[path = "zip.rs"] mod zip;

use std::fs;
use which::which;
use std::process::Command;
use std::path::PathBuf;
use crate::amass::zip::wrapper_zip_dir;

fn find_amass() -> PathBuf {
    which("amass").unwrap()
}

pub(crate) fn amass_version() -> String {
    let binary_location = find_amass();
    let mut command = Command::new(binary_location);
    command.arg("-version");
    let res = command.output().unwrap();
    std::str::from_utf8(&res.stdout).unwrap().to_string()
}

pub(crate) async fn amass(target: String, report_to: String) {
    println!("{}", report_to.clone());
    let binary_location = find_amass();
    fs::create_dir("amass_out").unwrap();
    // Amass scan!
    let mut command = Command::new(binary_location);
    command.arg("enum");
    command.arg("-active");
    command.arg("-ipv4");
    command.arg("-json");
    command.arg("amass_out/amass_json_out.json");
    command.arg("-log");
    command.arg("amass_out/errors.log");
    command.arg("-o");
    command.arg("amass_out/amass_txt_out.txt");
    command.arg("-src");
    command.arg("-timeout");
    command.arg("60");  // timeout after 1 hour
    command.arg("-p");
    command.arg("80,8080,8443,443");
    command.arg("-config");
    command.arg("amass.ini");
    command.arg("-d");
    command.arg(target);
    let res = command.output().unwrap();
    // Make a .zip for scan results
    wrapper_zip_dir("amass_out", "results.zip");
    // Submit the scan results
    let client = reqwest::Client::new();
    client.post(report_to).body(fs::read("results.zip").unwrap()).send().await.unwrap();
    // Delete the local scan result copy
    fs::remove_dir_all("amass_out").unwrap();
    fs::remove_file("results.zip").unwrap();
    ()
}