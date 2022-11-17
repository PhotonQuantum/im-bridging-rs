use dptree::case;
use proc_qq::{
    FriendMessageEvent, GroupMessageEvent, MessageChainParseTrait, MessageSendToSourceTrait,
};
use tracing::{info, warn};

use crate::db::{Group, DB};
use crate::dp_helper::EVHandler;
use crate::dp_helper::UpdateKind;
use crate::handlers::auth::{otp_auth, token_auth, OTP};
use crate::handlers::guard::must_admin;

pub fn request_otp_handler() -> EVHandler {
    case![UpdateKind::FriendMessage].chain(token_auth(dptree::endpoint(
        |ev: FriendMessageEvent, otp: OTP| async move {
            let pass = otp.generate_new();
            ev.send_message_to_source(
                format!("Your one-time password is:\n{}", pass).parse_message_chain(),
            )
            .await?;
            Ok(())
        },
    )))
}

pub fn join_handler() -> EVHandler {
    case![UpdateKind::GroupMessage]
        .chain(must_admin())
        .chain(otp_auth(dptree::endpoint(
            // TODO earlier: extract join name as string
            |db: DB, cluster: String, ev: GroupMessageEvent| async move {
                let group = Group::from_qq(ev.inner.group_code);
                let msg = match db.join(&cluster, &group).await {
                    Ok(_) => {
                        info!(?group, cluster, "group joined cluster");
                        "Joined to cluster"
                    }
                    Err(e) => {
                        warn!(?e, "failed to create new cluster");
                        "Failed to join cluster. Please try again later."
                    }
                };
                ev.send_message_to_source(msg.parse_message_chain()).await?;
                Ok(())
            },
        )))
}

// TODO leave cluster
