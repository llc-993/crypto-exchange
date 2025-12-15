use std::sync::Arc;
use tokio::sync::OnceCell;
use mongodb::{Client, Database, Collection};
use mongodb::options::{ClientOptions, FindOptions};
use bson::{doc, Document};
use serde::{Deserialize, Serialize};

/// 全局 MongoDBClient 单例
static GLOBAL_MONGODB_CLIENT: OnceCell<Arc<MongoDBClient>> = OnceCell::const_new();

/// MongoDB 客户端封装
/// 支持单例模式，一次初始化后全局使用
pub struct MongoDBClient {
    #[allow(dead_code)]
    client: Arc<Client>,
    database: Arc<Database>,
}

impl MongoDBClient {
    /// 创建未初始化的 MongoDBClient
    fn new(client: Client, database_name: String) -> Self {
        let database = client.database(&database_name);
        Self {
            client: Arc::new(client),
            database: Arc::new(database),
        }
    }

    /// 初始化全局 MongoDBClient
    /// 
    /// # 参数
    /// * `url` - MongoDB 连接 URL，例如: "mongodb://localhost:27017"
    /// * `database` - 数据库名称
    /// * `max_pool_size` - 最大连接池大小（可选）
    /// * `min_pool_size` - 最小连接池大小（可选）
    pub async fn init_global(
        url: &str,
        database: &str,
        max_pool_size: Option<u32>,
        min_pool_size: Option<u32>,
    ) -> Result<(), mongodb::error::Error> {
        let mut client_options = ClientOptions::parse(url).await?;
        
        // 设置连接池大小
        if let Some(max) = max_pool_size {
            client_options.max_pool_size = Some(max);
        }
        if let Some(min) = min_pool_size {
            client_options.min_pool_size = Some(min);
        }

        let client = Client::with_options(client_options)?;
        
        // 测试连接
        client
            .database("admin")
            .run_command(doc! {"ping": 1})
            .await?;

        let mongo_client = Arc::new(Self::new(client, database.to_string()));
        
        GLOBAL_MONGODB_CLIENT.set(mongo_client.clone())
            .map_err(|_| mongodb::error::Error::custom("Global MongoDBClient already initialized"))?;

        log::info!("✅ 全局 MongoDBClient 已初始化: {} -> {}", url, database);
        Ok(())
    }

    /// 获取全局 MongoDBClient 实例
    pub fn global() -> Option<Arc<MongoDBClient>> {
        GLOBAL_MONGODB_CLIENT.get().cloned()
    }

    /// 获取数据库实例
    pub fn database(&self) -> &Database {
        &self.database
    }

    /// 获取指定集合
    pub fn collection<T: Send + Sync>(&self, name: &str) -> Collection<T> {
        self.database.collection(name)
    }

    // ==================== 静态便捷方法 ====================

    /// 插入单个文档
    /// 
    /// # 示例
    /// ```ignore
    /// MongoDBClient::insert_one("users", &user_data).await?;
    /// ```
    pub async fn insert_one<T: Serialize + Send + Sync>(
        collection_name: &str,
        document: &T,
    ) -> Result<mongodb::results::InsertOneResult, mongodb::error::Error> {
        let Some(client) = Self::global() else {
            return Err(mongodb::error::Error::custom("MongoDBClient 未初始化"));
        };
        let collection = client.collection::<T>(collection_name);
        collection.insert_one(document).await
    }

    /// 插入多个文档
    pub async fn insert_many<T: Serialize + Send + Sync>(
        collection_name: &str,
        documents: Vec<T>,
    ) -> Result<mongodb::results::InsertManyResult, mongodb::error::Error> {
        let Some(client) = Self::global() else {
            return Err(mongodb::error::Error::custom("MongoDBClient 未初始化"));
        };
        let collection = client.collection::<T>(collection_name);

        collection.insert_many(documents).await
    }

