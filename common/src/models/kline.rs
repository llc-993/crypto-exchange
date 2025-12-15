use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use bson::{doc, Document};
use crate::{KlineInterval, MarketType};
use crate::models::coin_thumb_price::CoinThumbPrice;
use crate::mongodb::MongoDBClient;
use crate::redis::cache;
use crate::redis::keys::mongodb::COLLECTION_EXISTS;

/// K线数据
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Kline {
    /// 交易对
    pub symbol: String,

    /// 市场类型
    pub market_type: MarketType,

    /// K线时间间隔
    pub interval: KlineInterval,

    /// K线开始时间戳（秒）
    pub open_time: i64,

    /// K线结束时间戳（秒）
    pub close_time: i64,

    /// 开盘价
    pub open: Decimal,

    /// 最高价
    pub high: Decimal,

    /// 最低价
    pub low: Decimal,

    /// 收盘价
    pub close: Decimal,

    /// 成交量（可选）
    pub volume: Option<Decimal>,

    /// 成交额（可选）
    pub quote_volume: Option<Decimal>,

    /// K线是否已完成（时间窗口已关闭）
    pub is_closed: bool,
}

impl Kline {
    /// 创建新的K线
    pub fn new(
        symbol: String,
        market_type: MarketType,
        interval: KlineInterval,
        open_time: i64,
        close_time: i64,
        price: Decimal,
    ) -> Self {
        Self {
            symbol,
            market_type,
            interval,
            open_time,
            close_time,
            open: price,
            high: price,
            low: price,
            close: price,
            volume: None,
            quote_volume: None,
            is_closed: false,
        }
    }

    /// 更新价格（更新最高价、最低价、收盘价）
    pub fn update_price(&mut self, thumb_price: CoinThumbPrice) {
        if thumb_price.price > self.high {
            self.high = thumb_price.price;
        }
        if thumb_price.price < self.low {
            self.low = thumb_price.price;
        }
        self.close = thumb_price.price;
        // 累加成交量
        self.volume = Some(thumb_price.volume + self.volume.unwrap_or(Decimal::ZERO));
        // 计算成交额 = (成交量 * 当前价格) + 之前的
        self.quote_volume = Some(
            self.quote_volume.unwrap_or(Decimal::ZERO) + (thumb_price.price * thumb_price.volume)
        )
    }

    /// 标记K线为已完成
    pub fn mark_closed(&mut self) {
        self.is_closed = true;
    }
}

impl pulsar::DeserializeMessage for Kline {
    type Output = Result<Kline, serde_json::Error>;

    fn deserialize_message(payload: &pulsar::Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}

// ==================== MongoDB 操作 ====================

impl Kline {
    /// 生成 MongoDB 集合名称
    /// 格式: klines_{market_type}_{interval}
    /// 例如: klines_Spot_M1, klines_Futures_H1
    pub fn collection_name(&self) -> String {
        format!("klines_{}_{:?}_{:?}", self.symbol, self.market_type, self.interval)
    }

    /// 根据 market_type 和 interval 生成集合名称（静态方法）
    pub fn get_collection_name(symbol: String, market_type: MarketType, interval: KlineInterval) -> String {
        format!("klines_{}_{:?}_{:?}", symbol, market_type, interval)
    }

    /// 生成唯一 ID（用于 MongoDB 文档标识）
    /// 格式: {symbol}_{market_type}_{interval}_{open_time}
    pub fn generate_id(&self) -> String {
        format!("{}", self.open_time)
    }

    /// 转换为 MongoDB 查询文档
    fn to_filter(&self) -> Document {
        doc! {
            "symbol": &self.symbol,
            "market_type": self.market_type.as_str(),
            "interval": format!("{:?}", self.interval),
            "open_time": self.open_time,
        }
    }

    // ==================== Create 操作 ====================

