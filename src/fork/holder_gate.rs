// Copyright (c) 2024-2026 ScreenerBot (screenerbot.io)
// SPDX-License-Identifier: BUSL-1.1

//! RugCheck's `topHolders` includes pool vaults / bonding curves; this
//! module separates ordinary holders from pool accounts and sums the top N.

use std::collections::HashSet;

use crate::pools::types::PoolDescriptor;
use crate::tokens::TokenHolder;

/// Well-known AMM / LP authority wallets that own pool vault token accounts.
/// These are authorities, not program IDs. They are copied from the venue modules (cited per line).
pub const KNOWN_AMM_AUTHORITIES: &[&str] = &[
    "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1", // Raydium AMM v4 authority (swaps/direct/venues/raydium_amm_v4.rs)
    "HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC", // Meteora DAMM v2 pool authority (venues/meteora_damm.rs)
    "FhVo3mqL8PW5pH5U2CN4XE33DokiyZnUwuGpH2hmHLuM", // Meteora DBC pool authority (venues/meteora_dbc.rs)
];

/// Addresses that identify pool-owned holdings of one token.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PoolContext {
    /// Pool / bonding-curve account addresses (a vault's owner is often the pool PDA itself).
    pub pool_addresses: HashSet<String>,
    /// Pool reserve (vault) token-account addresses.
    pub vault_addresses: HashSet<String>,
    /// AMM / LP authority wallets that own vaults.
    pub amm_authorities: HashSet<String>,
}

impl PoolContext {
    /// Empty pool/vault sets, `amm_authorities` = KNOWN_AMM_AUTHORITIES.
    pub fn with_known_authorities() -> Self {
        Self {
            pool_addresses: HashSet::new(),
            vault_addresses: HashSet::new(),
            amm_authorities: KNOWN_AMM_AUTHORITIES
                .iter()
                .map(|&s| s.to_string())
                .collect(),
        }
    }

    /// `with_known_authorities()` plus every `pool_id.address()` into `pool_addresses` and every
    /// `reserve_accounts[i].address()` into `vault_addresses`, for every descriptor given.
    pub fn from_pools(pools: &[PoolDescriptor]) -> Self {
        let mut ctx = Self::with_known_authorities();
        for pool in pools {
            ctx.pool_addresses
                .insert(pool.pool_id.address().to_string());
            for reserve in &pool.reserve_accounts {
                ctx.vault_addresses.insert(reserve.address().to_string());
            }
        }
        ctx
    }

    /// True when the holder is pool-owned:
    ///   holder.address ∈ vault_addresses ∪ pool_addresses
    ///   OR holder.owner (when Some) ∈ pool_addresses ∪ vault_addresses ∪ amm_authorities.
    /// Exact, case-sensitive string compare (base58). Empty / whitespace-only owner strings never match.
    pub fn is_pool_holder(&self, holder: &TokenHolder) -> bool {
        if self.vault_addresses.contains(&holder.address)
            || self.pool_addresses.contains(&holder.address)
        {
            return true;
        }
        if let Some(owner) = &holder.owner {
            if !owner.trim().is_empty() {
                return self.pool_addresses.contains(owner)
                    || self.vault_addresses.contains(owner)
                    || self.amm_authorities.contains(owner);
            }
        }
        false
    }
}

/// Holders that are not pool-owned, in input order. Holders whose `pct` is not finite
/// (NaN, ±inf) are dropped as well, so the caller never ranks a junk reading.
pub fn ordinary_holders<'a>(holders: &'a [TokenHolder], ctx: &PoolContext) -> Vec<&'a TokenHolder> {
    holders
        .iter()
        .filter(|holder| holder.pct.is_finite() && !ctx.is_pool_holder(holder))
        .collect()
}

