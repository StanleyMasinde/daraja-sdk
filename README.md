# daraja-sdk

A memory-safe Rust SDK for [Safaricom Daraja](https://developer.safaricom.co.ke/) (M-Pesa API 3.0), focused on correctness and type safety.

> **Experimental:** This project is an early experiment. The API is unstable, coverage is limited, and it is not ready for production use.

## OAuth authentication

```rust
use daraja_sdk::mpesa;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let client = mpesa::Client::with_credentials("your-consumer-key", "your-consumer-secret");
    let token = client.generate_access_token().await?;
    println!("{}", token.access_token);

    Ok(())
}
```

## M-Pesa Express (STK Push)

Obtain an access token first, then use [`MpesaExpress`](https://docs.rs/daraja-sdk/latest/daraja_sdk/mpesa/struct.MpesaExpress.html) to send a Lipa na M-Pesa Online prompt:

```rust
use daraja_sdk::mpesa::{ExpressError, MpesaExpress};

#[tokio::main]
async fn main() -> Result<(), ExpressError> {
    // Obtain an access token first (see OAuth section above).
    let response = MpesaExpress::new()
        .access_token("your-access-token")
        .passkey("your-lipa-na-mpesa-passkey")
        .business_short_code(174379)
        .party_a(254700000000)
        .party_b(174379)
        .phone_number(254700000000)
        .amount(1)
        .account_reference("Order123")
        .tx_description("Payment")
        .call_back_url("https://your-domain.com/callback")
        .send_prompt()
        .await?;

    println!("{}", response.customer_message);
    Ok(())
}
```

## Planned features

- [x] **OAuth authentication** — generate access tokens for Daraja API requests
- [x] **M-Pesa Express (STK Push)** — initiate Lipa na M-Pesa Online payments
- [ ] **STK Push query** — query the status of an STK Push request
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
# add your sandbox credentials to config.toml (see below)
cargo build
cargo test
cargo doc --open
```

### `config.toml`

Integration tests call the live Daraja sandbox. Copy `config.toml.example` and fill in:

| Field | Description |
| --- | --- |
| `consumer_key` | Daraja app consumer key |
| `consumer_secret` | Daraja app consumer secret |
| `passkey` | Lipa Na M-Pesa Online passkey (sandbox or production) |
| `callback_url` | HTTPS URL where Daraja posts STK Push results |
| `phone_number` | Safaricom number to receive the STK prompt (`2547XXXXXXXX`) |

`config.toml` is gitignored so credentials are not committed.

To reduce repeated OAuth calls during local development, integration tests cache the last successful access token in `last_token.txt` at the project root. On subsequent runs, tests reuse the cached token if it has not expired instead of requesting a new one from the API. This file is gitignored and contains live credentials — do not commit it. Delete `last_token.txt` if you want to force a fresh token fetch or if the cache format changes.