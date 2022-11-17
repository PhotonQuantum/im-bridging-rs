use dptree::case;
use itertools::Itertools;
use proc_qq::{FriendMessageEvent, MessageChainParseTrait, MessageSendToSourceTrait};
use tracing::{info, warn};

use crate::db::DB;
use crate::dp_helper::EVHandler;
use crate::dp_helper::UpdateKind;
use crate::handlers::auth::token_auth;
use crate::handlers::parser::ClusterCommand;

pub fn cluster_handler() -> EVHandler {
    dptree::entry().branch(
        case![UpdateKind::FriendMessage].chain(token_auth(
            dptree::entry()
                .branch(case![ClusterCommand::List].endpoint(
                    |db: DB, ev: FriendMessageEvent| async move {
                        let clusters = db.clusters().await?.join("\n");
                        ev.send_message_to_source(
                            format!("Available clusters:\n{}", clusters).parse_message_chain(),
                        )
                        .await?;
                        Ok(())
                    },
                ))
                .branch(case![ClusterCommand::Add].endpoint(
                    |db: DB, ev: FriendMessageEvent| async move {
                        let msg = match db.new_cluster().await {
                            Ok(name) => {
                                info!(name, "new cluster created");
                                format!("New cluster created: {}", name)
                            }
                            Err(e) => {
                                warn!(?e, "failed to create new cluster");
                                "Failed to create cluster. Please try again later.".into()
                            }
                        };
                        ev.send_message_to_source(msg.parse_message_chain()).await?;
                        Ok(())
                    },
                )),
        )),
    )
    // TODO delete cluster
}
