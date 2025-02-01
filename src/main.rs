use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json;
use serde_yaml;
use std::error::Error;
use std::fs;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Deserialize)]
struct Config {
    notification: NotificationConfig,
    #[serde(default = "default_monitor_interval_sec")]
    monitor_interval_sec: u64,  // Wait time between checks, in seconds
    gpus: Vec<GpuConfig>,
}

// If monitor_interval_sec is not in the YAML, default to 60 seconds
fn default_monitor_interval_sec() -> u64 {
    60
}

#[derive(Debug, Deserialize)]
struct NotificationConfig {
    method: String,               // "discord" or "sms"
    #[serde(default)]
    discord_webhook_url: String,  // only needed if method == "discord"
    #[serde(default)]
    twilio_account_sid: String,   // only needed if method == "sms"
    #[serde(default)]
    twilio_auth_token: String,    // only needed if method == "sms"
    #[serde(default)]
    twilio_from_number: String,   // only needed if method == "sms"
    #[serde(default)]
    twilio_to_number: String,     // only needed if method == "sms"
}

#[derive(Debug, Deserialize)]
struct GpuConfig {
    name: String,
    url: String,
    #[serde(default)]
    in_stock_selector: String,  // The CSS selector used to check if it's in stock
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Read and parse the YAML configuration
    let config_text = fs::read_to_string("config.yaml")?;
    let config: Config = serde_yaml::from_str(&config_text)?;

    let client = Client::new();

    loop {
        // 2. Loop through each GPU and check stock
        for gpu in &config.gpus {
            match check_stock(&client, gpu).await {
                Ok(in_stock) => {
                    if in_stock {
                        println!("[IN STOCK] {}", gpu.name);
                        // 3. Send notification
                        match config.notification.method.as_str() {
                            "discord" => {
                                send_discord_notification(
                                    &client,
                                    &config.notification.discord_webhook_url,
                                    &gpu.name,
                                    &gpu.url,
                                )
                                .await?;
                            }
                            "sms" => {
                                send_sms_notification(
                                    &client,
                                    &config.notification,
                                    &gpu.name,
                                    &gpu.url,
                                )
                                .await?;
                            }
                            other => {
                                eprintln!("Unsupported notification method: {}", other);
                            }
                        }
                    } else {
                        println!("[OUT OF STOCK] {}", gpu.name);
                    }
                }
                Err(err) => {
                    eprintln!("Error checking stock for {}: {}", gpu.name, err);
                }
            }
        }

        println!("Waiting {} seconds before next check...", config.monitor_interval_sec);
        sleep(Duration::from_secs(config.monitor_interval_sec)).await;

        // OPTIONAL: If using Ctrl-C to exit, check if we got the shutdown signal:
        // if shutdown_signal.is_finished() {
        //     println!("Shutting down continuous loop...");
        //     break;
        // }
    }
}

/// Checks if a given GPU is in stock by scraping the web page.
async fn check_stock(client: &Client, gpu: &GpuConfig) -> Result<bool, Box<dyn Error>> {
    let resp = client.get(&gpu.url).send().await?.text().await?;
    let document = Html::parse_document(&resp);

    if gpu.in_stock_selector.is_empty() {
        // If no selector is provided, do a text-based check:
        return Ok(resp.contains("In Stock") || resp.contains("Add to Cart"));
    } else {
        // If the site is dynamic, you might need something more advanced (like a headless browser).
        let selector = match Selector::parse(&gpu.in_stock_selector) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("Failed to parse CSS selector '{}': {:?}", &gpu.in_stock_selector, e).into());
            }
        };
        let in_stock = document.select(&selector).next().is_some();
        Ok(in_stock)
    }
}

/// Send a message to Discord via a webhook.
async fn send_discord_notification(
    client: &Client,
    webhook_url: &str,
    gpu_name: &str,
    gpu_url: &str,
) -> Result<(), Box<dyn Error>> {
    let content = format!("**GPU In Stock**: {}\n{}", gpu_name, gpu_url);

    let body = serde_json::json!({
        "content": content
    });

    let res = client.post(webhook_url).json(&body).send().await?;
    if !res.status().is_success() {
        eprintln!("Failed to send Discord notification. Status: {}", res.status());
    }

    Ok(())
}

/// Send an SMS via Twilio (simple example with direct REST call).
async fn send_sms_notification(
    client: &Client,
    notif_config: &NotificationConfig,
    gpu_name: &str,
    gpu_url: &str,
) -> Result<(), Box<dyn Error>> {
    let twilio_url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
        notif_config.twilio_account_sid
    );

    let message_body = format!("GPU In Stock: {} - {}", gpu_name, gpu_url);

    let params = [
        ("From", notif_config.twilio_from_number.as_str()),
        ("To", notif_config.twilio_to_number.as_str()),
        ("Body", &message_body),
    ];

    let res = client
        .post(&twilio_url)
        .basic_auth(
            &notif_config.twilio_account_sid,
            Some(&notif_config.twilio_auth_token),
        )
        .form(&params)
        .send()
        .await?;

    if !res.status().is_success() {
        eprintln!("Failed to send SMS notification. Status: {}", res.status());
    }

    Ok(())
}
