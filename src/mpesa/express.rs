use std::fmt;

use base64::prelude::*;
use chrono::{FixedOffset, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::types::{DarajaApi, DarajaEnvironment};

const EAT_OFFSET_SECS: i32 = 3 * 3600;
const MAX_ACCOUNT_REFERENCE_LEN: usize = 12;
const MAX_TRANSACTION_DESC_LEN: usize = 13;

/// Errors returned when initiating an M-Pesa Express (STK Push) request.
#[derive(Debug)]
pub enum ExpressError {
    /// A required field was missing or violated Daraja length/format constraints.
    Validation(String),
    /// An HTTP failure or JSON deserialization error from the underlying request.
    Request(reqwest::Error),
    /// Daraja returned a non-success response with an API error payload.
    Api {
        /// Daraja error code.
        error_code: String,
        /// Human-readable error message from Daraja.
        error_message: String,
    },
    /// Daraja returned a non-success HTTP status without a parseable error body.
    UnexpectedResponse {
        /// HTTP status code.
        status: reqwest::StatusCode,
        /// Raw response body, when available.
        body: String,
    },
}

impl fmt::Display for ExpressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(message) => write!(f, "validation error: {message}"),
            Self::Request(err) => write!(f, "request error: {err}"),
            Self::Api {
                error_code,
                error_message,
            } => write!(f, "Daraja API error {error_code}: {error_message}"),
            Self::UnexpectedResponse { status, body } => {
                write!(f, "unexpected response ({status}): {body}")
            }
        }
    }
}

impl std::error::Error for ExpressError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Request(err) => Some(err),
            _ => None,
        }
    }
}

/// The immediate response from Daraja after initiating an STK Push request.
///
/// Returned by [`MpesaExpress::send_prompt`]. A `response_code` of `"0"` means Daraja
/// accepted the request; the customer still completes payment on their phone.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct StkPushResponse {
    /// Global unique identifier for the submitted payment request.
    #[serde(rename = "MerchantRequestID")]
    pub merchant_request_id: String,

    /// Global unique identifier for the processed checkout transaction request.
    #[serde(rename = "CheckoutRequestID")]
    pub checkout_request_id: String,

    /// Response code from Daraja. `"0"` indicates the request was accepted.
    pub response_code: String,

    /// Human-readable description of the response.
    pub response_description: String,

    /// Message displayed to the customer on the STK prompt.
    pub customer_message: String,
}

/// Identifies the STK Push transaction type sent to Daraja.
#[derive(Default, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    /// Use for PayBill numbers.
    #[default]
    CustomerPayBillOnline,
    /// Use for Till numbers.
    CustomerBuyGoodsOnline,
}

#[derive(Serialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
struct MpesaExpressRequest {
    business_short_code: u32,
    password: String,
    timestamp: String,
    transaction_type: TransactionType,
    amount: u32,
    party_a: u64,
    party_b: u32,
    phone_number: u64,
    #[serde(rename = "CallBackURL")]
    call_back_url: String,
    account_reference: String,
    transaction_desc: String,
}

#[derive(Deserialize)]
struct DarajaErrorResponse {
    #[serde(rename = "errorCode")]
    error_code: String,
    #[serde(rename = "errorMessage")]
    error_message: String,
}

/// Builder for initiating M-Pesa Express (STK Push) payment prompts.
///
/// Obtain an access token via [`crate::mpesa::Client::generate_access_token`], configure
/// the required fields, then call [`MpesaExpress::send_prompt`].
///
/// A callback URL is required by Daraja but must be handled by your application;
/// this SDK does not process callback payloads.
///
/// # Examples
///
/// ```no_run
/// use daraja_sdk::mpesa::{ExpressError, MpesaExpress};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), ExpressError> {
/// let response = MpesaExpress::new()
///     .access_token("your-access-token")
///     .passkey("your-passkey")
///     .business_short_code(174379)
///     .party_a(254700000000)
///     .party_b(174379)
///     .phone_number(254700000000)
///     .amount(1)
///     .account_reference("Order123")
///     .tx_description("Payment")
///     .call_back_url("https://your-domain.com/callback")
///     .send_prompt()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct MpesaExpress {
    passkey: String,
    access_token: String,
    request_body: MpesaExpressRequest,
    http_client: Client,
    environment: DarajaEnvironment,
}

impl DarajaApi for MpesaExpress {
    fn path(&self) -> &'static str {
        "mpesa/stkpush/v1/processrequest"
    }

    fn environment(&self) -> DarajaEnvironment {
        self.environment
    }
}

