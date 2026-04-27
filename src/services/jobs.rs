use crate::services::{notion::obtener_tareas, telegram};

pub async fn get_task_send_message(api_notion: String, api_telegram: String, chat_id: String) {
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

    telegram::enviar_mensaje(msg, api_telegram, chat_id)
        .await
        .unwrap();
}
