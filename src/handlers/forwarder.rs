use std::sync::Arc;

use anyhow::Result;
use dptree::case;
use proc_qq::re_exports::ricq;
use proc_qq::re_exports::ricq::msg::elem::RQElem;
use proc_qq::re_exports::ricq::structs::GroupMessage;
use proc_qq::{GroupMessageEvent, MessageChainParseTrait};
use tracing::error;

use crate::db::{Group, DB, IM};
use crate::dp_helper::{EVHandler, UpdateKind};

pub fn forwarder() -> EVHandler {
    case![UpdateKind::GroupMessage].endpoint(|db: DB, ev: GroupMessageEvent| async move {
        let group = ev.inner.group_code;
        let client = ev.client;
        let targets = db.forward_targets(&Group::from_qq(group)).await?;
        // TODO should have better logic separation (e.g. a special object for unified tg/qq forward)
        for target in targets {
            let client = client.clone();
            let msg = ev.inner.clone();
            tokio::spawn(async move {
                if let Err(e) = forward(client, msg, target.clone()).await {
                    error!(?e, ?group, "failed to forward message");
                }
            });
        }
        Ok(())
    })
}

async fn forward(client: Arc<ricq::Client>, msg: GroupMessage, group: Group) -> Result<()> {
    let Group { im, id } = group;
    let sender = client
        .get_group_member_info(msg.group_code, msg.from_uin)
        .await?;
    let sender_display = if sender.card_name.is_empty() {
        sender.nickname
    } else {
        format!("{} ({})", sender.card_name, sender.nickname)
    };
    // TODO extract parse logic out of join
    match im {
        IM::QQ => {
            let target_id: i64 = id.parse()?;
            let mut new_msg = format!("{}: ", sender_display).parse_message_chain();
            for elem in msg.elements.0 {
                let parsed = RQElem::from(elem);
                match parsed {
                    RQElem::At(x) => new_msg.push(x),
                    RQElem::Text(x) => new_msg.push(x),
                    RQElem::Face(x) => new_msg.push(x),
                    RQElem::MarketFace(x) => new_msg.push(x),
                    RQElem::Dice(x) => new_msg.push(x),
                    RQElem::FingerGuessing(x) => new_msg.push(x),
                    RQElem::LightApp(x) => new_msg.push(x),
                    RQElem::RichMsg(x) => new_msg.push(x),
                    RQElem::FriendImage(x) => new_msg.push(x),
                    RQElem::GroupImage(x) => new_msg.push(x),
                    RQElem::FlashImage(x) => new_msg.push(x),
                    RQElem::VideoFile(x) => new_msg.push(x),
                    _ => continue,
                }
            }
            // TODO recall
            client.send_group_message(target_id, new_msg).await?;
        }
    };
    Ok(())
}