impl MpesaExpress {
    /// Creates a new STK Push builder with default field values.
    pub fn new() -> Self {
        Self {
            access_token: String::new(),
            request_body: MpesaExpressRequest::default(),
            passkey: String::new(),
            http_client: Client::new(),
            environment: DarajaEnvironment::default(),
        }
    }

    /// Sets the OAuth access token obtained from [`crate::mpesa::Client::generate_access_token`].
    pub fn access_token(mut self, token: &str) -> Self {
        self.access_token = token.into();
        self
    }

    /// Sets the Lipa Na M-Pesa Online passkey for password generation.
    pub fn passkey(mut self, passkey: &str) -> Self {
        self.passkey = passkey.into();
        self
    }

    /// Sets the business shortcode (PayBill or Till number).
    pub fn business_short_code(mut self, shortcode: u32) -> Self {
        self.request_body.business_short_code = shortcode;
        self
    }

    /// Sets the transaction type (`CustomerPayBillOnline` or `CustomerBuyGoodsOnline`).
    pub fn transaction_type(mut self, transaction_type: TransactionType) -> Self {
        self.request_body.transaction_type = transaction_type;
        self
    }

    /// Sets the transaction amount in Kenyan Shillings.
    pub fn amount(mut self, amount: u32) -> Self {
        self.request_body.amount = amount;
        self
    }

    /// Sets the phone number sending money (`2547XXXXXXXX`).
    pub fn party_a(mut self, party_a: u64) -> Self {
        self.request_body.party_a = party_a;
        self
    }

    /// Sets the organization receiving the funds (credit party).
    pub fn party_b(mut self, party_b: u32) -> Self {
        self.request_body.party_b = party_b;
        self
    }

    /// Sets the mobile number to receive the USSD prompt (`2547XXXXXXXX`).
    pub fn phone_number(mut self, phone_number: u64) -> Self {
        self.request_body.phone_number = phone_number;
        self
    }

    /// Sets the HTTPS callback URL Daraja requires for posting transaction results.
    ///
    /// Your application must expose and handle this endpoint.
    pub fn call_back_url(mut self, url: &str) -> Self {
        self.request_body.call_back_url = url.into();
        self
    }

    /// Sets the account reference shown to the customer (max 12 characters).
    pub fn account_reference(mut self, reference: &str) -> Self {
        self.request_body.account_reference = reference.into();
        self
    }

    /// Sets the transaction description shown to the customer (max 13 characters).
    pub fn tx_description(mut self, description: &str) -> Self {
        self.request_body.transaction_desc = description.into();
        self
    }

    /// Validates the builder state, sends the STK Push request, and returns Daraja's response.
    ///
    /// Timestamps use East Africa Time (UTC+3) as required by Daraja.
    ///
    /// # Errors
    ///
    /// Returns [`ExpressError`] when required fields are missing/invalid, the HTTP request
    /// fails, or Daraja returns an error response.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use daraja_sdk::mpesa::{ExpressError, MpesaExpress};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ExpressError> {
    /// let response = MpesaExpress::new()
    ///     .access_token("access-token")
    ///     .passkey("passkey")
    ///     .business_short_code(174379)
    ///     .party_a(254700000000)
    ///     .party_b(174379)
    ///     .phone_number(254700000000)
    ///     .amount(1)
    ///     .account_reference("Order123")
    ///     .tx_description("Payment")
    ///     .call_back_url("https://your-domain.com/callback")
    ///     .send_prompt()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_prompt(mut self) -> Result<StkPushResponse, ExpressError> {
        self.validate()?;

        let eat = FixedOffset::east_opt(EAT_OFFSET_SECS).expect("EAT offset is valid");
        let timestamp = Utc::now()
            .with_timezone(&eat)
            .format("%Y%m%d%H%M%S")
            .to_string();
        let password = BASE64_STANDARD.encode(format!(
            "{}{}{}",
            self.request_body.business_short_code, self.passkey, timestamp
        ));
        self.request_body.timestamp = timestamp;
        self.request_body.password = password;

        let response = self
            .http_client
            .post(self.get_url())
            .bearer_auth(&self.access_token)
            .json(&self.request_body)
            .send()
            .await
            .map_err(ExpressError::Request)?;