    /// 查找单个文档
    pub async fn find_one<T: for<'de> Deserialize<'de> + Send + Sync>(
        collection_name: &str,
        filter: Document,
    ) -> Result<Option<T>, mongodb::error::Error> {
        let Some(client) = Self::global() else {
            return Err(mongodb::error::Error::custom("MongoDBClient 未初始化"));
        };
        let collection = client.collection::<T>(collection_name);
        collection.find_one(filter).await
    }

    /// 查找多个文档
    pub async fn find_many<T: for<'de> Deserialize<'de> + Send + Sync>(
        collection_name: &str,
        filter: Document,
        options: Option<FindOptions>,
    ) -> Result<Vec<T>, mongodb::error::Error> {
        let Some(client) = Self::global() else {
            return Err(mongodb::error::Error::custom("MongoDBClient 未初始化"));
        };
        let collection = client.collection::<T>(collection_name);
        let mut cursor = match options {
            Some(opts) => collection.find(filter).with_options(opts).await?,
            None => collection.find(filter).await?,
        };
        let mut results = Vec::new();
        while let Ok(true) = cursor.advance().await {
            match cursor.deserialize_current() {
                Ok(doc) => results.push(doc),
                Err(e) => {
                    log::error!("反序列化文档失败: {}", e);
                    break;
                }
            }
        }
        Ok(results)
    }

    /// 更新单个文档
    pub async fn update_one(
        collection_name: &str,
        filter: Document,
        update: Document,
    ) -> Result<mongodb::results::UpdateResult, mongodb::error::Error> {
        let Some(client) = Self::global() else {
            return Err(mongodb::error::Error::custom("MongoDBClient 未初始化"));
        };
        let collection = client.collection::<Document>(collection_name);
        let update_doc = doc! {"$set": update};
        collection.update_one(filter, update_doc).await
    }

    /// 更新多个文档
    pub async fn update_many(
        collection_name: &str,
        filter: Document,
        update: Document,
    ) -> Result<mongodb::results::UpdateResult, mongodb::error::Error> {
        let Some(client) = Self::global() else {
            return Err(mongodb::error::Error::custom("MongoDBClient 未初始化"));
        };
        let collection = client.collection::<Document>(collection_name);
        let update_doc = doc! {"$set": update};
        collection.update_many(filter, update_doc).await
    }

    /// 删除单个文档
    pub async fn delete_one(
        collection_name: &str,
        filter: Document,
    ) -> Result<mongodb::results::DeleteResult, mongodb::error::Error> {
        let Some(client) = Self::global() else {
            return Err(mongodb::error::Error::custom("MongoDBClient 未初始化"));
        };
        let collection = client.collection::<Document>(collection_name);
        collection.delete_one(filter).await
    }

    /// 删除多个文档
    pub async fn delete_many(
        collection_name: &str,
        filter: Document,
    ) -> Result<mongodb::results::DeleteResult, mongodb::error::Error> {
        let Some(client) = Self::global() else {
            return Err(mongodb::error::Error::custom("MongoDBClient 未初始化"));
        };
        let collection = client.collection::<Document>(collection_name);
        collection.delete_many(filter).await
    }

    /// 统计文档数量
    pub async fn count_documents(
        collection_name: &str,
        filter: Option<Document>,
    ) -> Result<u64, mongodb::error::Error> {
        let Some(client) = Self::global() else {
            return Err(mongodb::error::Error::custom("MongoDBClient 未初始化"));
        };
        let collection = client.collection::<Document>(collection_name);
        collection.count_documents(filter.unwrap_or_default()).await
    }

    /// 检查集合是否存在
    pub async fn collection_exists(collection_name: &str) -> Result<bool, mongodb::error::Error> {
        let Some(client) = Self::global() else {
            return Err(mongodb::error::Error::custom("MongoDBClient 未初始化"));
        };
        let collections = client.database.list_collection_names().await?;
        Ok(collections.iter().any(|name| name == collection_name))
    }
}

impl Default for MongoDBClient {
    fn default() -> Self {
        // 这个实现不应该被调用，因为需要通过 init_global 初始化
        panic!("MongoDBClient 必须通过 init_global 初始化");
    }
}

