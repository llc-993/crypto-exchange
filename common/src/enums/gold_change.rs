use serde::{Deserialize, Serialize};
use strum::{EnumIter,IntoEnumIterator, AsRefStr};
use crate::models::dto::label::Label;

/// 账变类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, AsRefStr)]
pub enum GoldChangeType {
    /// 提现申请 (-201) - 扣除余额，增加冻结余额
    #[strum(to_string = "提现申请")]
    CashOutRequest = -201,
    /// 提现 (-202) - 扣除冻结余额
    #[strum(to_string = "提现")]
    CashOut = -202,
    /// 提现失败返还 (202) - 扣除冻结余额，增加余额
    #[strum(to_string = "提现失败返还")]
    CashOutFail = 202,
    /// 充值 (888) - 增加余额
    #[strum(to_string = "充值")]
    CashIn = 888,
    /// 后台上分 (1)
    #[strum(to_string = "后台上分")]
    AdminChangeAdd = 1,
    /// 后台冻结 (5)
    #[strum(to_string = "后台冻结")]
    AdminFrozenAdd = 5,
    /// 后台下分 (-1)
    #[strum(serialize = "后台下分", to_string = "后台下分")]
    AdminChangeSub = -1,
    /// 后台解冻 (-4)
    #[strum(serialize = "后台解冻", to_string = "后台解冻")]
    AdminFrozenSub = -4,

}

impl GoldChangeType {
    /// 转换为 i32 值
    pub fn get_code(self) -> i32 {
        self as i32
    }

    /// 从 i32 值转换
    pub fn from_code(value: i32) -> Option<Self> {
        for e in Self::iter() {
            if e.get_code() == value {
                return Some(e);
            }
        }
        None
    }

    /// 获取描述
    pub fn description(&self) -> String {
        self.as_ref().to_string()
    }
    
    /// 获取所有枚举的 Label 列表
    pub fn all_labels() -> Vec<Label<i32, String>> {
        Self::iter().map(|e| {
            Label { value: e.get_code(), label: e.description() }
        }).collect()
    }
}

