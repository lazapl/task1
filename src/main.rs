
mod lp_pool;

pub use lp_pool::LpPool;

fn main() {
    let mut pool = LpPool::init(1_500_000, 1_000, 90_000, 90_000_000);
    println!("{:?}", pool);

    let add_liquidity = pool.add_liquidity(100_000_000);
    println!("{} LpToken", add_liquidity);

    let received_tokens = pool.swap(6_000_000);
    println!("{} Token", received_tokens);

    let add_liqudity2 = pool.add_liquidity(10_000_000);
    println!("{} LpToken", add_liqudity2);

    let staked = pool.swap(30_000_000);
    println!("{} Token", staked);

    let (tokens_out, staked_token_out) = pool.remove_liquidity(109_999_100);
    println!("{} Token {} StakedToken", tokens_out, staked_token_out);
}
