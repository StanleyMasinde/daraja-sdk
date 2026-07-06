# daraja-sdk

A memory-safe Rust SDK for [Safaricom Daraja](https://developer.safaricom.co.ke/) (M-Pesa API 3.0), focused on correctness and type safety.

> **Experimental:** This project is an early experiment. The API is unstable, coverage is limited, and it is not ready for production use.

```rust
use daraja_sdk::mpesa::Mpesa;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let mpesa = Mpesa::with_credentials(
        "your-consumer-key".into(),
        "your-consumer-secret".into(),
    );

    let token = mpesa.generate_access_token().await?;
    println!("{}", token.access_token);

    Ok(())
}
```

## Planned features

- [x] **OAuth authentication** — generate access tokens for Daraja API requests
- [ ] **M-Pesa Express (STK Push)** — initiate Lipa na M-Pesa Online payments and query transaction status
- [ ] **B2C** — send money from a business shortcode to a customer
- [ ] **C2B** — register validation and confirmation URLs for paybill/till payments
- [ ] **Transaction status** — query the result of a payment request
- [ ] **Account balance** — check balances for a shortcode
- [ ] **Reversals** — reverse a completed transaction
- [ ] **Production environment** — configurable sandbox vs production base URLs

## Developing locally

Requires a [Rust](https://www.rust-lang.org/tools/install) toolchain that supports edition 2024, plus sandbox credentials from the [Daraja Developer Portal](https://developer.safaricom.co.ke/).

```bash
git clone git@github.com:StanleyMasinde/daraja-sdk.git
cd daraja-sdk
cp config.toml.example config.toml
# add your sandbox consumer_key and consumer_secret to config.toml
cargo build
cargo test
cargo doc --open
```

Integration tests call the live Daraja sandbox. `config.toml` is gitignored so credentials are not committed.

To reduce repeated OAuth calls during local development, the integration test caches the last successful access token in `last_token.txt` at the project root. On subsequent runs, the test reuses the cached token if it has not expired instead of requesting a new one from the API. This file is gitignored and contains live credentials — do not commit it. Delete `last_token.txt` if you want to force a fresh token fetch or if the cache format changes.