    /// 确保集合已初始化（检查 Redis 缓存 -> 检查 MongoDB -> 创建索引）
    ///
    /// # 参数
    /// * `collection_name` - 集合名称
    /// * `market_type` - 市场类型
    /// * `interval` - K线时间间隔
    async fn ensure_collection_initialized(
        collection_name: &str
    ) -> Result<(), mongodb::error::Error> {
        // 1. 检查 Redis hash 中是否存在该 collection_name
        match cache.hget(COLLECTION_EXISTS, collection_name).await {
            Ok(Some(_)) => {
                // Redis 中存在，说明集合已初始化
                return Ok(());
            }
            Ok(None) => {
                // Redis 中不存在，继续检查 MongoDB
            }
            Err(e) => {
                // Redis 操作失败，记录警告但继续执行
                log::warn!("检查 Redis 缓存失败: {}，继续检查 MongoDB", e);
            }
        }

        // 2. 检查 MongoDB 中集合是否存在
        let exists = MongoDBClient::collection_exists(collection_name).await?;

        if exists {
            // 3. MongoDB 中存在，写入 Redis hash
            if let Err(e) = cache.hset(COLLECTION_EXISTS, collection_name, "1").await {
                log::warn!("写入 Redis 缓存失败: {}", e);
            }
        } else {
            // 4. MongoDB 中不存在，创建索引
            log::info!("集合 {} 不存在，正在创建索引...", collection_name);
            Self::create_indexes(collection_name).await?;

            // 创建索引后，写入 Redis hash
            if let Err(e) = cache.hset(COLLECTION_EXISTS, collection_name, "1").await {
                log::warn!("写入 Redis 缓存失败: {}", e);
            }
        }

        Ok(())
    }

    /// 插入单个 Kline 到 MongoDB
    ///
    /// insert_one/insert_many
    //     ↓
    // 检查 Redis Hash (mongodb:collections)
    //     ↓
    // 存在？ → 是 → 直接插入数据
    //     ↓
    //    否
    //     ↓
    // 检查 MongoDB 集合是否存在
    //     ↓
    // 存在？ → 是 → 写入 Redis Hash → 插入数据
    //     ↓
    //    否
    //     ↓
    // 调用 create_indexes() 创建索引
    //     ↓
    // 写入 Redis Hash
    //     ↓
    // 插入数据
    ///
    /// # 示例
    /// ```ignore
    /// let kline = Kline::new(...);
    /// kline.insert_one().await?;
    /// ```
    pub async fn insert_one(&self) -> Result<mongodb::results::InsertOneResult, mongodb::error::Error> {
        let collection_name = self.collection_name();

        // 确保集合已初始化
        Self::ensure_collection_initialized(&collection_name.as_str()).await?;

        MongoDBClient::insert_one(&collection_name, self).await
    }

    /// 插入多个 Kline 到 MongoDB（支持 upsert：存在则更新，不存在则插入）
    /// 注意：所有 Kline 必须有相同的 market_type 和 interval
    ///
    /// # 示例
    /// ```ignore
    /// let klines = vec![kline1, kline2, kline3];
    /// Kline::insert_many(klines).await?;
    /// ```
    pub async fn insert_many(klines: Vec<Kline>) -> Result<(), mongodb::error::Error> {
        if klines.is_empty() {
            return Err(mongodb::error::Error::custom("Kline 列表不能为空"));
        }

        let first_kline = &klines[0];
        let collection_name = first_kline.collection_name();

        // 确保集合已初始化
        Self::ensure_collection_initialized(&collection_name).await?;

        let Some(client) = MongoDBClient::global() else {
            return Err(mongodb::error::Error::custom("MongoDBClient 未初始化"));
        };
        let collection = client.collection::<Document>(&collection_name);

        // 对每个 Kline 执行 upsert 操作
        for kline in &klines {
            kline.upsert_one().await?;
        }

        Ok(())
    }

    // ==================== Read 操作 ====================

    /// 根据条件查找单个 Kline
    ///
    /// # 参数
    /// * `symbol` - 交易对
    /// * `market_type` - 市场类型
    /// * `interval` - K线时间间隔
    /// * `open_time` - K线开始时间戳
    ///
    /// # 示例
    /// ```ignore
    /// let kline = Kline::find_one("BTCUSDT", MarketType::Spot, KlineInterval::M1, 1234567890).await?;
    /// ```
    pub async fn find_one(
        symbol: &str,
        market_type: MarketType,
        interval: KlineInterval,
        open_time: i64,
    ) -> Result<Option<Kline>, mongodb::error::Error> {
        let filter = doc! {
            "symbol": symbol,
            "market_type": market_type.as_str(),
            "interval": format!("{:?}", interval),
            "open_time": open_time,
        };
        let collection_name = Self::get_collection_name(symbol.to_string(), market_type, interval);
        MongoDBClient::find_one(&collection_name, filter).await
    }

