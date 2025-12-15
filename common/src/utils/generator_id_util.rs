// 根据ID生成编号
const SEED: [char; 35] = [
    'E', '5', 'F', 'C', 'D', 'G', '3', 'H', 'Q', 'A', '4', 'B', '1', 'N', 'O',
    'P', 'I', 'J', '2', 'R', 'S', 'T', 'U', 'V', '6', '7', 'M', 'W', 'X', '8',
    '9', 'K', 'L', 'Y', 'Z',
];

/// 根据id生成编号
pub fn generate_for_id(id: i64) -> String {
    let mut num = id + 10000;
    let mut code = String::new();
    
    while num > 0 {
        let mod_val = num % 35;
        num = (num - mod_val) / 35;
        code.insert(0, SEED[mod_val as usize]);
    }
    
    while code.len() < 4 {
        code.insert(0, '0');
    }
    
    code
}

/// ID配置，定义各部分的位数
#[derive(Debug, Clone)]
pub struct IdConfig {
    pub user_group_bits: i32,
    pub top_id_bits: i32,
    pub t2_id_bits: i32,
}

impl IdConfig {
    pub fn user_id_bits(&self) -> i32 {
        64 - self.user_group_bits - self.top_id_bits - self.t2_id_bits
    }
}

/// 默认配置：userGroup:2位, topId:16位, t2Id:16位, userId:30位
pub const CURRENT_ID_CONFIG: IdConfig = IdConfig {
    user_group_bits: 2,
    top_id_bits: 16,
    t2_id_bits: 16,
};

/// 生成 UID
/// 
/// # Arguments
/// * `user_id` - 用户ID
/// * `user_group` - 1 表示母账户 2表示子账户
/// * `top_id` - 顶级代理ID (t1_id)
/// * `t2_id` - 二级代理ID
/// 
/// # Returns
/// 生成的UID (64位整数)
pub fn generate_uid(
    user_id: i64,
    user_group: i64,
    top_id: i64,
    t2_id: i64,
) -> i64 {
    let config = &CURRENT_ID_CONFIG;
    let user_group = user_group.max(0);
    let top_id = top_id.max(0);
    let t2_id = t2_id.max(0);
    
    // 验证范围
    assert!(
        user_group < (1i64 << config.user_group_bits),
        "userGroup too large"
    );
    assert!(
        top_id < (1i64 << config.top_id_bits),
        "topId too large"
    );
    assert!(
        t2_id < (1i64 << config.t2_id_bits),
        "t2Id too large"
    );
    assert!(
        user_id < (1i64 << config.user_id_bits()),
        "userId too large"
    );
    
    // 组合各部分
    (user_group << (config.top_id_bits + config.t2_id_bits + config.user_id_bits()))
        | (top_id << (config.t2_id_bits + config.user_id_bits()))
        | (t2_id << config.user_id_bits())
        | user_id
}

/// 从UID解析用户ID
pub fn parse_user_id(uid: i64) -> i64 {
    let config = &CURRENT_ID_CONFIG;
    uid & ((1i64 << config.user_id_bits()) - 1)
}

/// 用户ID解析结果
#[derive(Debug, Clone)]
pub struct UserIdVO {
    pub user_id: i64,
    pub user_group: i64,
    pub top_id: i64,
    pub t2_id: i64,
}

/// 解析UID为各个组成部分
pub fn parse_user_id_to_vo(uid: i64) -> UserIdVO {
    let config = &CURRENT_ID_CONFIG;
    
    let user_id_mask = (1i64 << config.user_id_bits()) - 1;
    let t2_id_mask = (1i64 << config.t2_id_bits) - 1;
    let top_id_mask = (1i64 << config.top_id_bits) - 1;
    let user_group_mask = (1i64 << config.user_group_bits) - 1;
    
    let user_id = uid & user_id_mask;
    let t2_id = (uid >> config.user_id_bits()) & t2_id_mask;
    let top_id = (uid >> (config.user_id_bits() + config.t2_id_bits)) & top_id_mask;
    let user_group = (uid >> (config.user_id_bits() + config.t2_id_bits + config.top_id_bits)) & user_group_mask;
    
    UserIdVO {
        user_id,
        user_group,
        top_id,
        t2_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_for_id() {
        let code = generate_for_id(10000);
        println!("Generated code for id 10000: {}", code);
        assert!(!code.is_empty());
        assert!(code.len() >= 4);
    }
    
    #[test]
    fn test_generate_uid() {
        // 测试用例：userId=100001, userGroup=0, topId=10002, t2Id=0
        let uid = generate_uid(100001, 0, 10002, 0);
        println!("Generated UID: {}", uid);
        
        // 解析回去验证
        let vo = parse_user_id_to_vo(uid);
        assert_eq!(vo.user_id, 100001);
        assert_eq!(vo.user_group, 0);
        assert_eq!(vo.top_id, 10002);
        assert_eq!(vo.t2_id, 0);
        
        // 测试 parse_user_id
        let parsed_user_id = parse_user_id(uid);
        assert_eq!(parsed_user_id, 100001);
    }
}
