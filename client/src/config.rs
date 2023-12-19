use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const SERVER_ADDRESS: &str = "127.0.0.1:44444";
pub const SERVER_PORT: u16 = 44444;

#[derive(Debug, Event, Serialize, Deserialize)]
pub enum MessageType {
    Connect,
    Disconnect,
    Nickname,
    Play,
    Chat,
    Game,
    Coordinate,
}

#[derive(Debug, Event, Serialize, Deserialize)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Event, Serialize, Deserialize)]
pub struct MessageToServer {
    pub message_type: MessageType,
    pub content: Option<String>,
    pub coordinate: Option<Coordinate>,
}

#[derive(Debug, Event, Serialize, Deserialize)]
pub struct MessageToClient {
    pub message_type: MessageType,
    pub content: Option<String>,
    pub coordinate: Option<Coordinate>,
}

#[derive(Debug, Event, Serialize, Deserialize, Component)]
pub struct Player {
    pub id: u64,
    pub nickname: Option<String>,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Event, Serialize, Deserialize, Component, Resource)]
pub struct AllPlayers(pub Vec<Player>);
