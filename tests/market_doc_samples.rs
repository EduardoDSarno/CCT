use serde_json::Value;

#[test]
fn test_bybit_kline_sample_from_docs() {
    let msg = r#"{"topic":"kline.5.BTCUSDT","data":[{"start":1672324800000,"end":1672325099999,"interval":"5","open":"16649.5","close":"16677","high":"16677","low":"16608","volume":"2.081","turnover":"34666.4005","confirm":false,"timestamp":1672324988882}],"ts":1672324988882,"type":"snapshot"}"#;
    let value: Value = serde_json::from_str(msg).expect("Bybit kline sample should be valid JSON");

    assert_eq!(value["topic"], "kline.5.BTCUSDT");
    assert_eq!(value["type"], "snapshot");
    assert!(value["data"].is_array());
    assert_eq!(value["data"][0]["interval"], "5");
}

#[test]
fn test_bybit_trade_sample_from_docs() {
    let msg = r#"{"topic":"publicTrade.BTCUSDT","type":"snapshot","ts":1672304486868,"data":[{"T":1672304486865,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"16578.50","L":"PlusTick","i":"20f43950-d8dd-5b31-9112-a178eb6023af","BT":false,"seq":1783284617}]}"#;
    let value: Value = serde_json::from_str(msg).expect("Bybit trade sample should be valid JSON");

    assert_eq!(value["topic"], "publicTrade.BTCUSDT");
    assert_eq!(value["type"], "snapshot");
    assert!(value["data"].is_array());
    assert_eq!(value["data"][0]["S"], "Buy");
}

#[test]
fn test_hyperliquid_subscribe_sample_from_docs() {
    let msg = r#"{"method":"subscribe","subscription":{"type":"trades","coin":"SOL"}}"#;
    let value: Value = serde_json::from_str(msg).expect("Hyperliquid subscribe sample should be valid JSON");

    assert_eq!(value["method"], "subscribe");
    assert_eq!(value["subscription"]["type"], "trades");
    assert_eq!(value["subscription"]["coin"], "SOL");
}

#[test]
fn test_hyperliquid_subscription_response_sample_from_docs() {
    let msg = r#"{"channel":"subscriptionResponse","data":{"method":"subscribe","subscription":{"type":"trades","coin":"SOL"}}}"#;
    let value: Value = serde_json::from_str(msg).expect("Hyperliquid subscription response should be valid JSON");

    assert_eq!(value["channel"], "subscriptionResponse");
    assert_eq!(value["data"]["method"], "subscribe");
    assert_eq!(value["data"]["subscription"]["type"], "trades");
}
