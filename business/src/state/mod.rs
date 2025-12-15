use std::sync::Arc;
use rbatis::RBatis;
use common::mq::message_queue::MessageQueue;
use common::services::config_service::ConfigService;
use common::services::email::EmailServiceSupport;
use common::services::emqx_service::EmqxService;
use common::services::ip_service::IpService;
use common::services::sms::SmsServiceSupport;
use common::services::upload::UploadServiceSupport;
use common::utils::redis_util::RedisUtil;
use crate::service::agent_relation_service::AgentRelationService;

#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub rb: Arc<RBatis>,
    pub redis: Arc<RedisUtil>,
    pub config_service: Arc<ConfigService>,
    pub ip_service: Arc<IpService>,
    pub upload_service: Arc<UploadServiceSupport>,
    pub email_service: Arc<EmailServiceSupport>,
    pub emqx_service: Arc<EmqxService>,
    pub sms_service: Arc<SmsServiceSupport>,
    pub mq: Arc<MessageQueue>,
    pub agent_relation_service: Arc<AgentRelationService>,
}