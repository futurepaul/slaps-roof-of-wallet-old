use lnd_rust::macaroon_data::MacaroonData;
use lnd_rust::rpc;
use lnd_rust::rpc_grpc::Lightning;
use lnd_rust::rpc_grpc::LightningClient;
use lnd_rust::tls_certificate::TLSCertificate;
use std::{net::SocketAddr, sync::Arc};

use grpc::ClientStub;
use grpc::RequestOptions;

use failure::{Fail, Error};

#[derive(Fail, Debug)]
pub enum LndError {
    #[fail(display = "{}", _0)]
    Io(#[cause] std::io::Error),
    #[fail(display = "Problem with reading env: {}", _0)]
    EnvError(String),
    #[fail(display = "Problem with gathering credentials: {}", _0)]
    CredentialsError(String),
    #[fail(display = "Problem with creating client: {}", _0)]
    ClientCreationError(String),
    #[fail(display = "Problem with GRPC request: {}", _0)]
    GrpcError(String)

}

impl From<std::io::Error> for LndError {
    fn from(err: std::io::Error) -> LndError {
        LndError::Io(err)
    }
}

impl From<std::net::AddrParseError> for LndError {
    fn from(err: std::net::AddrParseError) -> LndError {
        LndError::CredentialsError(err.to_string())
    }
}

impl From<grpc::Error> for LndError {
    fn from(err: grpc::Error) -> LndError {
        LndError::GrpcError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, LndError>;

fn env_or_bust(var: &str) -> Result<String> {
    std::env::var(var).or_else(|_| Err(LndError::EnvError(format!("Environment variable {} not found", var))))
}

pub struct Credentials {
    certificate: TLSCertificate,
    invoice_macaroon: Option<MacaroonData>,
    readonly_macaroon: Option<MacaroonData>,
    socket_addr: SocketAddr,
}

impl Credentials {
    pub fn read_from_env() -> Result<Credentials> {
        let certificate = {
            let cert_filename = env_or_bust("LND_TLS_CERT")?;
            TLSCertificate::from_path(cert_filename)?
        };

        let invoice_macaroon = {
            let macaroon_file_path = env_or_bust("LND_INVOICE_MACAROON")?;
            MacaroonData::from_file_path(macaroon_file_path)?
        };

        let readonly_macaroon = {
            let macaroon_file_path = env_or_bust("LND_READ_MACAROON")?;
            MacaroonData::from_file_path(macaroon_file_path)?
        };

        let socket_addr: SocketAddr = {
            let url = env_or_bust("LND_GRPC_URL")?;
            url.parse()?
        };

        Ok(Credentials {
            certificate: certificate,
            invoice_macaroon: Some(invoice_macaroon),
            readonly_macaroon: Some(readonly_macaroon),
            socket_addr: socket_addr,
        })
    }
}

pub fn new_client(creds: Credentials) -> Result<LightningClient> {
    let host = creds.socket_addr.ip().to_string();
    let conf = Default::default();

    // Can't convert this tls error to failure because it's private so...
    let tls = match creds.certificate.into_tls(host.as_str()) {
        Ok(tls) => tls,
        Err(err) => return Err(LndError::ClientCreationError(err.to_string()))
    };

    let client = grpc::Client::new_expl(&creds.socket_addr, host.as_str(), tls, conf)?;

    Ok(LightningClient::with_client(Arc::new(client)))
}

pub fn get_wallet_balance(creds: &Credentials, client: &LightningClient) -> Result<i64> {
    let wallet_req = rpc::WalletBalanceRequest::new();

    if let Some(mac) = &creds.readonly_macaroon {
        let wallet_resp = client.wallet_balance(
            RequestOptions {
                metadata: mac.metadata(),
            },
            wallet_req,
        );
        let w = wallet_resp.wait()?;

        //total_balance = confirmed_balance + unconfirmed_balance
        return Ok(w.1.get_total_balance())
    } else {
        return Err(LndError::GrpcError("Get wallet balance failed".to_string()))
    }
}

pub fn get_channel_balance(creds: &Credentials, client: &LightningClient) -> Result<i64> {
    let channel_req = rpc::ChannelBalanceRequest::new();

    if let Some(mac) = &creds.readonly_macaroon {
        let channel_resp = client.channel_balance(
            RequestOptions {
                metadata: mac.metadata(),
            },
            channel_req,
        );
        let c = channel_resp.wait()?;

        //sum of channels balance
        //look up pending like this: pending_open_balance
        return Ok(c.1.get_balance());
    } else {
        return Err(LndError::GrpcError("Get channel balance failed".to_string()));
    }
}

//Even though we have an invoice macaroon, we still need to use the readonly for this (not sure why!)
pub fn get_info(creds: &Credentials, client: &LightningClient) -> Result<lnd_rust::rpc::GetInfoResponse> {
    let req = rpc::GetInfoRequest::new();
    if let Some(mac) = &creds.readonly_macaroon {
        let resp = client.get_info(
            RequestOptions {
                metadata: mac.metadata(),
            },
            req,
        );
        return Ok(resp.wait().unwrap().1);
    } else {
        return Err(LndError::GrpcError("Get info failed".to_string()))
    }
}

pub fn create_invoice(amount: u32, memo: String, creds: &Credentials, client: &LightningClient) -> Result<lnd_rust::rpc::AddInvoiceResponse> {
    let mut invoice = rpc::Invoice::default();
    invoice.memo = memo;
    invoice.value = amount as i64;
    invoice.expiry = 120; //(2 minutes);

    if let Some(mac) = &creds.invoice_macaroon {
        let resp = client.add_invoice(
            RequestOptions {
                metadata: mac.metadata(),
            },
            invoice,
        );
        return Ok(resp.wait().unwrap().1);
    } else {
        return Err(LndError::GrpcError("Add invoice failed".to_string()))
    }

}

// //Here's where we construct our invoice, basing it off a default invoice (there are a ton more fields)
// let mut invoice = rpc::Invoice::default();
// invoice.memo = "Woah we're really doing this!".to_string();
// invoice.value = 100; // sats
// invoice.expiry = 120; //(2 minutes);
// let invoice_resp = client.add_invoice(metadata(&invoice_macaroon_data), invoice);
// dbg!(invoice_resp.wait().unwrap());

//Using the readonly macaroon again
