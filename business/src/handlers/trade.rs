use actix_web::{web, Responder};
use common::models::CreateExchangeOrderReq;
use common::response::R;
use common::AppError;
use orm::entities::AppExchangeSpotCoin;
use rust_decimal::Decimal;
use sa_token_plugin_actix_web::StpUtil;

// 交易API模块
// 现货交易模块
pub async fn create_order(
    req: web::Json<CreateExchangeOrderReq>,
) -> Result<impl Responder, AppError> {
    if req.amount <= Decimal::ZERO {
        return Err(AppError::business("amount-not-null"));
    }
    if req.order_type == 1 && req.price <= Decimal::ZERO {
        return Err(AppError::business("price-not-null"));
    }

    // 创建订单逻辑
    let user_id = match StpUtil::get_login_id_as_long().await {
        Ok(id) => id,
        Err(_) => return Err(AppError::auth("lost-login")),
    };

    // 查询交易对是否存在
    let spot_coin = AppExchangeSpotCoin::select_spot_coin_by_symbol(req.symbol.clone())
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?
        .ok_or_else(|| AppError::business("not-support-symbol"))?;

    if !spot_coin.can_buy.unwrap_or(true) && req.direction == 0 {
        return Err(AppError::business("trading_pair_can_not_buy"));
    } else if !spot_coin.can_sell.unwrap_or(true) && req.direction == 1 {
        return Err(AppError::business("trading_pair_can_not_sell"));
    }

    R::ok()
}

pub async fn cancel_order() {
    // 取消订单逻辑
}

pub async fn get_orders() {
    // 获取订单列表逻辑
}

pub async fn get_trades() {
    // 获取成交记录逻辑
}
