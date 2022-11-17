use dptree::case;
use proc_qq::NewFriendRequestEvent;
use tracing::info;

use crate::dp_helper::{EVHandler, UpdateKind};
use crate::handlers::auth::Token;

pub fn new_friend_handler() -> EVHandler {
    case![UpdateKind::NewFriendRequest]
        .filter(|token: Token, ev: NewFriendRequestEvent| ev.inner.message.contains(&*token))
        .endpoint(|ev: NewFriendRequestEvent| async move {
            info!(
                uid = ev.inner.req_uin,
                nick = ev.inner.req_nick,
                "Accepting new friend request"
            );
            Ok(ev.accept().await?)
        })
}
