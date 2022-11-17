use std::convert::Infallible;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use chbs::config::BasicConfig;
use chbs::probability::Probability;
use chbs::scheme::ToScheme;
use chbs::word::{WordList, WordSampler};
use dashmap::DashSet;
use dptree::case;
use internment::Intern;
use proc_qq::{
    FriendMessageEvent, GroupMessageEvent, MessageChainParseTrait, MessageSendToSourceTrait,
};

use crate::dp_helper::{EVHandler, UpdateKind};

#[derive(Debug, Clone, Default)]
pub struct OTP {
    otps: Arc<DashSet<String>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Token(Intern<String>);

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for Token {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Default for Token {
    fn default() -> Self {
        Self(Intern::new(random_pass()))
    }
}

pub fn random_pass() -> String {
    let config: BasicConfig<WordSampler> = chbs::config::BasicConfigBuilder::default()
        .word_provider(WordList::builtin_eff_short().sampler())
        .capitalize_words(Probability::Never)
        .capitalize_first(Probability::Never)
        .separator("-")
        .words(5usize)
        .build()
        .unwrap();
    let scheme = config.to_scheme();
    scheme.generate()
}

impl OTP {
    pub fn generate_new(&self) -> String {
        let pass = random_pass();
        self.otps.insert(pass.clone());
        pass
    }
    pub fn verify(&self, pass: &str) -> bool {
        self.otps.remove(pass).is_some()
    }
}

#[derive(Debug, Clone)]
pub struct Given(String);

impl FromStr for Given {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

pub fn token_auth(authed: EVHandler) -> EVHandler {
    dptree::entry()
        .branch(
            case![UpdateKind::FriendMessage]
                .filter_async(
                    |Given(given), token: Token, ev: FriendMessageEvent| async move {
                        if given == token.as_ref() {
                            true
                        } else {
                            drop(
                                ev.send_message_to_source("Invalid token".parse_message_chain())
                                    .await,
                            );
                            false
                        }
                    },
                )
                .chain(authed.clone()),
        )
        .branch(
            case![UpdateKind::GroupMessage]
                .filter_async(
                    |Given(given), token: Token, ev: GroupMessageEvent| async move {
                        if given == token.as_ref() {
                            true
                        } else {
                            drop(
                                ev.send_message_to_source("Invalid token".parse_message_chain())
                                    .await,
                            );
                            false
                        }
                    },
                )
                .chain(authed),
        )
}

pub fn otp_auth(authed: EVHandler) -> EVHandler {
    dptree::entry()
        .branch(
            case![UpdateKind::FriendMessage]
                .filter_async(
                    |Given(given), otp: OTP, ev: FriendMessageEvent| async move {
                        if otp.verify(&given) {
                            true
                        } else {
                            drop(
                                ev.send_message_to_source(
                                    "Invalid one-time password".parse_message_chain(),
                                )
                                .await,
                            );
                            false
                        }
                    },
                )
                .chain(authed.clone()),
        )
        .branch(
            case![UpdateKind::GroupMessage]
                .filter_async(|Given(given), otp: OTP, ev: GroupMessageEvent| async move {
                    if otp.verify(&given) {
                        true
                    } else {
                        drop(
                            ev.send_message_to_source(
                                "Invalid one-time password".parse_message_chain(),
                            )
                            .await,
                        );
                        false
                    }
                })
                .chain(authed),
        )
}
