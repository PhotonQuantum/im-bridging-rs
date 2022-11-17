use std::ops::ControlFlow;

use anyhow::Result;
use async_trait::async_trait;
use dptree::di::DependencyMap;
use dptree::Endpoint;
use proc_qq::{
    GroupMessageRecallEvent, GroupMessageRecallEventProcess, MessageEvent, MessageEventProcess,
    Module, ModuleEventHandler, ModuleEventProcess, NewFriendRequestEvent,
    NewFriendRequestEventProcess,
};

pub type EVHandler = Endpoint<'static, DependencyMap, Result<()>>;

struct EventCollector {
    dp: DependencyMap,
    handler: EVHandler,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum UpdateKind {
    GroupMessage,
    GroupMessageRecall,
    FriendMessage,
    GroupTempMessage,
    NewFriendRequest,
}

#[async_trait]
impl MessageEventProcess for EventCollector {
    async fn handle(&self, event: &MessageEvent) -> Result<bool> {
        let mut dmap = DependencyMap::new();
        match event {
            MessageEvent::GroupMessage(msg) => {
                dmap.insert(UpdateKind::GroupMessage);
                dmap.insert(msg.clone());
            }
            MessageEvent::FriendMessage(msg) => {
                dmap.insert(UpdateKind::FriendMessage);
                dmap.insert(msg.clone());
            }
            MessageEvent::GroupTempMessage(msg) => {
                dmap.insert(UpdateKind::GroupTempMessage);
                dmap.insert(msg.clone());
            }
        }
        dmap.insert_container(self.dp.clone());
        if let ControlFlow::Break(b) = self.handler.dispatch(dmap).await {
            b?;
        }
        Ok(false)
    }
}

#[async_trait]
impl NewFriendRequestEventProcess for EventCollector {
    async fn handle(&self, event: &NewFriendRequestEvent) -> Result<bool> {
        let mut dmap = DependencyMap::new();
        dmap.insert(UpdateKind::NewFriendRequest);
        dmap.insert(event.clone());
        dmap.insert_container(self.dp.clone());
        if let ControlFlow::Break(b) = self.handler.dispatch(dmap).await {
            b?;
        }
        Ok(false)
    }
}

#[async_trait]
impl GroupMessageRecallEventProcess for EventCollector {
    async fn handle(&self, event: &GroupMessageRecallEvent) -> Result<bool> {
        let mut dmap = DependencyMap::new();
        dmap.insert(UpdateKind::GroupMessageRecall);
        dmap.insert(event.clone());
        dmap.insert_container(self.dp.clone());
        if let ControlFlow::Break(b) = self.handler.dispatch(dmap).await {
            b?;
        }
        Ok(false)
    }
}

pub fn module(dp: DependencyMap, handler: EVHandler) -> Module {
    let on_message = ModuleEventHandler {
        name: "EventCollector".to_string(),
        process: ModuleEventProcess::Message(Box::new(EventCollector {
            dp: dp.clone(),
            handler: handler.clone(),
        })),
    };
    let on_new_friend = ModuleEventHandler {
        name: "EventCollector".to_string(),
        process: ModuleEventProcess::NewFriendRequest(Box::new(EventCollector {
            dp: dp.clone(),
            handler: handler.clone(),
        })),
    };
    let on_recall = ModuleEventHandler {
        name: "EventCollector".to_string(),
        process: ModuleEventProcess::GroupMessageRecall(Box::new(EventCollector { dp, handler })),
    };
    Module {
        id: "dp_handler".to_string(),
        name: "DI Adaptor".to_string(),
        handles: vec![on_message, on_new_friend, on_recall],
    }
}
