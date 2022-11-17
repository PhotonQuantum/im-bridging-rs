use std::iter;

use clap::{Parser, Subcommand};
use dptree::case;
use futures::future::ok;
use proc_qq::{FriendMessageEvent, GroupMessageEvent, MessageContentTrait};

use crate::dp_helper::{EVHandler, UpdateKind};
use crate::handlers::auth::Given;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Cluster {
        #[command(subcommand)]
        cmd: ClusterCommand,
        #[arg(short, long)]
        token: Given,
    },
    RequestOTP {
        #[arg(short, long)]
        token: Given,
    },
    Join {
        cluster: String,
        #[arg(short, long)]
        otp: Given,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ClusterCommand {
    Add,
    List,
}

pub fn parse_cmd(ev: EVHandler) -> EVHandler {
    #[derive(Debug, Clone)]
    struct Input(String);

    let parsed =
        dptree::filter_map(|Input(input)| input.strip_prefix('/').map(|s| Input(s.to_string())))
            .filter_map(|Input(input)| {
                let split = shellwords::split(&input).ok()?;
                Args::try_parse_from(iter::once("im-bridge".to_string()).chain(split))
                    .map_err(|e| e.to_string())
                    .ok()
            })
            .map(|args: Args| args.cmd)
            .chain(ev);
    dptree::entry()
        .branch(
            case![UpdateKind::FriendMessage]
                .map(|ev: FriendMessageEvent| Input(ev.message_content()))
                .chain(parsed.clone()),
        )
        .branch(
            case![UpdateKind::GroupMessage]
                .map(|ev: GroupMessageEvent| Input(ev.message_content()))
                .chain(parsed),
        )
}
