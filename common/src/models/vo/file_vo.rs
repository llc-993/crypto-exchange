use serde::{Deserialize, Serialize};

/// 上传文件返回信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileVO {
    /// 文件上传到服务器的相对路径，访问文件: 文件下载域名 + '/' + 相对路径
    pub file_path: Option<String>,
    /// 文件下载域名
    pub file_host: Option<String>,
}

impl FileVO {
    pub fn new(file_path: String, file_host: String) -> Self {
        Self {
            file_path: Some(file_path),
            file_host: Some(file_host),
        }
    }
}
