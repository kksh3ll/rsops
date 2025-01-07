mod alerting;
mod container_monitor;
mod notification;
mod resource_monitor;

use alerting::AlertRule;
use alerting::{ContainerAlertRule, ResourceAlertRule, ResourceThreshold};
use container_monitor::{ContainerMonitor, DockerContainerMonitor};
use notification::{EmailNotifier, NotificationSender, SlackNotifier};
use resource_monitor::{ResourceMonitor, SystemResourceMonitor};
use std::sync::Arc;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Initialize monitors
    let resource_monitor = SystemResourceMonitor::new();
    let container_monitor = DockerContainerMonitor::new()?;

    // Initialize notification senders
    let email_notifier = EmailNotifier::new(
        "smtp.example.com".to_string(),
        587,
        "username".to_string(),
        "password".to_string(),
        "from@example.com".to_string(),
        "to@example.com".to_string(),
    );

    let slack_notifier = SlackNotifier::new(
        "https://hooks.slack.com/services/your/webhook/url".to_string(),
        "#monitoring".to_string(),
    );

    let notifiers: Vec<Arc<dyn NotificationSender + Send + Sync>> =
        vec![Arc::new(email_notifier), Arc::new(slack_notifier)];

    // Set monitoring thresholds
    let resource_threshold = ResourceThreshold {
        cpu_threshold: 80.0,    // 80% CPU usage
        memory_threshold: 90.0, // 90% memory usage
        disk_threshold: 85.0,   // 85% disk usage
    };

    println!("Starting monitoring system...");

    let mut interval = time::interval(Duration::from_secs(60));

    loop {
        interval.tick().await;

        // Collect metrics
        if let Ok(metrics) = resource_monitor.collect_metrics().await {
            // Evaluate resource alerts
            let resource_rule = ResourceAlertRule::new(resource_threshold.clone(), metrics);
            if let Ok(Some(alert)) = resource_rule.evaluate().await {
                // Send notifications
                for notifier in &notifiers {
                    if let Err(e) = notifier.send(&alert).await {
                        log::error!("Failed to send notification: {}", e);
                    }
                }
            }
        }

        // Monitor containers
        if let Ok(containers) = container_monitor.list_containers().await {
            for container in containers {
                let container_rule = ContainerAlertRule { container };
                if let Ok(Some(alert)) = container_rule.evaluate().await {
                    // Send notifications
                    for notifier in &notifiers {
                        if let Err(e) = notifier.send(&alert).await {
                            log::error!("Failed to send notification: {}", e);
                        }
                    }
                }
            }
        }
    }
}
