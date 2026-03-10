use serde::{Deserialize, Serialize};

// ── Request types ──────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Part {
    Text {
        text: String,
    },
    InlineData {
        #[serde(rename = "inlineData")]
        inline_data: InlineData,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineData {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub data: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_modalities: Option<Vec<String>>,
}

// ── Response types ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GeminiResponse {
    pub candidates: Option<Vec<Candidate>>,
}

#[derive(Debug, Deserialize)]
pub struct Candidate {
    pub content: Option<Content>,
}

// ── Builder functions ──────────────────────────────────────────────────

pub fn build_text_request(
    user_message: &str,
    system_prompt: &str,
    temperature: Option<f64>,
) -> GeminiRequest {
    GeminiRequest {
        contents: vec![Content {
            role: Some("user".to_string()),
            parts: vec![Part::Text {
                text: user_message.to_string(),
            }],
        }],
        system_instruction: Some(Content {
            role: None,
            parts: vec![Part::Text {
                text: system_prompt.to_string(),
            }],
        }),
        generation_config: Some(GenerationConfig {
            temperature,
            response_modalities: None,
        }),
    }
}

pub fn build_image_request(
    image_base64: &str,
    mime_type: &str,
    text: &str,
    system_prompt: &str,
    temperature: Option<f64>,
) -> GeminiRequest {
    GeminiRequest {
        contents: vec![Content {
            role: Some("user".to_string()),
            parts: vec![
                Part::InlineData {
                    inline_data: InlineData {
                        mime_type: mime_type.to_string(),
                        data: image_base64.to_string(),
                    },
                },
                Part::Text {
                    text: text.to_string(),
                },
            ],
        }],
        system_instruction: Some(Content {
            role: None,
            parts: vec![Part::Text {
                text: system_prompt.to_string(),
            }],
        }),
        generation_config: Some(GenerationConfig {
            temperature,
            response_modalities: None,
        }),
    }
}

pub fn build_image_generation_request(
    image_base64: &str,
    mime_type: &str,
    text: &str,
    system_prompt: &str,
    temperature: Option<f64>,
) -> GeminiRequest {
    GeminiRequest {
        contents: vec![Content {
            role: Some("user".to_string()),
            parts: vec![
                Part::InlineData {
                    inline_data: InlineData {
                        mime_type: mime_type.to_string(),
                        data: image_base64.to_string(),
                    },
                },
                Part::Text {
                    text: text.to_string(),
                },
            ],
        }],
        system_instruction: Some(Content {
            role: None,
            parts: vec![Part::Text {
                text: system_prompt.to_string(),
            }],
        }),
        generation_config: Some(GenerationConfig {
            temperature,
            response_modalities: Some(vec!["TEXT".to_string(), "IMAGE".to_string()]),
        }),
    }
}

// ── Extractors ─────────────────────────────────────────────────────────

/// Extract the first text part from a Gemini response.
pub fn extract_text(response: &GeminiResponse) -> Option<String> {
    response
        .candidates
        .as_ref()?
        .first()?
        .content
        .as_ref()?
        .parts
        .iter()
        .find_map(|part| match part {
            Part::Text { text } => Some(text.clone()),
            _ => None,
        })
}

/// Extract the first inline image from a Gemini response.
pub fn extract_image(response: &GeminiResponse) -> Option<InlineData> {
    response
        .candidates
        .as_ref()?
        .first()?
        .content
        .as_ref()?
        .parts
        .iter()
        .find_map(|part| match part {
            Part::InlineData { inline_data } => Some(inline_data.clone()),
            _ => None,
        })
}

// ── Unit tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_text_request() {
        let req = build_text_request("Hello", "Be helpful", Some(0.5));
        let json = serde_json::to_value(&req).unwrap();

        assert_eq!(json["contents"][0]["role"], "user");
        assert_eq!(json["contents"][0]["parts"][0]["text"], "Hello");
        assert_eq!(json["systemInstruction"]["parts"][0]["text"], "Be helpful");
        assert_eq!(json["generationConfig"]["temperature"], 0.5);
    }

    #[test]
    fn test_build_image_request() {
        let req = build_image_request("abc123", "image/png", "Describe this", "Be helpful", None);
        let json = serde_json::to_value(&req).unwrap();

        let parts = &json["contents"][0]["parts"];
        assert_eq!(parts[0]["inlineData"]["mimeType"], "image/png");
        assert_eq!(parts[0]["inlineData"]["data"], "abc123");
        assert_eq!(parts[1]["text"], "Describe this");
    }

    #[test]
    fn test_build_image_generation_request() {
        let req =
            build_image_generation_request("img64", "image/jpeg", "Edit this", "Be creative", Some(0.9));
        let json = serde_json::to_value(&req).unwrap();

        let modalities = json["generationConfig"]["responseModalities"]
            .as_array()
            .unwrap();
        assert_eq!(modalities.len(), 2);
        assert_eq!(modalities[0], "TEXT");
        assert_eq!(modalities[1], "IMAGE");
    }

    #[test]
    fn test_extract_text() {
        let raw = serde_json::json!({
            "candidates": [{
                "content": {
                    "parts": [{ "text": "Hello world" }]
                }
            }]
        });
        let resp: GeminiResponse = serde_json::from_value(raw).unwrap();
        assert_eq!(extract_text(&resp), Some("Hello world".to_string()));
    }

    #[test]
    fn test_extract_text_empty() {
        let resp = GeminiResponse { candidates: None };
        assert_eq!(extract_text(&resp), None);
    }

    #[test]
    fn test_extract_image() {
        let raw = serde_json::json!({
            "candidates": [{
                "content": {
                    "parts": [
                        { "text": "Here is the image" },
                        { "inlineData": { "mimeType": "image/png", "data": "base64data" } }
                    ]
                }
            }]
        });
        let resp: GeminiResponse = serde_json::from_value(raw).unwrap();
        let img = extract_image(&resp).unwrap();
        assert_eq!(img.mime_type, "image/png");
        assert_eq!(img.data, "base64data");
    }
}
