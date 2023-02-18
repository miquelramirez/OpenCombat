use serde_derive::{Deserialize, Serialize};

use crate::{
    state::{battle::message::BattleStateMessage, client::ClientStateMessage},
    sync::BattleStateCopy,
};

use self::network::NetworkMessage;

pub mod network;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    Input(InputMessage),
    Output(OutputMessage),
    Network(NetworkMessage),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InputMessage {
    RequireCompleteSync,
    BattleState(BattleStateMessage),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OutputMessage {
    LoadFromCopy(BattleStateCopy),
    BattleState(BattleStateMessage),
    ClientState(ClientStateMessage),
}