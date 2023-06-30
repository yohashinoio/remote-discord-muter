use tokio::sync::oneshot;

#[derive(Debug)]
pub enum RequestKind {
    Mute,
    Unmute,
    GetMuteSetting { resp: oneshot::Sender<bool> },
}

#[derive(Debug)]
pub enum MuteKind {
    Muted,
    Unmuted,
}
