use serum_dex::state::Market;

pub fn max_bid(serum_market: &Market) -> f32 {
    *serum_market.bids.iter().max().expect("Bids empty") as f32
}

pub fn min_ask(serum_market: &Market) -> f32 {
    *serum_market.asks.iter().min().expect("Asks empty") as f32
}

pub fn get_serum_spot_price(serum_market: &Market) -> f32 {
    let max_bid = max_bid(serum_market);
    let min_ask = min_ask(serum_market);

    let mut diff: f32 = min_ask - max_bid;
    if diff < 0f32 {
        diff = 0f32;
    }

    diff /= 2f32;

    max_bid + diff
}
