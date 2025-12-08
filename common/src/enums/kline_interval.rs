use serde::{Deserialize, Serialize};
use std::fmt;

/// K线时间间隔配置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KlineIntervalConfig {
    /// 本系统格式
    pub system: &'static str,
    /// Binance 格式
    pub binance: &'static str,
    /// Bitget 格式
    pub bitget: &'static str,
    /// OKX 格式
    pub okx: &'static str,
}

/// K线时间间隔枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KlineInterval {
    /// 1分钟
    Min1,
    /// 5分钟
    Min5,
    /// 15分钟
    Min15,
    /// 30分钟
    Min30,
    /// 1小时
    Hour1,
    /// 1天
    Day1,
    /// 1周
    Week1,
    /// 1月
    Month1,
}

impl KlineInterval {
    /// 获取配置
    pub const fn config(&self) -> KlineIntervalConfig {
        match self {
            Self::Min1 => KlineIntervalConfig {
                system: "1min",
                binance: "1m",
                bitget: "candle1m",
                okx: "candle1m",
            },
            Self::Min5 => KlineIntervalConfig {
                system: "5min",
                binance: "5m",
                bitget: "candle5m",
                okx: "candle5m",
            },
            Self::Min15 => KlineIntervalConfig {
                system: "15min",
                binance: "15m",
                bitget: "candle15m",
                okx: "candle15m",
            },
            Self::Min30 => KlineIntervalConfig {
                system: "30min",
                binance: "30m",
                bitget: "candle30m",
                okx: "candle30m",
            },
            Self::Hour1 => KlineIntervalConfig {
                system: "1h",
                binance: "1h",
                bitget: "candle1H",
                okx: "candle1H",
            },
            Self::Day1 => KlineIntervalConfig {
                system: "1day",
                binance: "1d",
                bitget: "candle1D",
                okx: "candle1D",
            },
            Self::Week1 => KlineIntervalConfig {
                system: "1week",
                binance: "1w",
                bitget: "candle1W",
                okx: "candle1W",
            },
            Self::Month1 => KlineIntervalConfig {
                system: "1month",
                binance: "1M",
                bitget: "candle1M",
                okx: "candle1M",
            },
        }
    }

    /// 获取本系统格式
    pub fn interval(&self) -> &'static str {
        self.config().system
    }

    /// 获取 Binance 格式
    pub fn binance_interval(&self) -> &'static str {
        self.config().binance
    }

    /// 获取 Bitget 格式
    pub fn bitget_interval(&self) -> &'static str {
        self.config().bitget
    }

    /// 获取 OKX 格式
    pub fn okx_interval(&self) -> &'static str {
        self.config().okx
    }

    /// 从本系统格式解析
    pub fn from_interval(s: &str) -> Option<Self> {
        Self::all()
            .into_iter()
            .find(|interval| interval.interval() == s)
    }

    /// 从 Binance 格式解析
    pub fn from_binance_interval(s: &str) -> Option<Self> {
        Self::all()
            .into_iter()
            .find(|interval| interval.binance_interval() == s)
    }

    /// 从 Bitget 格式解析
    pub fn from_bitget_interval(s: &str) -> Option<Self> {
        Self::all()
            .into_iter()
            .find(|interval| interval.bitget_interval() == s)
    }

    /// 从 OKX 格式解析
    pub fn from_okx_interval(s: &str) -> Option<Self> {
        Self::all()
            .into_iter()
            .find(|interval| interval.okx_interval() == s)
    }

    /// 获取所有支持的间隔
    pub const fn all() -> [Self; 8] {
        [
            Self::Min1,
            Self::Min5,
            Self::Min15,
            Self::Min30,
            Self::Hour1,
            Self::Day1,
            Self::Week1,
            Self::Month1,
        ]
    }
}

impl fmt::Display for KlineInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.interval())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_formats() {
        // 测试 1分钟
        let min1 = KlineInterval::Min1;
        assert_eq!(min1.interval(), "1min");
        assert_eq!(min1.binance_interval(), "1m");
        assert_eq!(min1.bitget_interval(), "candle1m");
        assert_eq!(min1.okx_interval(), "candle1m");

