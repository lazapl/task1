type TokenAmount = u64;
type StakedTokenAmount = u64;
type LPTokenAmount = u64;
type Price = u64; // fixed-point representation
type Fee = u64; // fixed-point representation


#[derive(Debug)]
pub struct LpPool {
    pub token_reserve: TokenAmount,
    pub staked_token_reserve: StakedTokenAmount,
    pub total_lp_tokens: LPTokenAmount,
    price: Price,
    fee_min: Fee,
    fee_max: Fee,
    liquidity_target: TokenAmount,
}

impl LpPool {
    pub fn init(price: Price, fee_min: Fee, fee_max: Fee, liquidity_target: TokenAmount) -> Self {
        Self {
            token_reserve: 0,
            staked_token_reserve: 0,
            total_lp_tokens: 0,
            price,
            fee_min,
            fee_max,
            liquidity_target,
        }
    }

    /// Add liquidity to the pool
    pub fn add_liquidity(&mut self, amount: TokenAmount) -> LPTokenAmount {
        let lp_tokens_to_mint = if self.total_lp_tokens == 0 {
            amount
        } else {
            let amount_u128 = amount as u128;
            let total_lp_tokens_u128 = self.total_lp_tokens as u128;
            let token_reserve_u128 = self.token_reserve as u128;
            (amount_u128 * total_lp_tokens_u128 / token_reserve_u128) as LPTokenAmount
        };
    
        self.token_reserve += amount;
        self.total_lp_tokens += lp_tokens_to_mint;
        lp_tokens_to_mint
    }

     /// Remove liquidity from the pool with rounding up
     pub fn remove_liquidity(&mut self, lp_tokens: LPTokenAmount) -> (TokenAmount, StakedTokenAmount) {
        let token_amount = ((lp_tokens as u128 * self.token_reserve as u128 + self.total_lp_tokens as u128 - 1) / self.total_lp_tokens as u128) as TokenAmount;
        let staked_token_amount = ((lp_tokens as u128 * self.staked_token_reserve as u128 + self.total_lp_tokens as u128 - 1) / self.total_lp_tokens as u128) as StakedTokenAmount;

        self.token_reserve -= token_amount;
        self.staked_token_reserve -= staked_token_amount;
        self.total_lp_tokens -= lp_tokens;

        (token_amount, staked_token_amount)
    }

    /// Swap StakedTokens for Tokens
    pub fn swap(&mut self, staked_token_amount: StakedTokenAmount) -> TokenAmount {
        let fee = self.calculate_fee();
        let token_amount = (staked_token_amount as u128 * self.price as u128 / 1_000_000) as TokenAmount;
        let fee_amount = (token_amount as u128 * fee as u128 / 1_000_000) as TokenAmount;
        let received_token_amount = token_amount - fee_amount;

        self.token_reserve -= received_token_amount;
        self.staked_token_reserve += staked_token_amount;

        received_token_amount
    }

    /// Calculate fee based on the liquidity state
    pub fn calculate_fee(&self) -> Fee {
        if self.token_reserve < self.liquidity_target {
            self.fee_max
        } else {
            // Calculate unstake fee if liquidity falls below target
            let amount_after = self.token_reserve - self.liquidity_target;
            let unstake_fee = self.fee_max - (self.fee_max - self.fee_min) * amount_after / self.liquidity_target;
            unstake_fee
        }
    }
}









#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let pool = LpPool::init(1_500_000, 1_000, 90_000, 90_000_000);
        assert_eq!(pool.price, 1_500_000);
        assert_eq!(pool.fee_min, 1_000);
        assert_eq!(pool.fee_max, 90_000);
        assert_eq!(pool.liquidity_target, 90_000_000);
        assert_eq!(pool.token_reserve, 0);
        assert_eq!(pool.staked_token_reserve, 0);
        assert_eq!(pool.total_lp_tokens, 0);
    }

    #[test]
    fn test_add_liquidity() {
        let mut pool = LpPool::init(1_500_000, 1_000, 90_000, 90_000_000);
        let lp_tokens = pool.add_liquidity(100_000_000);
        assert_eq!(lp_tokens, 100_000_000);
        assert_eq!(pool.token_reserve, 100_000_000);
        assert_eq!(pool.total_lp_tokens, 100_000_000);
    }

    #[test]
    fn test_remove_liquidity() {
        let mut pool = LpPool::init(1_500_000, 1_000, 90_000, 90_000_000);
        pool.add_liquidity(100_000_000);
        let (tokens, staked_tokens) = pool.remove_liquidity(50_000_000);
        assert_eq!(tokens, 50_000_000);
        assert_eq!(staked_tokens, 0);
        assert_eq!(pool.token_reserve, 50_000_000);
        assert_eq!(pool.total_lp_tokens, 50_000_000);
    }

    #[test]
    fn test_swap() {
        let mut pool = LpPool::init(1_500_000, 1_000, 90_000, 90_000_000);
        pool.add_liquidity(100_000_000);
        let received_tokens = pool.swap(6_000_000);
        let token_amount = (6_000_000 as u128 * 1_500_000 as u128 / 1_000_000) as TokenAmount;
        let fee_amount = (token_amount as u128 * 1_000 as u128 / 1_000_000) as TokenAmount;
        let expected_received_tokens = token_amount - fee_amount;

        assert_eq!(received_tokens, expected_received_tokens);
        assert_eq!(pool.token_reserve, 100_000_000 - expected_received_tokens);
        assert_eq!(pool.staked_token_reserve, 6_000_000);
    }

    #[test]
    fn test_calculate_fee() {
        let mut pool = LpPool::init(1_500_000, 1_000, 90_000, 90_000_000);
        pool.add_liquidity(50_000_000);
        assert_eq!(pool.calculate_fee(), 90_000); // Below target, so max fee
        pool.add_liquidity(50_000_000);
        assert_eq!(pool.calculate_fee(), 1_000); // Above target, so min fee
    }
}
