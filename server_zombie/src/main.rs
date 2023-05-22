mod nmap;
mod nikto;
mod amass;

use nmap::nmap;
use nikto::nikto;

use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse};
use std::fs;
use serde::Deserialize;
use crate::nmap::nmap_version;
use crate::nikto::nikto_version;
use crate::amass::amass_version;
use futures::StreamExt;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use actix_web::web::BytesMut;

#[derive(Debug, Clone)]
enum Task {
    Nmap(NmapTask),
    Nikto(NiktoTask),
    Amass(AmassTask),
}

#[derive(Debug, Clone)]
struct NmapTask {
    target: String,
    report_to: String,
}

#[derive(Deserialize)]
struct NmapInfo {
    report_to: String,
}

#[derive(Debug, Clone)]
struct NiktoTask {
    target: String,
    report_to: String,
}

#[derive(Deserialize)]
struct AmassInfo {
    report_to: String,
}

#[derive(Debug, Clone)]
struct AmassTask {
    target: String,
    report_to: String,
}

#[derive(Deserialize)]
struct NiktoInfo {
    report_to: String,
}

#[get("/tasks")]
async fn tasks(counter: web::Data<Arc<AtomicUsize>>) -> impl Responder {
    let count = counter.load(Ordering::SeqCst);
    format!("{}", count)
}

// Start an nmap scan
#[get("/nmap/{target}")]
async fn nmap_scan(
    target: web::Path<String>,
    nmap_info: web::Query<NmapInfo>,
    task_sender: web::Data<tokio::sync::mpsc::Sender<Task>>,
    counter: web::Data<Arc<AtomicUsize>>, // get the counter from app data
) -> impl Responder {
    let task = NmapTask {
        target: target.into_inner(),
        report_to: nmap_info.report_to.clone(),
    };
    task_sender.send(Task::Nmap(task)).await.unwrap();
    counter.fetch_add(1, Ordering::SeqCst); // increment the counter
    format!("Doing nmap and reporting back to {0}!", nmap_info.report_to)
}

// Start a nikto scan
#[get("/nikto/{target}")]
async fn nikto_scan(
    target: web::Path<String>,
    nikto_info: web::Query<NiktoInfo>,
    task_sender: web::Data<tokio::sync::mpsc::Sender<Task>>,
    counter: web::Data<Arc<AtomicUsize>>, // get the counter from app data
) -> impl Responder {
    let task = NiktoTask {
        target: target.into_inner(),
        report_to: nikto_info.report_to.clone(),
    };
    task_sender.send(Task::Nikto(task)).await.unwrap();
    counter.fetch_add(1, Ordering::SeqCst); // increment the counter
    format!("Doing nikto and reporting back to {0}!", nikto_info.report_to)
}

// Start an Amass scan
#[post("/amass/{target}")]
async fn amass_scan(mut payload: web::Payload,
    target: web::Path<String>,
    amass_info: web::Query<AmassInfo>,
    task_sender: web::Data<tokio::sync::mpsc::Sender<Task>>,
    counter: web::Data<Arc<AtomicUsize>>, // get the counter from app data
) -> impl Responder {
    HttpResponse::Ok().body(format!("Thank you"));
    let mut body = BytesMut::new();
    while let Some(chunk) = payload.next().await {
        body.extend_from_slice(&chunk.unwrap());
    }
    let filename = "amass".to_owned()+".zip";
    tokio::fs::write(filename, body.freeze()).await.unwrap();
    // unzip here
    let task = AmassTask {
        target: target.into_inner(),
        report_to: amass_info.report_to.clone(),
    };
    task_sender.send(Task::Amass(task)).await.unwrap();
    counter.fetch_add(1, Ordering::SeqCst); // increment the counter
    format!("Doing amass and reporting back to {0}!", amass_info.report_to)
}

// Get nmap version info
#[get("/nmap")]
async fn nmap_get_version() -> impl Responder {
    let version = nmap_version();
    format!("{}", version)
}

// Get nikto version info
#[get("/nikto")]
async fn nikto_get_version() -> impl Responder {
    let version = nikto_version();
    format!("{}", version)
}

// Get amass version info
#[get("/amass")]
async fn amass_get_version() -> impl Responder {
    let version = amass_version();
    format!("{}", version)
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let task_counter = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&task_counter);
    let (task_sender, mut task_receiver) = tokio::sync::mpsc::channel::<Task>(100);
    tokio::spawn(async move {
        while let Some(task) = task_receiver.recv().await {
            match task {
                Task::Nmap(nmap_task) => {
                    nmap(nmap_task.target, nmap_task.report_to).await;
                }
                Task::Nikto(nikto_task) => {
                    nikto(nikto_task.target, nikto_task.report_to).await;
                }
                Task::Amass(_) => todo!()
            }
            // Decrement counter after task is finished
            counter.fetch_sub(1, Ordering::SeqCst);
        }
    });
    HttpServer::new(move || {
        App::new()
            .data(task_sender.clone()) // clone the sender for each worker
            .data(task_counter.clone()) // clone the counter for each worker
            .service(nmap_scan)
            .service(nmap_get_version)
            .service(nikto_scan)
            .service(nikto_get_version)
            .service(tasks)
    })
        .bind(("127.0.0.1", 8088))?
        .run()
        .await
}
