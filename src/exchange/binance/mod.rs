use super::epoch_ms_to_datetime_utc;
use crate::{ExchangeTransformerId, Validator, error::DataError, model::{Direction, MarketData, Trade}, SubscriptionId, Identifiable};
use barter_integration::Instrument;
use serde::{Deserialize, Serialize};
use chrono::Utc;

/// Todo:
pub mod futures;


/// `Binance` & `BinanceFutures` `Subscription` response message.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct BinanceSubResponse {
    result: Option<Vec<String>>,
    id: u32,
}

impl Validator for BinanceSubResponse {
    fn validate(self) -> Result<Self, DataError>
    where
        Self: Sized,
    {
        if self.result.is_none() {
            Ok(self)
        } else {
            Err(DataError::Subscribe(
                "received failure subscription response".to_owned(),
            ))
        }
    }
}

/// Binance Message variants that could be received over [`WebSocket`].
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum BinanceMessage {
    Trade(BinanceTrade)
}

impl Identifiable for BinanceMessage {
    fn id(&self) -> SubscriptionId {
        match self {
            BinanceMessage::Trade(trade) => SubscriptionId::from(trade)
        }
    }
}

/// Binance specific Trade message.
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct BinanceTrade {
    #[serde(rename = "e")]
    event_type: String,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "T")]
    trade_ts: u64,
    #[serde(rename = "a")]
    id: u64,
    #[serde(rename = "p")]
    price: f64,
    #[serde(rename = "q")]
    quantity: f64,
    #[serde(rename = "m")]
    buyer_is_maker: bool,
}

impl From<&BinanceTrade> for SubscriptionId {
    fn from(trade: &BinanceTrade) -> Self {
        SubscriptionId(format!(
            "{}@{}",
            trade.symbol.to_lowercase(),
            trade.event_type
        ))
    }
}

impl From<(ExchangeTransformerId, Instrument, BinanceTrade)> for MarketData {
    fn from((exchange, instrument, trade): (ExchangeTransformerId, Instrument, BinanceTrade)) -> Self {
        Self::Trade(Trade {
            id: trade.id.to_string(),
            exchange: exchange.exchange().to_string(),
            instrument,
            received_timestamp: Utc::now(),
            exchange_timestamp: epoch_ms_to_datetime_utc(trade.trade_ts),
            price: trade.price,
            quantity: trade.quantity,
            direction: if trade.buyer_is_maker {
                Direction::Sell
            } else {
                Direction::Buy
            }
        })
    }
}