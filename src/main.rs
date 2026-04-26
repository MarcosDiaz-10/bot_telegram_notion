use dotenvy::dotenv;
use std::env::var;

use crate::{notion::obtener_tareas, telegram::enviar_mensaje};

mod notion;
mod telegram;
#[tokio::main]

async fn main() {
    dotenv().expect("No se pudo encontrar el archivo .env");

    let api_notion = var("API_NOTION").expect("API_NOTION no encontrada en .env");
    let api_telegram = var("API_TELEGRAM").expect("API_TELEGRAM no encontrada en .env");
    let chat_id = var("CHAT_ID").expect("CHAT_ID no encontrada en .env");

    let tareas = obtener_tareas(api_notion).await.unwrap();

    let mut msg = format!(
        "🏆 *REPORTE TAREAS DE HOY* 🏆 \n \n Sr\\.Diaz, tienes {} tareas \n\n",
        tareas.len()
    );
    let mut count = 1;
    for tarea in &tareas {
        msg += &format!(
            "*{}\\. {}*\n├ ⚡ *Prioridad:* {}\n├ 🛠️ *Tipo:* {}\n└ 🎓 *Curso:* {}\n\n",
            count, tarea.title, tarea.prioridad, tarea.tipo, tarea.curso
        );
        count += 1;
    }

    enviar_mensaje(msg, api_telegram, chat_id).await.unwrap();
}