        // 测试 1小时
        let hour1 = KlineInterval::Hour1;
        assert_eq!(hour1.interval(), "1h");
        assert_eq!(hour1.binance_interval(), "1h");
        assert_eq!(hour1.bitget_interval(), "candle1H");
        assert_eq!(hour1.okx_interval(), "candle1H");

        // 测试 1天
        let day1 = KlineInterval::Day1;
        assert_eq!(day1.interval(), "1day");
        assert_eq!(day1.binance_interval(), "1d");
        assert_eq!(day1.bitget_interval(), "candle1D");
        assert_eq!(day1.okx_interval(), "candle1D");
    }

    #[test]
    fn test_config_access() {
        let config = KlineInterval::Min1.config();
        assert_eq!(config.system, "1min");
        assert_eq!(config.binance, "1m");
        assert_eq!(config.bitget, "candle1m");
        assert_eq!(config.okx, "candle1m");
    }

    #[test]
    fn test_from_interval() {
        assert_eq!(KlineInterval::from_interval("1min"), Some(KlineInterval::Min1));
        assert_eq!(KlineInterval::from_interval("1h"), Some(KlineInterval::Hour1));
        assert_eq!(KlineInterval::from_interval("1day"), Some(KlineInterval::Day1));
        assert_eq!(KlineInterval::from_interval("invalid"), None);
    }

    #[test]
    fn test_from_binance_interval() {
        assert_eq!(KlineInterval::from_binance_interval("1m"), Some(KlineInterval::Min1));
        assert_eq!(KlineInterval::from_binance_interval("1h"), Some(KlineInterval::Hour1));
        assert_eq!(KlineInterval::from_binance_interval("1d"), Some(KlineInterval::Day1));
        assert_eq!(KlineInterval::from_binance_interval("invalid"), None);
    }

    #[test]
    fn test_from_bitget_interval() {
        assert_eq!(KlineInterval::from_bitget_interval("candle1m"), Some(KlineInterval::Min1));
        assert_eq!(KlineInterval::from_bitget_interval("candle1H"), Some(KlineInterval::Hour1));
        assert_eq!(KlineInterval::from_bitget_interval("candle1D"), Some(KlineInterval::Day1));
        assert_eq!(KlineInterval::from_bitget_interval("invalid"), None);
    }

    #[test]
    fn test_from_okx_interval() {
        assert_eq!(KlineInterval::from_okx_interval("candle1m"), Some(KlineInterval::Min1));
        assert_eq!(KlineInterval::from_okx_interval("candle1H"), Some(KlineInterval::Hour1));
        assert_eq!(KlineInterval::from_okx_interval("candle1D"), Some(KlineInterval::Day1));
        assert_eq!(KlineInterval::from_okx_interval("invalid"), None);
    }

    #[test]
    fn test_roundtrip_conversion() {
        for interval in KlineInterval::all() {
            // System format roundtrip
            let system_str = interval.interval();
            assert_eq!(KlineInterval::from_interval(system_str), Some(interval));

            // Binance format roundtrip
            let binance_str = interval.binance_interval();
            assert_eq!(KlineInterval::from_binance_interval(binance_str), Some(interval));

            // Bitget format roundtrip
            let bitget_str = interval.bitget_interval();
            assert_eq!(KlineInterval::from_bitget_interval(bitget_str), Some(interval));

            // OKX format roundtrip
            let okx_str = interval.okx_interval();
            assert_eq!(KlineInterval::from_okx_interval(okx_str), Some(interval));
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", KlineInterval::Min1), "1min");
        assert_eq!(format!("{}", KlineInterval::Hour1), "1h");
        assert_eq!(format!("{}", KlineInterval::Day1), "1day");
    }

    #[test]
    fn test_all_intervals() {
        let all = KlineInterval::all();
        assert_eq!(all.len(), 8);
        assert!(all.contains(&KlineInterval::Min1));
        assert!(all.contains(&KlineInterval::Month1));
    }
}
