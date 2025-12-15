use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// 雪花算法 ID 生成器
pub struct SnowflakeGenerator {
    /// 工作机器 ID (0-31)
    worker_id: i64,
    /// 数据中心 ID (0-31)
    datacenter_id: i64,
    /// 序列号 (0-4095)
    sequence: Mutex<i64>,
    /// 上次时间戳
    last_timestamp: Mutex<i64>,
}

const EPOCH: i64 = 1288834974657; // Twitter epoch
const WORKER_ID_BITS: i64 = 5;
const DATACENTER_ID_BITS: i64 = 5;
const SEQUENCE_BITS: i64 = 12;

const MAX_WORKER_ID: i64 = (1 << WORKER_ID_BITS) - 1;
const MAX_DATACENTER_ID: i64 = (1 << DATACENTER_ID_BITS) - 1;
const SEQUENCE_MASK: i64 = (1 << SEQUENCE_BITS) - 1;

const WORKER_ID_SHIFT: i64 = SEQUENCE_BITS;
const DATACENTER_ID_SHIFT: i64 = SEQUENCE_BITS + WORKER_ID_BITS;
const TIMESTAMP_SHIFT: i64 = SEQUENCE_BITS + WORKER_ID_BITS + DATACENTER_ID_BITS;

impl SnowflakeGenerator {
    /// 创建新的雪花算法生成器
    pub fn new(worker_id: i64, datacenter_id: i64) -> Self {
        assert!(worker_id <= MAX_WORKER_ID && worker_id >= 0, "worker_id 超出范围");
        assert!(datacenter_id <= MAX_DATACENTER_ID && datacenter_id >= 0, "datacenter_id 超出范围");
        
        Self {
            worker_id,
            datacenter_id,
            sequence: Mutex::new(0),
            last_timestamp: Mutex::new(-1),
        }
    }

    /// 生成下一个 ID
    pub fn next_id(&self) -> i64 {
        let mut sequence = self.sequence.lock().unwrap();
        let mut last_timestamp = self.last_timestamp.lock().unwrap();
        
        let mut timestamp = Self::current_millis();
        
        if timestamp < *last_timestamp {
            panic!("时钟回拨，拒绝生成 ID");
        }
        
        if timestamp == *last_timestamp {
            *sequence = (*sequence + 1) & SEQUENCE_MASK;
            if *sequence == 0 {
                timestamp = Self::wait_next_millis(*last_timestamp);
            }
        } else {
            *sequence = 0;
        }
        
        *last_timestamp = timestamp;
        
        ((timestamp - EPOCH) << TIMESTAMP_SHIFT)
            | (self.datacenter_id << DATACENTER_ID_SHIFT)
            | (self.worker_id << WORKER_ID_SHIFT)
            | *sequence
    }
    
    fn current_millis() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }
    
    fn wait_next_millis(last_timestamp: i64) -> i64 {
        let mut timestamp = Self::current_millis();
        while timestamp <= last_timestamp {
            timestamp = Self::current_millis();
        }
        timestamp
    }
}

// 全局雪花算法生成器
lazy_static::lazy_static! {
    static ref SNOWFLAKE: SnowflakeGenerator = SnowflakeGenerator::new(1, 1);
}

/// 生成雪花算法 ID
pub fn generate_id() -> i64 {
    SNOWFLAKE.next_id()
}

/// 生成雪花算法 ID 字符串
pub fn generate_id_string() -> String {
    SNOWFLAKE.next_id().to_string()
}
