use dptree::case;
use proc_qq::{GroupMessageEvent, MessageChainParseTrait, MessageSendToSourceTrait};
use tracing::error;

use crate::dp_helper::{EVHandler, UpdateKind};

pub fn must_admin() -> EVHandler {
    case![UpdateKind::GroupMessage].filter_async(|ev: GroupMessageEvent| async move {
        let client = ev.client.clone();
        let group = ev.inner.group_code;
        let sender = ev.inner.from_uin;
        match client.get_group_admin_list(group).await {
            Ok(admins) => admins.contains_key(&sender),
            Err(e) => {
                error!(group, ?e, "failed to get admin list of group");
                drop(
                    ev.send_message_to_source("Failed to authenticate user.".parse_message_chain())
                        .await,
                );
                false
            }
        }
    })
}
