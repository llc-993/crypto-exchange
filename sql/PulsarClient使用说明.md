# PulsarClient 使用说明

## 目录
- [概述](#概述)
- [初始化](#初始化)
- [发布消息](#发布消息)
- [订阅消息](#订阅消息)
- [Topic 定义](#topic-定义)
- [完整示例](#完整示例)
- [注意事项](#注意事项)

## 概述

`PulsarClient` 是封装了 Apache Pulsar 客户端的工具类，提供了全局单例模式，支持消息的发布和订阅。主要特性：

- ✅ 全局单例，自动管理 Producer
- ✅ 支持异步非阻塞发布（可在非 Tokio 线程中调用）
- ✅ 支持同步阻塞发布（等待确认）
- ✅ 支持延时消息
- ✅ 自动序列化/反序列化 JSON
- ✅ 支持 Shared 订阅模式

## 初始化

### 1. 在 main 函数中初始化

```rust
use common::PulsarClient;

#[tokio::main]
async fn main() {
    // 从配置读取 Pulsar URL
    let pulsar_url = "pulsar://localhost:6650";
    
    // 初始化全局 PulsarClient
    if let Err(e) = PulsarClient::init_global(pulsar_url).await {
        log::error!("❌ Pulsar 初始化失败: {}", e);
        return;
    }
    
    log::info!("✅ Pulsar 客户端连接成功");
}
```

### 2. 从配置文件初始化

```rust
use common::AppConfig;

let config = AppConfig::from_file_or_embedded(...).expect("配置加载失败");

if config.pulsar.enabled {
    if let Err(e) = PulsarClient::init_global(&config.pulsar.url).await {
        log::error!("❌ Pulsar 初始化失败: {}", e);
    } else {
        log::info!("✅ Pulsar 客户端连接成功: {}", config.pulsar.url);
    }
}
```

## 发布消息

### 1. 异步非阻塞发布（推荐）

**适用场景**：在 Tokio runtime 线程或非 Tokio 线程（如 disruptor 处理器线程）中调用

```rust
use common::PulsarClient;
use common::pulsar::topics;
use common::models::UnifiedTicker;

// 方式1：使用静态方法（推荐）
PulsarClient::publish_async(topics::ticker::SPOT_TICKER, unified_ticker);

// 方式2：使用实例方法
let client = PulsarClient::global().unwrap();
client.send("my-topic", &my_data).await?;
```

**特点**：
- 不阻塞当前线程
- 自动在后台异步执行
- 支持在非 Tokio 线程中调用

### 2. 同步阻塞发布

**适用场景**：需要等待消息发送确认

```rust
use common::PulsarClient;

// 方式1：使用静态方法
PulsarClient::publish("my-topic", &my_data).await;

// 方式2：使用实例方法（等待确认）
let client = PulsarClient::global().unwrap();
client.send_blocking("my-topic", &my_data).await?;
```

**特点**：
- 阻塞等待发送完成
- 返回发送结果

### 3. 延时消息发布

```rust
use common::PulsarClient;

let client = PulsarClient::global().unwrap();

// 发送延时消息，10秒后投递
client.send_delay("my-topic", &my_data, 10).await?;
```

### 实际使用示例

#### 示例1：发布 Ticker 数据（input 服务）

```rust
use common::PulsarClient;
use common::pulsar::topics;
use common::models::UnifiedTicker;

// 从交易所获取数据后，转换为 UnifiedTicker
match common::TickerConverter::from_binance_spot(&json_msg, symbol) {
    Ok(unified_ticker) => {
        // 异步发布到 Pulsar
        PulsarClient::publish_async(topics::ticker::SPOT_TICKER, unified_ticker);
    }
    Err(e) => log::error!("Ticker 转换失败: {}", e),
}
```

#### 示例2：发布价格数据（exchange 服务）

```rust
use common::PulsarClient;
use common::pulsar::topics;
use common::models::coin_thumb_price::CoinThumbPrice;

fn handle_price_change(symbol: &str, price: Decimal, market_type: MarketType) {
    // 生成交易数量
    let volume = exponential_random_decimal(price.clone());
    let thumb_price = CoinThumbPrice::new(
        symbol.to_string(), 
        price, 
        volume, 
        market_type
    );

    // 异步发布到 Pulsar
    PulsarClient::publish_async(topics::ticker::EX_THUMB_PRICE, thumb_price);
}
```

## 订阅消息

### 1. 基本订阅流程

```rust
use common::PulsarClient;
use common::pulsar::topics;
use common::models::UnifiedTicker;
use futures::StreamExt;
use pulsar::Consumer;

pub async fn start_ticker_consumer() {
    let Some(client) = PulsarClient::global() else {
        log::error!("PulsarClient 未初始化，无法启动消费者");
        return;
    };

    // 订阅 Topic
    match client.subscribe::<UnifiedTicker>(
        topics::ticker::SPOT_TICKER, 
        "my-subscription-group"
    ).await {
        Ok(consumer) => {
            log::info!("✅ 消费者已启动");
            tokio::spawn(consume_loop(consumer));
        }
        Err(e) => log::error!("订阅失败: {}", e),
    }
}

async fn consume_loop(mut consumer: Consumer<UnifiedTicker, pulsar::TokioExecutor>) {
    while let Some(msg) = consumer.next().await {
        let Ok(msg) = msg else {
            log::error!("接收消息失败: {:?}", msg.err());
            continue;
        };

        match msg.deserialize() {
            Ok(data) => {
                // 处理消息
                log::debug!("收到消息: {:?}", data);
                
                // 确认消息
                if let Err(e) = consumer.ack(&msg).await {
                    log::error!("Ack 失败: {}", e);
                }
            }
            Err(e) => log::error!("反序列化失败: {}", e),
        }
    }
    log::warn!("消费者循环退出");
}
```

### 2. 实际使用示例（market 服务）

```rust
use common::models::coin_thumb_price::CoinThumbPrice;
use common::pulsar::topics;
use common::PulsarClient;
use futures::StreamExt;
use pulsar::Consumer;

pub async fn start_thumb_price_consumer() {
    let Some(client) = PulsarClient::global() else {
        log::error!("PulsarClient 未初始化，无法启动消费者");
        return;
    };

    match client.subscribe::<CoinThumbPrice>(
        topics::ticker::EX_THUMB_PRICE, 
        "ex-thumb-price-group"
    ).await {
        Ok(consumer) => {
            log::info!("✅ 价格消费者已启动");
            tokio::spawn(consume_thumb_price_loop(consumer));
        }
        Err(e) => log::error!("订阅价格失败: {}", e),
    }
}

async fn consume_thumb_price_loop(mut consumer: Consumer<CoinThumbPrice, pulsar::TokioExecutor>) {
    while let Some(msg) = consumer.next().await {
        let Ok(msg) = msg else {
            log::error!("[价格] 接收消息失败: {:?}", msg.err());
            continue;
        };

        match msg.deserialize() {
            Ok(thumb_price) => {
                // 处理价格数据
                log::debug!("[价格] 收到: {} {:?} 价格: {}",
                    thumb_price.symbol, 
                    thumb_price.market_type, 
                    thumb_price.price
                );

                // 业务处理...
                // generate_kline(&thumb_price).await;
                // generate_thumb_stats(&thumb_price).await;

                // 确认消息
                if let Err(e) = consumer.ack(&msg).await {
                    log::error!("[价格] Ack 失败: {}", e);
                }
            }
            Err(e) => log::error!("[价格] 反序列化失败: {}", e),
        }
    }
    log::warn!("价格消费者循环退出");
}
```

## Topic 定义

所有 Topic 常量定义在 `common/src/pulsar/topics.rs` 中：

### Ticker 相关 Topics

```rust
use common::pulsar::topics;

// 现货 Ticker 数据
topics::ticker::SPOT_TICKER        // "spot-ticker"

// 合约 Ticker 数据
topics::ticker::FUTURES_TICKER    // "futures-ticker"

// 撮合引擎推送的交易数据
topics::ticker::EX_THUMB_PRICE    // "ex-thumb-price"
```

### Mark Price 相关 Topics

```rust
// 合约标记价格数据
topics::mark_price::FUTURES_MARK_PRICE  // "futures-mark-price"
```

### Funding Rate Topics

```rust
// Futures Funding Rate
topics::funding_rate::FUTURES_FUNDING_RATE  // "futures-funding-rate"
```

### Kline 相关 Topics

```rust
// 现货 K线数据
topics::kline::SPOT_KLINE          // "spot-kline"

// 合约 K线数据
topics::kline::FUTURES_KLINE      // "futures-kline"
```

### Depth 相关 Topics

```rust
// 现货深度数据
topics::depth::SPOT_DEPTH          // "spot-depth"

// 合约深度数据
topics::depth::FUTURES_DEPTH       // "futures-depth"
```

## 完整示例

### 示例1：完整的发布者（input 服务）

```rust
use common::PulsarClient;
use common::pulsar::topics;
use common::models::UnifiedTicker;

// 在 main 函数中初始化
#[tokio::main]
async fn main() {
    // 初始化 Pulsar
    if let Err(e) = PulsarClient::init_global("pulsar://localhost:6650").await {
        log::error!("Pulsar 初始化失败: {}", e);
        return;
    }
    
    // 业务逻辑：从交易所获取数据
    // ...
    
    // 转换为 UnifiedTicker
    let unified_ticker = UnifiedTicker {
        exchange: "binance".to_string(),
        symbol: "BTCUSDT".to_string(),
        market_type: MarketType::Spot,
        close: Decimal::from(50000),
        // ... 其他字段
    };
    
    // 发布到 Pulsar
    PulsarClient::publish_async(topics::ticker::SPOT_TICKER, unified_ticker);
}
```

### 示例2：完整的消费者（market 服务）

```rust
use common::models::coin_thumb_price::CoinThumbPrice;
use common::pulsar::topics;
use common::PulsarClient;
use futures::StreamExt;
use pulsar::Consumer;

// 在 main 函数中初始化
#[tokio::main]
async fn main() {
    // 初始化 Pulsar
    if let Err(e) = PulsarClient::init_global("pulsar://localhost:6650").await {
        log::error!("Pulsar 初始化失败: {}", e);
        return;
    }
    
    // 启动消费者
    start_thumb_price_consumer().await;
    
    // 保持服务运行
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl-c signal");
}

pub async fn start_thumb_price_consumer() {
    let Some(client) = PulsarClient::global() else {
        log::error!("PulsarClient 未初始化");
        return;
    };

    match client.subscribe::<CoinThumbPrice>(
        topics::ticker::EX_THUMB_PRICE, 
        "ex-thumb-price-group"
    ).await {
        Ok(consumer) => {
            tokio::spawn(consume_loop(consumer));
        }
        Err(e) => log::error!("订阅失败: {}", e),
    }
}

async fn consume_loop(mut consumer: Consumer<CoinThumbPrice, pulsar::TokioExecutor>) {
    while let Some(msg) = consumer.next().await {
        let Ok(msg) = msg else {
            continue;
        };

        match msg.deserialize() {
            Ok(thumb_price) => {
                // 处理业务逻辑
                process_thumb_price(&thumb_price).await;
                
                // 确认消息
                if let Err(e) = consumer.ack(&msg).await {
                    log::error!("Ack 失败: {}", e);
                }
            }
            Err(e) => log::error!("反序列化失败: {}", e),
        }
    }
}
```

## 注意事项

### 1. 数据模型要求

所有要发布到 Pulsar 的数据结构必须：
- 实现 `Serialize` 和 `Deserialize` trait
- 实现 `Send + Sync + 'static`
- 实现 `pulsar::DeserializeMessage` trait（用于订阅）

```rust
use serde::{Deserialize, Serialize};
use pulsar::DeserializeMessage;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MyData {
    pub field: String,
}

impl DeserializeMessage for MyData {
    type Output = Result<MyData, serde_json::Error>;

    fn deserialize_message(payload: &pulsar::Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}
```

### 2. 订阅模式

当前使用 `Shared` 订阅模式，多个消费者可以共享同一个订阅组，消息会在消费者之间负载均衡。

### 3. 消息确认（ACK）

**重要**：处理完消息后必须调用 `consumer.ack(&msg).await`，否则消息会被重新投递。

```rust
// ✅ 正确：处理完消息后确认
match msg.deserialize() {
    Ok(data) => {
        process_data(&data).await;
        consumer.ack(&msg).await?;  // 必须确认
    }
    Err(e) => log::error!("反序列化失败: {}", e),
}

// ❌ 错误：忘记确认会导致消息重复投递
match msg.deserialize() {
    Ok(data) => {
        process_data(&data).await;
        // 缺少 ack！
    }
    _ => {}
}
```

### 4. 错误处理

- 发布失败：会记录错误日志，但不会抛出异常（`publish_async`）
- 订阅失败：需要检查返回值并处理错误
- 反序列化失败：应该记录错误并继续处理下一条消息

### 5. 线程安全

- `publish_async` 可以在任何线程中调用（包括非 Tokio 线程）
- `publish` 和 `subscribe` 必须在 Tokio runtime 中调用
- 全局单例是线程安全的

### 6. Producer 管理

- Producer 按 topic 自动创建和管理
- 每个 topic 只有一个 Producer 实例
- Producer 在首次发送消息时自动创建

### 7. 性能优化建议

1. **使用 `publish_async`**：在不需要等待确认的场景下，使用异步发布
2. **批量处理**：在消费者中批量处理消息以提高吞吐量
3. **合理设置订阅组**：不同服务使用不同的订阅组名称
4. **及时 ACK**：处理完消息后立即确认，避免重复投递

## 常见问题

### Q1: 为什么消息发送失败但没有报错？

A: `publish_async` 是异步非阻塞的，错误会记录在日志中。如果需要确认发送结果，使用 `publish_blocking` 或 `send_blocking`。

### Q2: 如何确保消息顺序？

A: 当前实现使用 `Shared` 订阅模式，不保证顺序。如果需要顺序消费，可以使用 `Exclusive` 订阅模式（需要修改代码）。

### Q3: 如何处理消息积压？

A: 
1. 增加消费者实例数量
2. 优化消息处理逻辑，提高处理速度
3. 使用批量处理
4. 检查是否有消息未及时 ACK

### Q4: 如何监控 Pulsar 消息？

A: 可以通过 Pulsar 管理界面或使用 Pulsar 的监控 API 查看 topic 的消息数量、消费延迟等信息。

## 相关文件

- `common/src/pulsar/client.rs` - PulsarClient 实现
- `common/src/pulsar/topics.rs` - Topic 常量定义
- `input/src/main.rs` - 发布者示例
- `market/src/consumer/thumb_price_consumer.rs` - 消费者示例
- `exchange/src/consumer/ticker_consumer.rs` - 消费者示例

