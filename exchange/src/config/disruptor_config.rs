use common::config::app_config::DisruptorConfig;
use disruptor::{BusySpin, BusySpinWithSpinLoopHint, Producer};
use once_cell::sync::OnceCell;
use std::sync::Mutex;
use crate::consumer::disruptor_consumer::{UnifiedTickerEvent, handle_ticker_event};

/// 将 buffer_size 调整为最接近的 2 的幂次方
/// Disruptor 要求 buffer_size 必须是 2 的幂次方
fn adjust_buffer_size(size: usize) -> usize {
    if size == 0 {
        return 1;
    }
    if (size & (size - 1)) == 0 {
        // 已经是 2 的幂次方
        return size;
    }
    // 找到大于等于 size 的最小 2 的幂次方
    let mut power = 1;
    while power < size {
        power <<= 1;
    }
    power
}

/// Producer 类型标记
static IS_MULTI: OnceCell<bool> = OnceCell::new();

/// Single Producer 存储
type SingleProducerType = disruptor::SingleProducer<UnifiedTickerEvent, disruptor::SingleConsumerBarrier>;
static PRODUCER_SINGLE: OnceCell<Mutex<Option<Box<SingleProducerType>>>> = OnceCell::new();

/// Multi Producer 存储  
type MultiProducerType = disruptor::MultiProducer<UnifiedTickerEvent, disruptor::SingleConsumerBarrier>;
static PRODUCER_MULTI: OnceCell<Mutex<Option<Box<MultiProducerType>>>> = OnceCell::new();

/// 发布事件到 Disruptor
pub fn publish<F>(update: F)
where
    F: FnOnce(&mut UnifiedTickerEvent),
{
    if let Some(&is_multi) = IS_MULTI.get() {
        if is_multi {
            if let Some(producer) = PRODUCER_MULTI.get() {
                if let Ok(mut p) = producer.lock() {
                    if let Some(ref mut producer) = *p {
                        producer.publish(update);
                    }
                }
            }
        } else {
            if let Some(producer) = PRODUCER_SINGLE.get() {
                if let Ok(mut p) = producer.lock() {
                    if let Some(ref mut producer) = *p {
                        producer.publish(update);
                    }
                }
            }
        }
    }
}

/// 全局 Disruptor Producer 单例（兼容旧代码）
pub static PRODUCER: OnceCell<Mutex<()>> = OnceCell::new();

/// 初始化 Disruptor
/// 此函数在 main.rs 中被调用
pub fn init(config: &DisruptorConfig) {
    if !config.enabled {
        log::info!("Disruptor 未启用");
        return;
    }

    // 调整 buffer_size 为 2 的幂次方
    let buffer_size = adjust_buffer_size(config.buffer_size);
    if buffer_size != config.buffer_size {
        log::warn!("Disruptor buffer_size {} 不是 2 的幂次方，已调整为 {}", config.buffer_size, buffer_size);
    }

    log::info!("正在初始化 Disruptor: Size={}, Strategy={}, Type={}", 
        buffer_size, config.wait_strategy, config.consumer_type);

    let factory = || UnifiedTickerEvent::default();
    
    // 根据配置构建 Disruptor Producer
    match (config.consumer_type.as_str(), config.wait_strategy.as_str()) {
        ("Multi", "BusySpinWithSpinLoopHint") => {
            let producer = disruptor::build_multi_producer(buffer_size, factory, BusySpinWithSpinLoopHint)
                .handle_events_with(handle_ticker_event)
                .build();
            if PRODUCER_MULTI.set(Mutex::new(Some(Box::new(producer)))).is_err() {
                log::error!("Disruptor Multi Producer 只能初始化一次!");
                return;
            }
            if IS_MULTI.set(true).is_err() {
                log::error!("IS_MULTI 只能设置一次!");
                return;
            }
        }
        ("Multi", _) => {
            let producer = disruptor::build_multi_producer(buffer_size, factory, BusySpin)
                .handle_events_with(handle_ticker_event)
                .build();
            if PRODUCER_MULTI.set(Mutex::new(Some(Box::new(producer)))).is_err() {
                log::error!("Disruptor Multi Producer 只能初始化一次!");
                return;
            }
            if IS_MULTI.set(true).is_err() {
                log::error!("IS_MULTI 只能设置一次!");
                return;
            }
        }
        ("Single", "BusySpinWithSpinLoopHint") => {
            let producer = disruptor::build_single_producer(buffer_size, factory, BusySpinWithSpinLoopHint)
                .handle_events_with(handle_ticker_event)
                .build();
            if PRODUCER_SINGLE.set(Mutex::new(Some(Box::new(producer)))).is_err() {
                log::error!("Disruptor Single Producer 只能初始化一次!");
                return;
            }
            if IS_MULTI.set(false).is_err() {
                log::error!("IS_MULTI 只能设置一次!");
                return;
            }
        }
        _ => {
            let producer = disruptor::build_single_producer(buffer_size, factory, BusySpin)
                .handle_events_with(handle_ticker_event) //设置消费者
                .build();
            if PRODUCER_SINGLE.set(Mutex::new(Some(Box::new(producer)))).is_err() {
                log::error!("Disruptor Single Producer 只能初始化一次!");
                return;
            }
            if IS_MULTI.set(false).is_err() {
                log::error!("IS_MULTI 只能设置一次!");
                return;
            }
        }
    }

    // 初始化兼容的 PRODUCER（用于 ticker_consumer.rs）
    PRODUCER.set(Mutex::new(())).unwrap();

    log::info!("✅ Disruptor Ticker 处理引擎启动成功");
}
