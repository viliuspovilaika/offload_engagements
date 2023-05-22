#[path = "zip.rs"] mod zip;

use std::fs;
use which::which;
use std::process::Command;
use std::path::PathBuf;
use crate::nikto::zip::wrapper_zip_dir;

fn find_nikto() -> PathBuf {
    which("nikto").unwrap()
}

pub(crate) fn nikto_version() -> String {
    let binary_location = find_nikto();
    let mut command = Command::new(binary_location);
    command.arg("-Version");
    let res = command.output().unwrap();
    std::str::from_utf8(&res.stdout).unwrap().to_string()
}

pub(crate) async fn nikto(target: String, report_to: String) {
    println!("{}", report_to.clone());
    let binary_location = find_nikto();
    fs::create_dir("nikto_out").unwrap();
    // Nikto scan!
    let mut command = Command::new(binary_location);
    command.arg("-h");
    command.arg(target);
    command.arg("-output");
    command.arg("nikto_out/results.xml");
    let res = command.output().unwrap();
    // Make a .zip for scan results
    wrapper_zip_dir("nikto_out", "results.zip");
    // Submit the scan results
    let client = reqwest::Client::new();
    client.post(report_to).body(fs::read("results.zip").unwrap()).send().await.unwrap();
    // Delete the local scan result copy
    fs::remove_dir_all("nikto_out").unwrap();
    fs::remove_file("results.zip").unwrap();
    ()
}