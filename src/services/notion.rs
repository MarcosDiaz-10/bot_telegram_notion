use crate::{models::notion::Tarea, state::SharedState};
use reqwest::{Client, header};
use serde_json::{Value, json};

pub async fn obtener_tareas(
    token_notion: String,
) -> Result<Vec<Tarea>, Box<dyn std::error::Error>> {
    let mut headers = header::HeaderMap::new();

    headers.insert(
        "Authorization",
        header::HeaderValue::from_str(&format!("Bearer {}", token_notion))?,
    );
    headers.insert(
        "Notion-Version",
        header::HeaderValue::from_static("2026-03-11"),
    );
    headers.insert(
        "Content-Type",
        header::HeaderValue::from_static("application/json"),
    );

    let body = json!({
        "filter": {
            "and": [{
                "property": "State",
                "status": {
                    "equals": "Not Started"
                }
            },
            {
                "property": "Date",
                "date": {
                    "equals": "today"
                }
            }
            ]

        },
        "sorts": [
            {
                "property": "Priority",
                "direction": "ascending"
            }
        ]
    });

    let client = Client::new();
    let res = client
        .post("https://api.notion.com/v1/data_sources/2e8a13af-9ae8-804f-8e1c-000bfdd3b261/query")
        .headers(headers)
        .json(&body)
        .send()
        .await?;
    if !res.status().is_success() {
        println!("Error de la API: {}", res.status());
    }

    let texto_respuesta = res.text().await?;

    let json_result: Value = serde_json::from_str(&texto_respuesta)?;

    let mut tareas_limpias: Vec<Tarea> = Vec::new();

    if let Some(resultados) = json_result["results"].as_array() {
        for tareas in resultados {
            let titulo = tareas["properties"]["Name"]["title"][0]["plain_text"]
                .as_str()
                .unwrap_or("Sin titulo");
            let curso = tareas["properties"]["Course"]["select"]["name"]
                .as_str()
                .unwrap_or("No tiene curso");
            let tipo = tareas["properties"]["Type"]["select"]["name"]
                .as_str()
                .unwrap_or("No tiene tipo");
            let prioridad = tareas["properties"]["Priority"]["select"]["name"]
                .as_str()
                .unwrap_or("No tiene tipo");

            let tarea = Tarea {
                title: titulo.to_string(),
                curso: curso.to_string(),
                tipo: tipo.to_string(),
                prioridad: prioridad.to_string(),
            };

            tareas_limpias.push(tarea);
        }
    }

    Ok(tareas_limpias)
}

pub async fn obtener_tareas_gemini(
    date: String,
    state: SharedState,
) -> Result<Vec<Tarea>, Box<dyn std::error::Error>> {
    let mut headers = header::HeaderMap::new();

    headers.insert(
        "Authorization",
        header::HeaderValue::from_str(&format!("Bearer {}", state.config.api_notion))?,
    );
    headers.insert(
        "Notion-Version",
        header::HeaderValue::from_static("2026-03-11"),
    );
    headers.insert(
        "Content-Type",
        header::HeaderValue::from_static("application/json"),
    );

    let body = json!({
        "filter": {
            "and": [{
                "property": "State",
                "status": {
                    "equals": "Not Started"
                }
            },
            {
                "property": "Date",
                "date": {
                    "equals": date
                }
            }
            ]

        },
        "sorts": [
            {
                "property": "Priority",
                "direction": "ascending"
            }
        ]
    });

    let client = Client::new();
    let res = client
        .post("https://api.notion.com/v1/data_sources/2e8a13af-9ae8-804f-8e1c-000bfdd3b261/query")
        .headers(headers)
        .json(&body)
        .send()
        .await?;
    if !res.status().is_success() {
        println!("Error de la API: {}", res.status());
    }

    let texto_respuesta = res.text().await?;

    let json_result: Value = serde_json::from_str(&texto_respuesta)?;

    let mut tareas_limpias: Vec<Tarea> = Vec::new();

    if let Some(resultados) = json_result["results"].as_array() {
        for tareas in resultados {
            let titulo = tareas["properties"]["Name"]["title"][0]["plain_text"]
                .as_str()
                .unwrap_or("Sin titulo");
            let curso = tareas["properties"]["Course"]["select"]["name"]
                .as_str()
                .unwrap_or("No tiene curso");
            let tipo = tareas["properties"]["Type"]["select"]["name"]
                .as_str()
                .unwrap_or("No tiene tipo");
            let prioridad = tareas["properties"]["Priority"]["select"]["name"]
                .as_str()
                .unwrap_or("No tiene tipo");

            let tarea = Tarea {
                title: titulo.to_string(),
                curso: curso.to_string(),
                tipo: tipo.to_string(),
                prioridad: prioridad.to_string(),
            };

            tareas_limpias.push(tarea);
        }
    }

    Ok(tareas_limpias)
}

pub async fn mark_done(
    name: &str,
    state: SharedState,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut headers = header::HeaderMap::new();

    headers.insert(
        "Authorization",
        header::HeaderValue::from_str(&format!("Bearer {}", state.config.api_notion))?,
    );
    headers.insert(
        "Notion-Version",
        header::HeaderValue::from_static("2026-03-11"),
    );
    headers.insert(
        "Content-Type",
        header::HeaderValue::from_static("application/json"),
    );

    let body = json!({
        "filter": {

                "property": "Name",
                "title": {
                    "contains": name
                }
        }
    });

    let res = state
        .http_client
        .post("https://api.notion.com/v1/data_sources/2e8a13af-9ae8-804f-8e1c-000bfdd3b261/query")
        .headers(headers.clone())
        .json(&body)
        .send()
        .await?;
    if !res.status().is_success() {
        println!("Error de la API: {}", res.status());
        return Ok(format!("Error de la API: {}", res.text().await?));
    }

    let texto_respuesta = res.text().await?;

    let json_result: Value = serde_json::from_str(&texto_respuesta)?;

    if let Some(results) = json_result["results"].as_array() {
        if let Some(tarea) = results.first() {
            let id = tarea["id"].as_str().unwrap_or("");
            let update_body = json!({
                "properties": {
                    "State": {
                        "status": {
                            "id": "d40a0f53-90a1-4d4c-905b-a840724ae64a",
                            "name": "Done"
                        }
                    }
                }
            });

            let update_res = state
                .http_client
                .patch(&format!("https://api.notion.com/v1/pages/{}", id))
                .headers(headers.clone())
                .json(&update_body)
                .send()
                .await?;

            if !update_res.status().is_success() {
                println!("Error al actualizar la tarea: {}", update_res.text().await?);
            } else {
                println!("Tarea '{}' marcada como completada", name);
                return Ok(format!("Tarea **{}** marcada como completada", name));
            }
        } else {
            println!("No se encontró la tarea con el nombre '{}'", name);
            return Ok(format!("No se encontró la tarea con el nombre '{}'", name));
        }
    }

    Ok("Tarea marcada como completada".to_string())
}
