use lnd_rust::macaroon_data::MacaroonData;
use lnd_rust::rpc;
use lnd_rust::rpc_grpc::Lightning;
use lnd_rust::rpc_grpc::LightningClient;
use lnd_rust::tls_certificate::TLSCertificate;
use std::{net::SocketAddr, sync::Arc};

use grpc::ClientStub;
use grpc::RequestOptions;

fn env_or_bust(var: &str) -> String {
    std::env::var(var).expect(&format!("Environment variable {} not found", var))
}

pub struct Credentials {
    certificate: TLSCertificate,
    invoice_macaroon: Option<MacaroonData>,
    readonly_macaroon: Option<MacaroonData>,
    socket_addr: SocketAddr,
}

impl Credentials {
    pub fn read_from_env() -> Credentials {
        let certificate = {
            let cert_filename = env_or_bust("LND_TLS_CERT");
            TLSCertificate::from_path(cert_filename).unwrap()
        };

        let invoice_macaroon = {
            let macaroon_file_path = env_or_bust("LND_INVOICE_MACAROON");
            MacaroonData::from_file_path(macaroon_file_path).unwrap()
        };

        let readonly_macaroon = {
            let macaroon_file_path = env_or_bust("LND_READ_MACAROON");
            MacaroonData::from_file_path(macaroon_file_path).unwrap()
        };

        let socket_addr: SocketAddr = {
            let url = env_or_bust("LND_GRPC_URL");
            url.parse().unwrap()
        };

        Credentials {
            certificate: certificate,
            invoice_macaroon: Some(invoice_macaroon),
            readonly_macaroon: Some(readonly_macaroon),
            socket_addr: socket_addr,
        }
    }
}

pub fn new_client(creds: Credentials) -> LightningClient {
    let host = creds.socket_addr.ip().to_string();
    let conf = Default::default();

    let tls = creds.certificate.into_tls(host.as_str()).unwrap();
    let c = grpc::Client::new_expl(&creds.socket_addr, host.as_str(), tls, conf).unwrap();
    LightningClient::with_client(Arc::new(c))
}
// let client =

//We don't need to use this closure to create the RequestOptions object, but it looks nicer so...
// let metadata = |macaroon_data: &MacaroonData|
//     RequestOptions { metadata: macaroon_data.metadata(), };

pub fn get_wallet_balance(creds: &Credentials, client: &LightningClient) -> i64 {
    let wallet_req = rpc::WalletBalanceRequest::new();

    if let Some(mac) = &creds.readonly_macaroon {
        let wallet_resp = client.wallet_balance(
            RequestOptions {
                metadata: mac.metadata(),
            },
            wallet_req,
        );
        let w = wallet_resp.wait().unwrap();

        //total_balance = confirmed_balance + unconfirmed_balance
        return w.1.get_total_balance();
    } else {
        panic!("No readonly.macaroon found!");
    }
}

pub fn get_channel_balance(creds: &Credentials, client: &LightningClient) -> i64 {
    let channel_req = rpc::ChannelBalanceRequest::new();

    if let Some(mac) = &creds.readonly_macaroon {
        let channel_resp = client.channel_balance(
            RequestOptions {
                metadata: mac.metadata(),
            },
            channel_req,
        );
        let c = channel_resp.wait().unwrap();

        //sum of channels balance
        //look up pending like this: pending_open_balance
        return c.1.get_balance();
    } else {
        panic!("No readonly.macaroon found!");
    }
}

//Even though we have an invoice macaroon, we still need to use the readonly for this (not sure why!)
pub fn get_info(creds: &Credentials, client: &LightningClient) -> lnd_rust::rpc::GetInfoResponse {
    let req = rpc::GetInfoRequest::new();
    if let Some(mac) = &creds.readonly_macaroon {
        let resp = client.get_info(
            RequestOptions {
                metadata: mac.metadata(),
            },
            req,
        );
        return resp.wait().unwrap().1;
    } else {
        panic!("No readonly.macaroon found!")
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
