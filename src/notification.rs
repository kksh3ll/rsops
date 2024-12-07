use crate::alerting::Alert;
use async_trait::async_trait;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use reqwest::Client;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotificationError {
    #[error("Failed to send email: {0}")]
    EmailError(String),
    #[error("Failed to send Slack notification: {0}")]
    SlackError(String),
    #[error("Failed to send notification: {0}")]
    GeneralError(String),
}

#[async_trait]
pub trait NotificationSender {
    async fn send(&self, alert: &Alert) -> Result<(), NotificationError>;
}

pub struct EmailNotifier {
    smtp_server: String,
    smtp_port: u16,
    username: String,
    password: String,
    from_address: String,
    to_address: String,
}

pub struct SlackNotifier {
    webhook_url: String,
    channel: String,
}

impl EmailNotifier {
    pub fn new(
        smtp_server: String,
        smtp_port: u16,
        username: String,
        password: String,
        from_address: String,
        to_address: String,
    ) -> Self {
        Self {
            smtp_server,
            smtp_port,
            username,
            password,
            from_address,
            to_address,
        }
    }
}

#[async_trait]
impl NotificationSender for EmailNotifier {
    async fn send(&self, alert: &Alert) -> Result<(), NotificationError> {
        let email = Message::builder()
            .from(self.from_address.parse().map_err(|e| NotificationError::EmailError(e.to_string()))?)
            .to(self.to_address.parse().map_err(|e| NotificationError::EmailError(e.to_string()))?)
            .subject(format!("[{}] {} Alert: {}", alert.severity, alert.source, alert.message))
            .body(format!(
                "Alert Details:\n\nSource: {}\nSeverity: {:?}\nMessage: {}\nDetails: {}\nTimestamp: {}",
                alert.source, alert.severity, alert.message, alert.details, alert.timestamp
            ))
            .map_err(|e| NotificationError::EmailError(e.to_string()))?;

        let creds = Credentials::new(self.username.clone(), self.password.clone());

        let mailer = SmtpTransport::relay(&self.smtp_server)
            .map_err(|e| NotificationError::EmailError(e.to_string()))?
            .port(self.smtp_port)
            .credentials(creds)
            .build();

        mailer.send(&email)
            .map_err(|e| NotificationError::EmailError(e.to_string()))?;

        Ok(())
    }
}

impl SlackNotifier {
    pub fn new(webhook_url: String, channel: String) -> Self {
        Self {
            webhook_url,
            channel,
        }
    }
}

#[async_trait]
impl NotificationSender for SlackNotifier {
    async fn send(&self, alert: &Alert) -> Result<(), NotificationError> {
        let client = Client::new();

        let text = format!(
            "*[{}] {} Alert*\n>Message: {}\n>Details: {}\n>Timestamp: {}",
            alert.severity, alert.source, alert.message, alert.details, alert.timestamp
        );

        let payload = json!({
            "channel": self.channel,
            "text": text,
        });

        client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| NotificationError::SlackError(e.to_string()))?;

        Ok(())
    }
}