/// Sum of the `n` largest finite `pct` values among `holders`.
/// Rules (same as `three_largest_pcts` in src/filtering/sources/rugcheck.rs):
/// - non-finite pct is skipped, never ranked;
/// - ties count as separate holders (three at 20.0 → 60.0 for n ≥ 3);
/// - input order does not matter (sort a local Vec<f64> descending with `f64::total_cmp`
///   AFTER filtering non-finite values);
/// - fewer than `n` finite holders → sum of all of them.
///
/// Returns `None` when `n == 0` or there is no finite holder at all. Missing data is never reported as 0.
pub fn top_n_sum(holders: &[&TokenHolder], n: usize) -> Option<f64> {
    if n == 0 {
        return None;
    }
    let mut pcts: Vec<f64> = holders
        .iter()
        .map(|h| h.pct)
        .filter(|pct| pct.is_finite())
        .collect();
    if pcts.is_empty() {
        return None;
    }
    pcts.sort_by(|a, b| b.total_cmp(a));
    let sum: f64 = pcts.into_iter().take(n).sum();
    Some(sum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::{AccountId, AssetId, ChainId, PoolId};
    use crate::pools::types::ProtocolId;
    use std::time::Instant;

    fn holder(address: &str, pct: f64, owner: Option<&str>) -> TokenHolder {
        TokenHolder {
            address: address.to_string(),
            amount: "0".to_string(),
            pct,
            owner: owner.map(str::to_string),
            insider: false,
        }
    }

    #[test]
    fn pool_vault_excluded() {
        let mut vault_addresses = HashSet::new();
        vault_addresses.insert("Vault111".to_string());
        let mut pool_addresses = HashSet::new();
        pool_addresses.insert("Pool111".to_string());
        let ctx = PoolContext {
            pool_addresses,
            vault_addresses,
            ..Default::default()
        };
        let h_vault = holder("Vault111", 50.0, None);
        let h_pool = holder("Pool111", 40.0, None);
        let h_ord = holder("Ordinary111", 10.0, None);
        assert!(ctx.is_pool_holder(&h_vault));
        assert!(ctx.is_pool_holder(&h_pool));
        assert!(!ctx.is_pool_holder(&h_ord));
        let list = [h_vault, h_pool, h_ord];
        let ordinary = ordinary_holders(&list, &ctx);
        assert_eq!(ordinary.len(), 1);
        assert_eq!(ordinary[0].address, "Ordinary111");
    }

    #[test]
    fn pool_address_owner_excluded() {
        let mut pool_addresses = HashSet::new();
        pool_addresses.insert("Pool111".to_string());
        let ctx = PoolContext {
            pool_addresses,
            ..Default::default()
        };
        let h1 = holder("RandomVaultAddr", 80.0, Some("Pool111"));
        let h2 = holder("Ordinary111", 5.0, None);
        let list = [h1, h2];
        let ordinary = ordinary_holders(&list, &ctx);
        assert_eq!(ordinary.len(), 1);
        assert_eq!(ordinary[0].address, "Ordinary111");
    }

    #[test]
    fn vault_address_owner_excluded() {
        let mut vault_addresses = HashSet::new();
        vault_addresses.insert("Vault111".to_string());
        let ctx = PoolContext {
            vault_addresses,
            ..Default::default()
        };
        let h1 = holder("RandomAddr", 80.0, Some("Vault111"));
        let h2 = holder("Ordinary111", 5.0, None);
        assert!(ctx.is_pool_holder(&h1));
        assert!(!ctx.is_pool_holder(&h2));
        let list = [h1, h2];
        let ordinary = ordinary_holders(&list, &ctx);
        assert_eq!(ordinary.len(), 1);
        assert_eq!(ordinary[0].address, "Ordinary111");
    }

    #[test]
    fn amm_authority_owner_excluded() {
        let ctx = PoolContext::with_known_authorities();
        let h1 = holder("SomeVaultAddr", 90.0, Some(KNOWN_AMM_AUTHORITIES[0]));
        let h2 = holder("Ordinary111", 5.0, None);
        let list = [h1, h2];
        let ordinary = ordinary_holders(&list, &ctx);
        assert_eq!(ordinary.len(), 1);
        assert_eq!(ordinary[0].address, "Ordinary111");
    }

    #[test]
    fn ordinary_holder_kept() {
        let mut ctx = PoolContext::with_known_authorities();
        ctx.pool_addresses.insert("Pool111".to_string());
        ctx.vault_addresses.insert("Vault111".to_string());

        let h1 = holder("Ordinary111", 5.0, None);
        let h2 = holder("Ordinary222", 6.0, Some(""));
        let h3 = holder("Ordinary333", 7.0, Some("   "));
        let h4 = holder("Ordinary444", 8.0, Some("SomeWallet"));

        assert!(!ctx.is_pool_holder(&h1));
        assert!(!ctx.is_pool_holder(&h2));
        assert!(!ctx.is_pool_holder(&h3));
        assert!(!ctx.is_pool_holder(&h4));

        let list = [h1, h2, h3, h4];
        let ordinary = ordinary_holders(&list, &ctx);
        assert_eq!(ordinary.len(), 4);
    }

    #[test]
    fn match_is_case_sensitive() {
        let mut vault_addresses = HashSet::new();
        vault_addresses.insert("Vault111".to_string());
        let ctx_vault = PoolContext {
            vault_addresses,
            ..Default::default()
        };
        let h_addr_lower = holder("vault111", 10.0, None);
        assert!(!ctx_vault.is_pool_holder(&h_addr_lower));
        let list1 = [h_addr_lower];
        let ordinary1 = ordinary_holders(&list1, &ctx_vault);
        assert_eq!(ordinary1.len(), 1);

        let ctx_auth = PoolContext::with_known_authorities();
        let lower_auth = KNOWN_AMM_AUTHORITIES[0].to_lowercase();
        let h_owner_lower = holder("SomeAddr", 10.0, Some(&lower_auth));
        assert!(!ctx_auth.is_pool_holder(&h_owner_lower));
        let list2 = [h_owner_lower];
        let ordinary2 = ordinary_holders(&list2, &ctx_auth);
        assert_eq!(ordinary2.len(), 1);
    }

    #[test]
    fn nan_pct_dropped() {
        let ctx = PoolContext::default();
        let h_nan = holder("H1", f64::NAN, None);
        let h_30 = holder("H2", 30.0, None);
        let h_inf = holder("H3", f64::INFINITY, None);
        let h_neginf = holder("H4", f64::NEG_INFINITY, None);
        let h_10 = holder("H5", 10.0, None);
        let list = [
            h_nan.clone(),
            h_30.clone(),
            h_inf.clone(),
            h_neginf.clone(),
            h_10.clone(),
        ];
        let ordinary = ordinary_holders(&list, &ctx);
        assert_eq!(ordinary.len(), 2);
        assert_eq!(ordinary[0].address, "H2");
        assert_eq!(ordinary[1].address, "H5");

        let top1 = top_n_sum(&ordinary, 1);
        assert!(top1.is_some());
        assert!((top1.unwrap() - 30.0).abs() < 1e-9);

        let all_refs = [&h_nan, &h_30, &h_inf, &h_neginf, &h_10];
        let top10 = top_n_sum(&all_refs, 10);
        assert!(top10.is_some());
        assert!((top10.unwrap() - 40.0).abs() < 1e-9);
    }

    #[test]
    fn top10_sum_with_fewer_than_10_holders() {
        let holders = [
            holder("H1", 5.0, None),
            holder("H2", 4.0, None),
            holder("H3", 3.0, None),
            holder("H4", 2.0, None),
        ];
        let refs: Vec<&TokenHolder> = holders.iter().collect();
        let sum = top_n_sum(&refs, 10);
        assert!(sum.is_some());
        assert!((sum.unwrap() - 14.0).abs() < 1e-9);
    }

    #[test]
    fn top_n_sum_takes_largest_n_from_unsorted() {
        let pcts = [
            4.0, 1.0, 11.0, 7.0, 2.0, 12.0, 9.0, 3.0, 8.0, 5.0, 10.0, 6.0,
        ];
        let holders: Vec<TokenHolder> = pcts
            .iter()
            .enumerate()
            .map(|(i, &p)| holder(&format!("H{i}"), p, None))
            .collect();
        let refs: Vec<&TokenHolder> = holders.iter().collect();
        let sum = top_n_sum(&refs, 10);
        assert!(sum.is_some());
        assert!((sum.unwrap() - 75.0).abs() < 1e-9);
    }

    #[test]
    fn top_n_sum_ties_counted_separately() {
        let holders = [
            holder("H1", 20.0, None),
            holder("H2", 20.0, None),
            holder("H3", 20.0, None),
        ];
        let refs: Vec<&TokenHolder> = holders.iter().collect();
        let sum = top_n_sum(&refs, 3);
        assert!(sum.is_some());
        assert!((sum.unwrap() - 60.0).abs() < 1e-9);
    }

    #[test]
    fn top_n_sum_none_without_data() {
        assert!(top_n_sum(&[], 10).is_none());

        let nan_holder = holder("H1", f64::NAN, None);
        assert!(top_n_sum(&[&nan_holder], 10).is_none());

        let valid_holder = holder("H2", 10.0, None);
        assert!(top_n_sum(&[&valid_holder], 0).is_none());
    }

    #[test]
    fn from_pools_collects_pool_and_vault_addresses() {
        let descriptor = PoolDescriptor {
            pool_id: PoolId::new(ChainId::Solana, "PoolAddr111").unwrap(),
            program_kind: ProtocolId::new("RAYDIUM CPMM"),
            base_mint: AssetId::new(ChainId::Solana, "TokenMint111").unwrap(),
            quote_mint: AssetId::new(
                ChainId::Solana,
                "So11111111111111111111111111111111111111112",
            )
            .unwrap(),
            reserve_accounts: vec![
                AccountId::new(ChainId::Solana, "Vault1").unwrap(),
                AccountId::new(ChainId::Solana, "Vault2").unwrap(),
            ],
            liquidity_usd: 12_345.0,
            volume_h24_usd: 6789.0,
            last_updated: Instant::now(),
        };

        let ctx = PoolContext::from_pools(&[descriptor]);
        assert!(ctx.pool_addresses.contains("PoolAddr111"));
        assert!(ctx.vault_addresses.contains("Vault1"));
        assert!(ctx.vault_addresses.contains("Vault2"));
        for &auth in KNOWN_AMM_AUTHORITIES {
            assert!(ctx.amm_authorities.contains(auth));
        }
    }
}
