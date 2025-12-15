use std::sync::Arc;
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use log::{info, error};
use crate::error::AppError;
use crate::models::req::email_req::SendEmailReq;
use crate::models::config_mapping::gmail_config::GmailConfig;
use crate::services::config_service::ConfigService;
use super::EmailService;

/// Gmail (SMTP) 邮件服务实现
pub struct GmailEmailService {
    config_service: Arc<ConfigService>,
}

impl GmailEmailService {
    pub fn new(config_service: Arc<ConfigService>) -> Self {
        Self { config_service }
    }
}

#[async_trait::async_trait]
impl EmailService for GmailEmailService {
    fn name(&self) -> &str {
        "gmail"
    }

    async fn send(&self, req: SendEmailReq) -> Result<(), AppError> {
        let config_service = self.config_service.clone();
        let req_clone = req.clone();
        
        // 异步发送邮件
        tokio::spawn(async move {
            if let Err(e) = Self::do_send(config_service, req_clone).await {
                error!("发送邮件失败: {:?}", e);
            }
        });
        
        Ok(())
    }
}

impl GmailEmailService {
    async fn do_send(
        config_service: Arc<ConfigService>,
        req: SendEmailReq,
    ) -> Result<(), AppError> {
        // 加载Gmail配置
        let config = config_service.load_config::<GmailConfig>().await?;
        
        let smtp_host = config.smtp_host
            .unwrap_or_else(|| "smtp.gmail.com".to_string());
        let username = config.gmail_username
            .ok_or_else(|| AppError::unknown("error.gmail_username_not_configured"))?;
        let password = config.gmail_password
            .ok_or_else(|| AppError::unknown("error.gmail_password_not_configured"))?;
        
        info!("Gmail [{}] send to [{}] subject: {}", username, req.to_address, req.subject);
        
        // 构建邮件
        let email = Message::builder()
            .from(username.parse().map_err(|e| {
                AppError::unknown_with_params(
                    "error.invalid_email_address",
                    serde_json::json!({"msg": format!("{}", e)})
                )
            })?)
            .to(req.to_address.parse().map_err(|e| {
                AppError::unknown_with_params(
                    "error.invalid_email_address",
                    serde_json::json!({"msg": format!("{}", e)})
                )
            })?)
            .subject(&req.subject)
            .header(ContentType::TEXT_HTML)
            .body(req.html_body)
            .map_err(|e| {
                AppError::unknown_with_params(
                    "error.build_email_failed",
                    serde_json::json!({"msg": format!("{}", e)})
                )
            })?;
        
        // 创建SMTP传输
        let creds = Credentials::new(username, password);
        let mailer = SmtpTransport::starttls_relay(&smtp_host)
            .map_err(|e| {
                AppError::unknown_with_params(
                    "error.smtp_connection_failed",
                    serde_json::json!({"msg": format!("{}", e)})
                )
            })?
            .credentials(creds)
            .build();
        
        // 发送邮件
        mailer.send(&email)
            .map_err(|e| {
                error!("SMTP发送失败: {:?}", e);
                AppError::unknown_with_params(
                    "error.send_email_failed",
                    serde_json::json!({"msg": format!("{}", e)})
                )
            })?;
        
        info!("邮件发送成功: to={}, subject={}", req.to_address, req.subject);
        Ok(())
    }
}
