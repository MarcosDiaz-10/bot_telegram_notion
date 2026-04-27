use dotenvy::dotenv;
use services::{notion, telegram};
use state::{AppConfig, AppState};
use std::{env::var, sync::Arc};
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{routes::router, services::jobs::get_task_send_message};

mod models;
mod routes;
mod services;
mod state;
#[tokio::main]

async fn main() {
    dotenv().expect("No se pudo encontrar el archivo .env");

    let config = AppConfig {
        api_notion: var("API_NOTION").expect("API_NOTION no encontrada en .env"),
        api_telegram: var("API_TELEGRAM").expect("API_TELEGRAM no encontrada en .env"),
        chat_id: var("CHAT_ID").expect("CHAT_ID no encontrada en .env"),
    };

    let shared_state = Arc::new(AppState {
        config,
        http_client: reqwest::Client::new(),
    });

    let sched = JobScheduler::new().await.unwrap();
    let job_reportes_shared_state = Arc::clone(&shared_state);
    let job_reportes = Job::new_async("0 0 12,17,21 * * *", move |_uuid, _l| {
        let state = Arc::clone(&job_reportes_shared_state);
        Box::pin(async move {
            println!("Ejecutando tarea programada: Enviar reporte de tareas");
            get_task_send_message(
                state.config.api_notion.clone(),
                state.config.api_telegram.clone(),
                state.config.chat_id.clone(),
            )
            .await
        })
    });

    let jobs = match job_reportes {
        Ok(job) => job,
        Err(e) => {
            println!("Error al crear el job: {}", e);
            return;
        }
    };

    sched.add(jobs).await.unwrap();

    tokio::spawn(async move {
        println!("Iniciando scheduler de tareas programadas");
        sched.start().await.unwrap();
    });

    let app = router(shared_state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Servidor escuchando en http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}
