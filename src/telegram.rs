pub async fn enviar_mensaje(
    msg: String,
    api_telegram: String,
    chat_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client
        .post(&format!(
            "https://api.telegram.org/bot{}/sendMessage",
            api_telegram
        ))
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text": msg,
            "parse_mode": "MarkdownV2"
        }))
        .send()
        .await?;

    if res.status().is_success() {
        println!("Mensaje enviado correctamente");
    } else {
        println!(
            "Error al enviar el mensaje: {}",
            res.text()
                .await
                .unwrap_or_else(|_| "No se pudo obtener el error".to_string())
        );
    }

    Ok(())
}
