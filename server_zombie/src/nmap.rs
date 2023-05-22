#[path = "zip.rs"] mod zip;

use std::fs;
use which::which;
use std::process::Command;
use std::path::PathBuf;
use crate::nmap::zip::wrapper_zip_dir;

fn find_nmap() -> PathBuf {
    which("nmap").unwrap()
}

pub(crate) fn nmap_version() -> String {
    let binary_location = find_nmap();
    let mut command = Command::new(binary_location);
    command.arg("--version");
    let res = command.output().unwrap();
    std::str::from_utf8(&res.stdout).unwrap().to_string()
}

// Do an nmap scan, zip up the results and send them to the desired callback
pub(crate) async fn nmap(target: String, report_to: String) {
    println!("{}", report_to.clone());
    let binary_location = find_nmap();
    fs::create_dir("nmap_out").unwrap();
    // Nmap scan!
    let mut command = Command::new(binary_location);
    command.arg("-A");
    command.arg("-p61209");
    command.arg("-v");
    command.arg("--script=default,vuln");
    command.arg("-oA");
    command.arg("nmap_out/results");
    command.arg(target);
    let res = command.output().unwrap();
    // Make a .zip for scan results
    wrapper_zip_dir("nmap_out", "results.zip");
    // Submit the scan results
    let client = reqwest::Client::new();
    // don't have to read the command output because we're loading files client.post(report_to).body(res.stdout).send().await.unwrap();
    //println!("{}", client.post(report_to).body(fs::read("results.zip").unwrap()).send().await.unwrap().text().await.unwrap());
    client.post(report_to).body(fs::read("results.zip").unwrap()).send().await.unwrap();
    // Delete the local scan result copy
    fs::remove_dir_all("nmap_out").unwrap();
    fs::remove_file("results.zip").unwrap();
    ()
}