        if response.status().is_success() {
            return response
                .json::<StkPushResponse>()
                .await
                .map_err(ExpressError::Request);
        }

        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|err| format!("failed to read response body: {err}"));

        if let Ok(api_error) = serde_json::from_str::<DarajaErrorResponse>(&body) {
            return Err(ExpressError::Api {
                error_code: api_error.error_code,
                error_message: api_error.error_message,
            });
        }

        Err(ExpressError::UnexpectedResponse { status, body })
    }

    fn validate(&self) -> Result<(), ExpressError> {
        if self.access_token.is_empty() {
            return Err(ExpressError::Validation("access token is required".into()));
        }
        if self.passkey.is_empty() {
            return Err(ExpressError::Validation("passkey is required".into()));
        }
        if self.request_body.business_short_code == 0 {
            return Err(ExpressError::Validation(
                "business shortcode is required".into(),
            ));
        }
        if self.request_body.amount == 0 {
            return Err(ExpressError::Validation(
                "amount must be greater than zero".into(),
            ));
        }
        if self.request_body.party_a == 0 {
            return Err(ExpressError::Validation("party_a is required".into()));
        }
        if self.request_body.party_b == 0 {
            return Err(ExpressError::Validation("party_b is required".into()));
        }
        if self.request_body.phone_number == 0 {
            return Err(ExpressError::Validation("phone_number is required".into()));
        }
        if self.request_body.call_back_url.is_empty() {
            return Err(ExpressError::Validation("call_back_url is required".into()));
        }
        if url::Url::parse(&self.request_body.call_back_url).is_err() {
            return Err(ExpressError::Validation(
                "call_back_url must be a valid URL".into(),
            ));
        }
        if self.request_body.account_reference.is_empty() {
            return Err(ExpressError::Validation(
                "account_reference is required".into(),
            ));
        }
        if self.request_body.account_reference.len() > MAX_ACCOUNT_REFERENCE_LEN {
            return Err(ExpressError::Validation(format!(
                "account_reference must be at most {MAX_ACCOUNT_REFERENCE_LEN} characters"
            )));
        }
        if self.request_body.transaction_desc.len() > MAX_TRANSACTION_DESC_LEN {
            return Err(ExpressError::Validation(format!(
                "transaction description must be at most {MAX_TRANSACTION_DESC_LEN} characters"
            )));
        }

        Ok(())
    }
}

impl Default for MpesaExpress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mpesa::test_support::{TestConfig, get_access_token};

    #[test]
    fn transaction_type_serializes_as_plain_string() {
        let request = MpesaExpressRequest {
            transaction_type: TransactionType::CustomerPayBillOnline,
            ..Default::default()
        };

        let json = serde_json::to_string(&request).expect("request should serialize");
        assert!(json.contains(r#""TransactionType":"CustomerPayBillOnline""#));
        assert!(!json.contains("CustomerPayBillOnline\":null"));
    }

    #[test]
    fn request_serializes_callback_url_field() {
        let request = MpesaExpressRequest {
            call_back_url: "https://example.com/callback".into(),
            ..Default::default()
        };

        let json = serde_json::to_string(&request).expect("request should serialize");
        assert!(json.contains(r#""CallBackURL":"https://example.com/callback""#));
        assert!(!json.contains("CallBackUrl"));
    }

    #[test]
    fn stk_push_response_deserializes_id_fields() {
        let json = r#"{
            "MerchantRequestID": "29115-34620561-1",
            "CheckoutRequestID": "ws_CO_191220191020363925",
            "ResponseCode": "0",
            "ResponseDescription": "Success.",
            "CustomerMessage": "Success. Request accepted for processing"
        }"#;

        let response: StkPushResponse = serde_json::from_str(json).expect("response should parse");
        assert_eq!(response.merchant_request_id, "29115-34620561-1");
        assert_eq!(response.checkout_request_id, "ws_CO_191220191020363925");
    }

    #[tokio::test]
    async fn send_prompt_rejects_incomplete_builder() {
        let err = MpesaExpress::new()
            .send_prompt()
            .await
            .expect_err("incomplete builder should fail validation");

        assert!(matches!(err, ExpressError::Validation(_)));
    }

    #[tokio::test]
    async fn test_stk_push() {
        let test_config = TestConfig::load(false);
        let access_token = get_access_token().await;
        let response = MpesaExpress::new()
            .access_token(&access_token)
            .passkey(&test_config.passkey)
            .business_short_code(174379)
            .party_a(test_config.phone_number)
            .party_b(174379)
            .phone_number(test_config.phone_number)
            .amount(1)
            .account_reference("Sample")
            .tx_description("Something")
            .call_back_url(&test_config.callback_url)
            .send_prompt()
            .await
            .expect("STK Push request should succeed against sandbox");

        assert_eq!(response.response_code, "0");
    }
}
