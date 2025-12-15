use std::sync::Arc;
use rbatis::RBatis;
use crate::state::AppState;
use common::utils::generator_id_util;
use orm::entities::agent::AppAgentRelation;
use orm::entities::user::AppUser;
use rbatis::rbdc::datetime::DateTime;
use common::AppError;

pub struct AgentRelationService {
    rb: Arc<RBatis>,
}

impl AgentRelationService {

    pub fn new(rb: Arc<RBatis>) -> Self { Self{rb} }

    /// 创建用户代理关系
    pub async fn create_app_user_agent_relation(
        &self,
        user: &AppUser,
        owner: Option<&AppAgentRelation>,
    ) -> Result<AppAgentRelation, AppError> {
        let user_id = user.id.ok_or(AppError::business("error.user_not_found"))?;
        let account = user.user_account.clone().ok_or(AppError::business("error.user_account_not_found"))?;
        
        // 生成邀请码
        let code = generator_id_util::generate_for_id(user_id);
        
        let ag = AppAgentRelation {
            id: None,
            t1_id: owner.map(|o| o.t1_id).unwrap_or(user_id),
            t2_id: match owner.map(|o| o.level) {
                // 如果他的上级是总代,(他自己是一级代理)那么二级代理id为-1(表示没有)
                Some(0) => -1,
                // 如果他的上级是一级代理,那么二级代理id就是他本身
                Some(1) => user_id,
                // 其他情况, 上级是二级代理或者二级代理的下线.那么二级代理id继承上级的
                _ => owner.map(|o| o.t2_id).unwrap_or(-1),
            },
            t3_id: match owner.map(|o| o.level) {
                // 如果他的上级是总代或者一级,那么三级代理ID为-1(表示没有)
                Some(0) | Some(1) => Some(-1),
                // 如果他的上级是二级代理,那么三级代理ID就是他本身
                Some(2) => Some(user_id),
                // 其他情况,上级是三级代理或者三级代理的下线.那么二级代理id继承上级的
                _ => owner.and_then(|o| o.t3_id),
            },
            top_share_code: owner.map(|o| o.top_share_code.clone()).unwrap_or(code.clone()),
            ori_user_id: user_id,
            ori_share_code: code,
            ori_account: account.clone(),
            // 级别 : (0)-总代 (1)-一级代理 (2)-二级代理 (3-无限)-会员
            level: owner.map(|o| o.level + 1).unwrap_or(1),
            user_group: user.user_group.unwrap_or(0),
            
            // 1级代理（直属上级）
            p1_id: owner.map(|o| o.ori_user_id).unwrap_or(-1),
            p1_code: owner.map(|o| o.ori_share_code.clone()).unwrap_or_default(),
            p1_account: owner.map(|o| o.ori_account.clone()).unwrap_or_default(),
            
            p2_id: owner.map(|o| o.p1_id).unwrap_or(-1),
            p2_code: owner.map(|o| o.p1_code.clone()).unwrap_or_default(),
            p2_account: owner.map(|o| o.p1_account.clone()).unwrap_or_default(),
            
            p3_id: owner.map(|o| o.p2_id).unwrap_or(-1),
            p3_code: owner.map(|o| o.p2_code.clone()).unwrap_or_default(),
            p3_account: owner.map(|o| o.p2_account.clone()).unwrap_or_default(),
            
            p4_id: owner.map(|o| Some(o.p3_id)).flatten(),
            p5_id: owner.and_then(|o| o.p4_id),
            p6_id: owner.and_then(|o| o.p5_id),
            p7_id: owner.and_then(|o| o.p6_id),
            p8_id: owner.and_then(|o| o.p7_id),
            p9_id: owner.and_then(|o| o.p8_id),
            
            create_by: Some(owner.map(|o| o.ori_account.clone()).unwrap_or_else(|| "root".to_string())),
            create_time: Some(DateTime::now()),
            update_by: Some(owner.map(|o| o.ori_account.clone()).unwrap_or_else(|| "root".to_string())),
            update_time: Some(DateTime::now()),
        };
        
        // 保存到数据库
        AppAgentRelation::insert(self.rb.as_ref(), &ag).await
            .map_err(|_| AppError::unknown("error.db_error"))?;
        
        Ok(ag)
    }
}