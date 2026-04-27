use crate::{
    models::telegram::TelegramUpdate, services::gemini::analizar_con_gemini, state::SharedState,
};
use axum::{Json, extract::State, http::StatusCode};

pub async fn manager_receive_message(
    State(_state): State<SharedState>,
    Json(payload): Json<TelegramUpdate>,
) -> StatusCode {
    if let Some(message) = payload.message {
        let chat_id = message.chat.id;
        let chat_username = match message.chat.username {
            Some(uname) => uname,
            None => "Sin_username".to_string(),
        };

        if let Some(text) = message.text {
            println!(
                "Mensaje: {}, chat_id: {}, username: {}",
                text, chat_id, chat_username
            );

            analizar_con_gemini(text, _state).await.unwrap();
        } else {
            println!(
                "Se recibio un mensaje sin texto del chat_id: {}, username: {}",
                chat_id, chat_username
            );
        }
    }

    StatusCode::OK
}
