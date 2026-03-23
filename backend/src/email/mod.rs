/// 邮件通知模块
/// 支持发送流量告警、到期提醒等通知邮件

use lettre::{
    Message, SmtpTransport, Transport,
    message::{header::ContentType, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
};
use std::env;
use tracing::info;

mod templates;



/// 邮件客户端配置
#[derive(Clone)]
pub struct EmailClient {
    smtp_server: String,
    smtp_port: u16,
    username: String,
    password: String,
    from_email: String,
    from_name: String,
}

impl EmailClient {
    /// 从环境变量创建邮件客户端
    pub fn from_env() -> Option<Self> {
        let smtp_server = env::var("SMTP_SERVER").ok()?;
        let smtp_port = env::var("SMTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(587);
        let username = env::var("SMTP_USERNAME").ok()?;
        let password = env::var("SMTP_PASSWORD").ok()?;
        let from_email = env::var("SMTP_FROM_EMAIL")
            .ok()
            .unwrap_or_else(|| username.clone());
        let from_name = env::var("SMTP_FROM_NAME")
            .ok()
            .unwrap_or_else(|| "LX-Proxy".to_string());

        Some(Self {
            smtp_server,
            smtp_port,
            username,
            password,
            from_email,
            from_name,
        })
    }

    /// 创建 SMTP 传输
    fn create_transport(&self) -> SmtpTransport {
        SmtpTransport::relay(&self.smtp_server)
            .unwrap()
            .credentials(Credentials::new(
                self.username.clone(),
                self.password.clone(),
            ))
            .port(self.smtp_port)
            .build()
    }

    /// 发送邮件
    pub fn send_email(
        &self,
        to_email: &str,
        to_name: &str,
        subject: &str,
        html_body: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let from = Mailbox::new(
            Some(self.from_name.clone()),
            self.from_email.parse()?,
        );
        let to = Mailbox::new(Some(to_name.to_string()), to_email.parse()?);

        let email = Message::builder()
            .from(from)
            .to(to)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(html_body.clone()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html_body),
                    ),
            )?;

        let mailer = self.create_transport();
        mailer.send(&email)?;

        info!("📧 Email sent to {} ({})", to_email, subject);
        Ok(())
    }

    /// 发送流量告警邮件
    pub fn send_traffic_alert(
        &self,
        to_email: &str,
        to_name: &str,
        inbound_name: &str,
        usage_percent: f64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let subject = format!("⚠️ 流量告警 - {}", inbound_name);
        let html_body = templates::traffic_alert(inbound_name, usage_percent);
        self.send_email(to_email, to_name, &subject, html_body)
    }

    /// 发送到期提醒邮件
    pub fn send_expiry_reminder(
        &self,
        to_email: &str,
        to_name: &str,
        inbound_name: &str,
        days_left: u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let subject = format!("⏰ 到期提醒 - {}", inbound_name);
        let html_body = templates::expiry_reminder(inbound_name, days_left);
        self.send_email(to_email, to_name, &subject, html_body)
    }

    /// 发送禁用通知邮件
    pub fn send_disabled_notification(
        &self,
        to_email: &str,
        to_name: &str,
        inbound_name: &str,
        reason: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let subject = format!("🚫 服务已禁用 - {}", inbound_name);
        let html_body = templates::disabled_notification(inbound_name, reason);
        self.send_email(to_email, to_name, &subject, html_body)
    }

    /// 测试邮件连接
    pub fn test_connection(&self, to_email: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.send_email(
            to_email,
            "Admin",
            "✅ LX-Proxy 邮件测试",
            templates::test_email(),
        )
    }
}

/// 检查邮件是否已配置
pub fn is_email_configured() -> bool {
    env::var("SMTP_SERVER").is_ok()
        && env::var("SMTP_USERNAME").is_ok()
        && env::var("SMTP_PASSWORD").is_ok()
}
