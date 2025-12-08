# 加密货币交易所

使用Rust开发的加密货币交易所系统。

## 项目结构

```
crypto-exchange/
├── business/       # 用户API请求接口
├── agent/          # 代理服务
├── manage/         # 后台管理
├── common/         # 公共配置库(redis、mysql等)
├── exchange/       # 交易撮合系统(现货、合约、期货)
├── input/          # 接入第三方交易所数据(binance、huobi等)
├── market/         # 平台币AI做市
├── orm/            # 数据库操作库
├── job/            # 定时任务系统
└── block/          # 区块链模块
    ├── eth/        # 以太坊模块
    ├── btc/        # 比特币模块
    └── sol/        # Solana模块
```

## 模块说明

- **business**: 用户API请求接口，处理用户的交易、查询等请求
- **agent**: 代理服务，提供API代理功能
- **manage**: 后台管理系统，提供管理界面API
- **common**: 公共配置库，包含Redis、MySQL等连接配置
- **exchange**: 交易撮合系统，支持现货、合约、期货交易
- **input**: 接入第三方交易所数据，如Binance、Huobi等
- **market**: 平台币AI做市系统
- **orm**: 数据库操作库
- **job**: 定时任务系统
- **block**: 区块链模块，包含以太坊、比特币、Solana等子模块

## 运行方式

每个模块可以单独运行，例如：

```bash
# 运行用户API服务
cd business
cargo run

# 运行以太坊区块链服务
cd block/eth
cargo run
```

## 依赖关系

- common和orm为基础库，被其他模块依赖
- block为区块链基础库，被eth、btc、sol子模块依赖