    /// 根据交易对和时间范围查找多个 Kline
    ///
    /// # 参数
    /// * `symbol` - 交易对
    /// * `market_type` - 市场类型
    /// * `interval` - K线时间间隔
    /// * `start_time` - 开始时间戳（可选）
    /// * `end_time` - 结束时间戳（可选）
    /// * `limit` - 返回数量限制（可选）
    ///
    /// # 示例
    /// ```ignore
    /// let klines = Kline::find_many("BTCUSDT", MarketType::Spot, KlineInterval::M1, Some(1234567890), Some(1234567900), Some(100)).await?;
    /// ```
    pub async fn find_many(
        symbol: &str,
        market_type: MarketType,
        interval: KlineInterval,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<Kline>, mongodb::error::Error> {
        let mut filter = doc! {
            "symbol": symbol,
            "market_type": market_type.as_str(),
            "interval": format!("{:?}", interval),
        };

        // 添加时间范围过滤
        if start_time.is_some() || end_time.is_some() {
            let mut time_filter = Document::new();
            if let Some(start) = start_time {
                time_filter.insert("$gte", start);
            }
            if let Some(end) = end_time {
                time_filter.insert("$lte", end);
            }
            filter.insert("open_time", time_filter);
        }

        // 设置查询选项（排序和限制）
        let mut find_options = mongodb::options::FindOptions::default();
        find_options.sort = Some(doc! { "open_time": 1 }); // 按时间升序
        if let Some(limit_val) = limit {
            find_options.limit = Some(limit_val);
        }

        let collection_name = Self::get_collection_name(symbol.to_string(), market_type, interval);
        MongoDBClient::find_many(&collection_name, filter, Some(find_options)).await
    }

    /// 查找最新的 Kline（按时间倒序）
    ///
    /// # 参数
    /// * `symbol` - 交易对
    /// * `market_type` - 市场类型
    /// * `interval` - K线时间间隔
    /// * `limit` - 返回数量（默认 1）
    ///
    /// # 示例
    /// ```ignore
    /// let latest = Kline::find_latest("BTCUSDT", MarketType::Spot, KlineInterval::M1, Some(10)).await?;
    /// ```
    pub async fn find_latest(
        symbol: &str,
        market_type: MarketType,
        interval: KlineInterval,
        limit: Option<i64>,
    ) -> Result<Vec<Kline>, mongodb::error::Error> {
        let filter = doc! {
            "symbol": symbol,
            "market_type": market_type.as_str(),
            "interval": format!("{:?}", interval),
        };

        let mut find_options = mongodb::options::FindOptions::default();
        find_options.sort = Some(doc! { "open_time": -1 }); // 按时间倒序
        find_options.limit = Some(limit.unwrap_or(1));

        let collection_name = Self::get_collection_name(symbol.to_string(), market_type, interval);
        MongoDBClient::find_many(&collection_name, filter, Some(find_options)).await
    }

    /// 统计符合条件的 Kline 数量
    ///
    /// # 参数
    /// * `symbol` - 交易对（可选）
    /// * `market_type` - 市场类型（必需，用于确定集合）
    /// * `interval` - K线时间间隔（必需，用于确定集合）
    ///
    /// # 示例
    /// ```ignore
    /// let count = Kline::count(Some("BTCUSDT"), MarketType::Spot, KlineInterval::M1).await?;
    /// ```
    pub async fn count(
        symbol: &str,
        market_type: MarketType,
        interval: KlineInterval,
    ) -> Result<u64, mongodb::error::Error> {
        let mut filter = Document::new();

        filter.insert("symbol", symbol);

        filter.insert("market_type", market_type.as_str());
        filter.insert("interval", format!("{:?}", interval));

        let collection_name = Self::get_collection_name(symbol.to_string(), market_type, interval);
        MongoDBClient::count_documents(&collection_name, Some(filter)).await
    }

    // ==================== Update 操作 ====================

    /// 更新单个 Kline
    ///
    /// # 示例
    /// ```ignore
    /// let kline = Kline::new(...);
    /// kline.update_one().await?;
    /// ```
    pub async fn update_one(&self) -> Result<mongodb::results::UpdateResult, mongodb::error::Error> {
        let filter = self.to_filter();
        let update = doc! {
            "open": self.open.to_string(),
            "high": self.high.to_string(),
            "low": self.low.to_string(),
            "close": self.close.to_string(),
            "close_time": self.close_time,
            "volume": self.volume.as_ref().map(|v| v.to_string()),
            "quote_volume": self.quote_volume.as_ref().map(|v| v.to_string()),
            "is_closed": self.is_closed,
        };
        let collection_name = self.collection_name();
        MongoDBClient::update_one(&collection_name, filter, update).await
    }

    /// 更新或插入 Kline（如果不存在则插入，存在则更新）
    ///
    /// # 示例
    /// ```ignore
    /// let kline = Kline::new(...);
    /// kline.upsert_one().await?;
    /// ```
    pub async fn upsert_one(&self) -> Result<mongodb::results::UpdateResult, mongodb::error::Error> {
        let filter = self.to_filter();
        let update = doc! {
            "$set": {
                "open": self.open.to_string(),
                "high": self.high.to_string(),
                "low": self.low.to_string(),
                "close": self.close.to_string(),
                "close_time": self.close_time,
                "volume": self.volume.as_ref().map(|v| v.to_string()),
                "quote_volume": self.quote_volume.as_ref().map(|v| v.to_string()),
                "is_closed": self.is_closed,
            }
        };

        let Some(client) = MongoDBClient::global() else {
            return Err(mongodb::error::Error::custom("MongoDBClient 未初始化"));
        };
        let collection_name = self.collection_name();
        let collection = client.collection::<Document>(&collection_name);

        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(Some(true))
            .build();

        collection.update_one(filter, update).with_options(update_options).await
    }

    // ==================== Delete 操作 ====================

    /// 删除单个 Kline
    ///
    /// # 示例
    /// ```ignore
    /// let kline = Kline::new(...);
    /// kline.delete_one().await?;
    /// ```
    pub async fn delete_one(&self) -> Result<mongodb::results::DeleteResult, mongodb::error::Error> {
        let filter = self.to_filter();
        let collection_name = self.collection_name();
        MongoDBClient::delete_one(&collection_name, filter).await
    }

    /// 根据条件删除多个 Kline
    ///
    /// # 参数
    /// * `symbol` - 交易对（可选）
    /// * `market_type` - 市场类型（必需，用于确定集合）
    /// * `interval` - K线时间间隔（必需，用于确定集合）
    /// * `start_time` - 开始时间戳（可选）
    /// * `end_time` - 结束时间戳（可选）
    ///
    /// # 示例
    /// ```ignore
    /// Kline::delete_many(Some("BTCUSDT"), MarketType::Spot, KlineInterval::M1, Some(1234567890), Some(1234567900)).await?;
    /// ```
    pub async fn delete_many(
        symbol: &str,
        market_type: MarketType,
        interval: KlineInterval,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<mongodb::results::DeleteResult, mongodb::error::Error> {
        let mut filter = Document::new();


        filter.insert("symbol", symbol);

        filter.insert("market_type", market_type.as_str());
        filter.insert("interval", format!("{:?}", interval));

        // 添加时间范围过滤
        if start_time.is_some() || end_time.is_some() {
            let mut time_filter = Document::new();
            if let Some(start) = start_time {
                time_filter.insert("$gte", start);
            }
            if let Some(end) = end_time {
                time_filter.insert("$lte", end);
            }
            filter.insert("open_time", time_filter);
        }

        let collection_name = Self::get_collection_name(symbol.to_string(), market_type, interval);
        MongoDBClient::delete_many(&collection_name, filter).await
    }

    /// 为指定的 market_type 和 interval 创建 MongoDB 索引
    ///
    /// # 参数
    /// * `market_type` - 市场类型
    /// * `interval` - K线时间间隔
    ///
    /// # 示例
    /// ```ignore
    /// Kline::create_indexes(MarketType::Spot, KlineInterval::M1).await?;
    /// ```
    pub async fn create_indexes(collection_name: &str) -> Result<(), mongodb::error::Error> {
        let Some(client) = MongoDBClient::global() else {
            return Err(mongodb::error::Error::custom("MongoDBClient 未初始化"));
        };

        // let collection_name = Self::get_collection_name(symbol.to_string(), market_type, interval);
        let collection = client.collection::<Document>(&collection_name);

        // 创建复合索引：symbol + market_type + interval + open_time（唯一索引）
        let index_options = mongodb::options::IndexOptions::builder()
            .unique(Some(true))
            .build();

        let index_model = mongodb::IndexModel::builder()
            .keys(doc! {
                "symbol": 1,
                "market_type": 1,
                "interval": 1,
                "open_time": 1,
            })
            .options(Some(index_options))
            .build();

        collection.create_index(index_model).await?;

        // 创建时间索引（用于时间范围查询）
        let time_index = mongodb::IndexModel::builder()
            .keys(doc! { "open_time": 1 })
            .build();

        collection.create_index(time_index).await?;

        log::info!("✅ Kline 集合索引创建成功: {}", collection_name);
        Ok(())
    }
}