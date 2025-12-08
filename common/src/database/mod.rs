// 数据库连接模块
// 
// 此模块重新导出配置模块中的数据库功能
// 主要功能已移至 config::db_conf 模块

pub use crate::config::db_conf::{
    DbConfig,
    init_db,
    get_db,
    test_connection,
    get_pool_status,
};