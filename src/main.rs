#![allow(clippy::module_name_repetitions)]

use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use once_cell::sync::OnceCell;
use proc_qq::re_exports::bytes::Bytes;
use proc_qq::re_exports::ricq::version::ANDROID_WATCH;
use proc_qq::Authentication::QRCode;
use proc_qq::DeviceSource::JsonFile;
use proc_qq::{ClientBuilder, ShowQR};
use tracing::{debug, info};

use crate::config::Config;
use crate::db::DB;
use crate::handlers::auth::{Token, OTP};
use crate::handlers::handler;

mod config;
mod db;
mod dp_helper;
mod handlers;

static CONFIG: OnceCell<Config> = OnceCell::new();

fn mail_qrcode(f: Bytes) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
    let client = reqwest::Client::new();
    let config = CONFIG.get().cloned().unwrap().qrcode.unwrap();
    let form = reqwest::multipart::Form::new()
        .text("from", format!("IM Bridge <mailgun@{}>", config.domain))
        .text("to", config.to)
        .text("subject", "IM Bridge - QR Code")
        .text("text", "Please scan the qr code to login")
        .part(
            "attachment",
            reqwest::multipart::Part::bytes(f.to_vec()).file_name("qrcode.png"),
        );
    Box::pin(async move {
        let resp = client
            .post(&format!(
                "https://api.mailgun.net/v3/{}/messages",
                config.domain
            ))
            .basic_auth("api", Some(&config.apikey))
            .multipart(form)
            .send()
            .await?
            .text()
            .await?;
        info!(resp, "QR code sent");
        Ok(())
    })
}

fn qr_method() -> ShowQR {
    let by_mail = CONFIG.get().unwrap().qrcode.is_some();
    if by_mail {
        ShowQR::Custom(Box::pin(mail_qrcode))
    } else {
        ShowQR::PrintToConsole
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::from_env();
    debug!(?config, "config loaded");
    CONFIG.set(config.clone()).unwrap();

    let token = Token::default();
    let otp = OTP::default();
    let db = DB::connect(&config.mongodb.uri, &config.mongodb.database).await?;
    info!("Manage token: {}", token);
    let client = ClientBuilder::new()
        .priority_session(
            std::env::var("SESSION_FILE").unwrap_or_else(|_| "session.token".to_string()),
        )
        .authentication(QRCode)
        .device(JsonFile(
            std::env::var("DEVICE_FILE").unwrap_or_else(|_| "device.json".to_string()),
        ))
        .version(&ANDROID_WATCH)
        .modules(vec![dp_helper::module(
            dptree::deps![token, otp, db],
            handler(),
        )])
        .show_rq(Some(qr_method()))
        .build()
        .await?;
    client.start().await??;
    Ok(())
}
