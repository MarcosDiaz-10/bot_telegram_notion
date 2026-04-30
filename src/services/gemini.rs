use std::{error::Error, sync::Arc};

use axum::Json;
use serde_json::{Value, json};

use crate::{
    services::{
        notion::{mark_done, obtener_tareas_gemini},
        telegram::enviar_mensaje,
    },
    state::{self, SharedState},
};

pub async fn analizar_con_gemini(texto: String, state: SharedState) -> Result<(), Box<dyn Error>> {
    let url =
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent"
            .to_string();

    let fecha_hoy = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let instrucciones_sistema = format!(
        "Rol y Objetivo
        
        Sos el agente personal de productividad y automatización del Sr. Díaz. Tu único propósito es interpretar sus mensajes entrantes a través de Telegram, extraer la información clave y ejecutar las llamadas a funciones (tool calls) correspondientes para interactuar con su base de datos en Notion.

        Contexto Temporal
        Hoy es: {} (Utilizá esta fecha como referencia absoluta para interpretar términos relativos como 'mañana', 'la semana que viene', 'el próximo martes').

        Reglas de Operación (ESTRICTAS)

        Cero Suposiciones: Si el mensaje del Sr. Díaz no contiene la información mínima requerida para ejecutar una función de Notion (por ejemplo,el título de un apunte), NO inventes los datos. Ejecutá una respuesta pidiendo la información faltante antes de llamar a la herramienta de Notion.

        Prioridad de Ejecución: Analizá el mensaje. Si contiene una orden clara, tu primera acción debe ser llamar a la función correspondiente.

        Confirmación y Tono: Después de ejecutar una función con éxito, respondé siempre de forma breve, profesional y al grano. Usá un tono eficiente. 

        Manejo de Errores: Si una función de Notion devuelve un error, informale al Sr. Díaz exactamente qué falló sin usar jerga técnica innecesaria, y preguntale cómo desea proceder.

        Fuera de Contexto: Si el mensaje del Sr. Díaz es una charla casual o no requiere una acción en Notion, respondé de manera conversacional y recordale que estás listo para gestionar su productividad cuando lo necesite.
        
        Forma de la base de datos en Notion

        Tiene las siguientes propiedades:
        - Name (Título)
        - Priority (Select: High, Mid, Low)
        - State (Select: Not started, In Progress, Done)
        - Type (Select: University, Projects, Personal, General, Samantha)
        - Date (Date) (Algunas no tiene fecha de asignación, es opcional)
        - Course (Preparaduria Comu, Tutor AyED, CiberSec, Otros) (Algunas tareas pueden no tener este campo, es opcional)

        es necesario que el texto que me devuelvas sea compatible para enviarlo por telegram usando MarkdownV2, por lo que debes escapar los caracteres especiales de MarkdownV2 como guiones, puntos, paréntesis, etc. para evitar errores al enviar el mensaje. Si no especifico fecha toma la de hoy.
    ",
        fecha_hoy
    );

    let body = json!({
            "system_instruction": {
                "parts": [{"text": instrucciones_sistema}]
            },
            "contents": [{
                "parts": [{ "text": texto }]
            }],
            "tools": [{
                "functionDeclarations": [{
                    "name": "marcar_tarea_como_completa",
                    "description": "Marca una tarea como completa en la base de datos",
                    "parameters": {
                        "type": "OBJECT",
                        "properties": {
                            "Name": {
                                "type": "STRING",
                                "description": "El nombre descriptivo y conciso de la tarea"
                            },
                        },
                        "required": ["Name"]
                    }
                },
    {
                    "name": "obtener_tareas",
                    "description": "Obtiene la lista de tareas en la base de datos",
                    "parameters": {
                        "type": "OBJECT",
                        "properties": {
                            "Name": {
                                "type": "STRING",
                                "description": "El nombre descriptivo y conciso de la tarea"
                            },
                            "Course": {
                                "type": "STRING",
                                "description": "El curso al que pertenece la tarea, si es que tiene uno asignado"
                            },
                            "Prioridad": {
                                "type": "STRING",
                                "description": "La prioridad de la tarea, si es que tiene una asignada"
                            },
                            "State": {
                                "type": "STRING",
                                "description": "El estado de la tarea, si es que tiene uno asignado"
                            },
                            "Type": {
                                "type": "STRING",
                                "description": "El tipo de tarea, si es que tiene uno asignado"
                            },
                            "Date": {
                                "type": "STRING",
                                "description": "La fecha de la tarea"
                            },
                        },
                        "required": ["Name","Course","Prioridad","State","Type","Date"]
                    }
                }
                ]
            }]
        });

    let res = state
        .http_client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("x-goog-api-key", &state.config.api_gemini)
        .json(&body)
        .send()
        .await?;

    let json_response: Value = res.json().await?;
    println!("Respuesta de Gemini: {}", json_response);
    let parts = &json_response["candidates"][0]["content"]["parts"][0];

    if let Some(function_call) = parts.get("functionCall") {
        let function_name = function_call["name"].as_str().unwrap_or("");

        let function_result = llamar_funcion_gemini(
            function_name,
            function_call["args"].clone(),
            Arc::clone(&state),
        )
        .await?;

        enviar_mensaje(
            function_result,
            state.config.api_telegram.clone(),
            state.config.chat_id.clone(),
            "MarkdownV2".to_string(),
        )
        .await
        .unwrap()
    } else {
        println!("Respuesta de Gemini sin llamada a función:");
        println!(
            "{}",
            parts["text"]
                .as_str()
                .unwrap_or("No se llamo a la función y gemini no respondió")
        );
        enviar_mensaje(
            format!(
                "{}",
                parts["text"]
                    .as_str()
                    .unwrap_or("No se llamo a la función y gemini no respondió")
            ),
            state.config.api_telegram.clone(),
            state.config.chat_id.clone(),
            "MarkdownV2".to_string(),
        )
        .await
        .unwrap()
    }

    Ok(())
}

async fn llamar_funcion_gemini(
    function_name: &str,
    arguments: Value,
    state: SharedState,
) -> Result<String, Box<dyn Error>> {
    match function_name {
        "marcar_tarea_como_completa" => {
            let name = arguments["Name"].as_str().unwrap_or("");
            let str = mark_done(name, state).await?;
            return Ok(str);
        }
        "obtener_tareas" => {
            let name = arguments["Name"].as_str().unwrap_or("Any");
            let course = arguments["Course"].as_str().unwrap_or("Any");
            let prioridad = arguments["Prioridad"].as_str().unwrap_or("Any");
            let stateGemini = arguments["State"].as_str().unwrap_or("Any");
            let type_ = arguments["Type"].as_str().unwrap_or("Any");
            let date = arguments["Date"].as_str().unwrap_or("Any");
            let str = obtener_tareas_gemini(
                name.to_string(),
                course.to_string(),
                prioridad.to_string(),
                stateGemini.to_string(),
                type_.to_string(),
                date.to_string(),
                state,
            )
            .await?;
            return Ok(str);
        }
        _ => {
            println!("Función desconocida: {}", function_name);
            return Ok(format!("Función desconocida: {}", function_name));
        }
    }
}
