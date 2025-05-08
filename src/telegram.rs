use reqwest;
use serde_json::json;
use std::error::Error;

pub struct TelegramBot {
    bot_token: String,
    chat_id: String,
}

impl TelegramBot {
    pub fn new(bot_token: String, chat_id: String) -> Self {
        TelegramBot { bot_token, chat_id }
    }

    pub async fn send_message(&self, text: &str) -> Result<(), Box<dyn Error>> {
        let client = reqwest::Client::new();
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        let response = client.post(&url)
            .json(&json!({
                "chat_id": self.chat_id,
                "text": text
            }))
            .send()
            .await?;

        if response.status().is_success() {
            println!("Message sent successfully!");
        } else {
            println!("Failed to send message: {:?}", response.text().await?);
        }

        Ok(())
    }
}