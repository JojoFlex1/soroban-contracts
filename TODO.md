# Escrow Contract Test Completion for Issue #15

## Steps:
- [ ] 1. Implement comprehensive e2e tests in contracts/escrow/src/test.rs:
  - Deploy CarbonCreditToken for carbon and USDC.
  - Mint tokens to seller (carbon) and buyer (USDC).
  - test_create_offer: assert seller balance decreases, escrow increases, offer stored.
  - test_fill_offer_full: assert atomic swap, offer removed, balances correct.
  - test_fill_offer_partial: assert proportional fill, offer updated.
  - test_cancel_offer: assert remaining carbon returned.
  - Fix existing stub tests with real assertions.
- [ ] 2. Update this TODO.md after test.rs complete.
- [ ] 3. Run `cd contracts/escrow && cargo test` to verify.
- [ ] 4. Run workspace `cargo test`.
- [ ] 5. attempt_completion.

