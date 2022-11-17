use dptree::case;

use crate::dp_helper::EVHandler;
use crate::handlers::admin::{join_handler, request_otp_handler};
use crate::handlers::auth::Given;
use crate::handlers::cluster::cluster_handler;
use crate::handlers::forwarder::forwarder;
use crate::handlers::new_friend::new_friend_handler;
use crate::handlers::parser::{parse_cmd, ClusterCommand, Command};

mod admin;
pub mod auth;
mod cluster;
mod forwarder;
mod guard;
mod new_friend;
mod parser;

pub fn handler() -> EVHandler {
    dptree::entry()
        .branch(new_friend_handler())
        .branch(
            dptree::entry().chain(parse_cmd(
                dptree::entry()
                    .branch(case![Command::RequestOTP { token }].chain(request_otp_handler()))
                    .branch(
                        case![Command::Cluster { cmd, token }]
                            .map(|(cmd, _): (ClusterCommand, Given)| cmd)
                            .map(|(_, given): (ClusterCommand, Given)| given)
                            .chain(cluster_handler()),
                    )
                    .branch(
                        case![Command::Join { cluster, otp }]
                            .map(|(cluster, _): (String, Given)| cluster)
                            .map(|(_, given): (String, Given)| given)
                            .chain(join_handler()),
                    ),
            )),
        )
        .branch(forwarder())
}
