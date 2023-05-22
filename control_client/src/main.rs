mod zip;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, post};
use actix_web::web::BytesMut;
use rand::{distributions::Alphanumeric, Rng};
use futures::StreamExt;

const ZOMBIES: [&str; 1] = ["http://127.0.0.1:8088"];
const TARGET: &str = "your-target-website.com";
const SERVER_ADDR: &str = "127.0.0.1:8084";

type SharedData = Arc<Mutex<HashMap<String, String>>>;

async fn find_least_used_zombie(zombies: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let mut min_tasks = std::usize::MAX;
    let mut min_zombie = "";

    for &zombie in zombies {
        let tasks_str = reqwest::get(format!("{}/tasks", zombie)).await?.text().await?;
        let tasks: usize = tasks_str.trim().parse().map_err(|_| "Failed to parse tasks into a number")?;

        if tasks < min_tasks {
            min_tasks = tasks;
            min_zombie = zombie;
        }
    }

    Ok(min_zombie.to_string())
}

async fn generate_unique_mutex(task_mutexes: &SharedData) -> String {
    loop {
        let mutex = rand::thread_rng().sample_iter(&Alphanumeric).take(7).map(char::from).collect();
        let tasks = task_mutexes.lock().unwrap();
        if !tasks.contains_key(&mutex) {
            return mutex;
        }
    }
}

// Sending tasks to our zombie
async fn submit_scan_to_zombie(mutex: &str, scan_type: &str, target: &str, zombie: &str) -> Result<String, Box<dyn std::error::Error>> {
    if scan_type == "amass" {
        let file = tokio::fs::File::open("amass_config.zip").await?;
        let client = reqwest::Client::new();
        let response = client.post(&format!("{}/{}/{}?report_to=http://{}/report/{}", zombie, scan_type, target, SERVER_ADDR, mutex))
            .header("Content-Type", "application/zip")
            .body(file)
            .send()
            .await?
            .text()
            .await?;
        Ok(response)
    } else {
        let response = reqwest::get(format!("{}/{}/{}?report_to=http://{}/report/{}", zombie, scan_type, target, SERVER_ADDR, mutex)).await?.text().await?;
        Ok(response)
    }
}

#[post("/report/{mutex}")]
async fn report(mut payload: web::Payload, mutex: web::Path<String>, data: web::Data<SharedData>) -> impl Responder {
    let mutex_str = mutex.into_inner();
    let tasks = data.lock().unwrap();
    if let Some(task) = tasks.get(&mutex_str) {
        HttpResponse::Ok().body(format!("Thank you"));
        let mut body = BytesMut::new();
        while let Some(chunk) = payload.next().await {
            body.extend_from_slice(&chunk.unwrap());
        }
        let filename = task.to_owned()+".zip";
        tokio::fs::write(filename, body.freeze()).await.unwrap();
        format!("Thank you, Nice:)")
    } else {
        format!("Thank u {}! But we have no task with this mutex.", mutex_str)
    }
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a zip file for amass config
    zip::zip_file("amass.ini", "amass.zip");

    let task_mutexes: SharedData = Arc::new(Mutex::new(HashMap::new()));
    let least_used_zombie = find_least_used_zombie(&ZOMBIES).await?;
    let server_data = web::Data::new(task_mutexes.clone());
    let server = HttpServer::new(move || {
        App::new()
            .app_data(server_data.clone())
            .service(report)
    })
        .bind(SERVER_ADDR)?.run();

    // Give the server some time to set up
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Generate a unique mutex and submit scan for each task
    for &task in &["nmap", "nikto", "amass"] {
        let mutex = generate_unique_mutex(&task_mutexes).await;
        {
            let mut tasks = task_mutexes.lock().unwrap();
            tasks.insert(mutex.clone(), format!("{}-{}", task, TARGET));
        }
        println!("{}", submit_scan_to_zombie(&mutex, task, TARGET, &least_used_zombie).await?);
    }

    // Wait for all tasks to complete
    //while task_mutexes.lock().unwrap().len() > 0 {
    //    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    //}

    // Delete the zip file for amass config
    fs::remove_file("amass.zip").unwrap();

    // Stop the server and exit the program
    server.await.unwrap();

    Ok(())
}
