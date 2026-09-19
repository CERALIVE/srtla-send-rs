//! Request/reply control for the sender-owned pool, never a snapshot mutation.

use std::fmt;

use tokio::sync::{mpsc, oneshot};

use crate::bind_map::{LinkId, Priority};
use crate::connection::SrtlaConnection;

const CAPACITY: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkKey {
    LinkId(LinkId),
    /// Telemetry position, NOT the connection's internal random identifier.
    ConnId(usize),
}

#[derive(Clone, Debug)]
pub enum PoolControlRequest {
    SetLinkPriority {
        key: LinkKey,
        priority: Option<Priority>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolControlReply {
    pub applied: bool,
    pub key: LinkKey,
    pub link_id: Option<LinkId>,
    pub conn_id: usize,
    pub priority: Option<Priority>,
    pub effective_priority: Option<Priority>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PoolControlError {
    UnknownLink(LinkKey),
    Unavailable,
    Busy,
    PoolReloaded,
}

impl fmt::Display for PoolControlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownLink(key) => write!(f, "unknown link: {key:?}"),
            Self::Unavailable => f.write_str("sender pool control unavailable"),
            Self::Busy => f.write_str("sender pool control queue full"),
            Self::PoolReloaded => f.write_str("sender pool reloaded before application"),
        }
    }
}

impl std::error::Error for PoolControlError {}

pub type PoolControlResult = Result<PoolControlReply, PoolControlError>;

#[derive(Clone)]
pub struct PoolControlHandle {
    tx: mpsc::Sender<PoolControlMessage>,
}

impl PoolControlHandle {
    /// Enqueue without blocking either a Tokio task or the synchronous RPC thread.
    /// Success here is NOT application: await the returned reply before reporting it.
    /// Callers own their reply deadline; dropping the receiver cancels queued work.
    pub fn submit(
        &self,
        request: PoolControlRequest,
    ) -> Result<oneshot::Receiver<PoolControlResult>, PoolControlError> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .try_send(PoolControlMessage { request, reply })
            .map_err(|err| match err {
                mpsc::error::TrySendError::Full(_) => PoolControlError::Busy,
                mpsc::error::TrySendError::Closed(_) => PoolControlError::Unavailable,
            })?;
        Ok(rx)
    }
}

pub(crate) struct PoolControlReceiver {
    rx: mpsc::Receiver<PoolControlMessage>,
}

impl PoolControlReceiver {
    pub(crate) fn channel() -> (PoolControlHandle, Self) {
        let (tx, rx) = mpsc::channel(CAPACITY);
        (PoolControlHandle { tx }, Self { rx })
    }

    pub(crate) async fn recv(&mut self) -> Option<PoolControlMessage> {
        self.rx.recv().await
    }

    /// Close BEFORE the reload's first await; old handles can never address a new pool.
    pub(crate) fn close_for_reload(&mut self) {
        self.rx.close();
        while let Ok(message) = self.rx.try_recv() {
            // A dropped reply means its caller cancelled; nothing remains to notify.
            let _ = message.reply.send(Err(PoolControlError::PoolReloaded));
        }
    }
}

pub(crate) struct PoolControlMessage {
    request: PoolControlRequest,
    reply: oneshot::Sender<PoolControlResult>,
}

impl PoolControlMessage {
    pub(crate) fn apply(self, connections: &mut [SrtlaConnection]) {
        if self.reply.is_closed() {
            return;
        }
        let result = match self.request {
            PoolControlRequest::SetLinkPriority { key, priority } => {
                apply_priority(connections, key, priority)
            }
        };
        // Application is synchronous on the pool owner; a concurrent cancellation
        // after application can discard the reply, but cannot undo the operation.
        let _ = self.reply.send(result);
    }
}

fn apply_priority(
    connections: &mut [SrtlaConnection],
    key: LinkKey,
    priority: Option<Priority>,
) -> PoolControlResult {
    let index = match &key {
        LinkKey::LinkId(id) => connections
            .iter()
            .position(|conn| conn.link_id.as_ref() == Some(id)),
        LinkKey::ConnId(index) => (*index < connections.len()).then_some(*index),
    }
    .ok_or_else(|| PoolControlError::UnknownLink(key.clone()))?;
    let conn = &mut connections[index];
    match (&key, priority) {
        (LinkKey::LinkId(_), Some(value)) => conn.priority_override_link = Some(value),
        (LinkKey::LinkId(_), None) => conn.clear_priority_override_link(),
        (LinkKey::ConnId(_), Some(value)) => conn.priority_override_conn = Some(value),
        (LinkKey::ConnId(_), None) => conn.clear_priority_override_conn(),
    }
    Ok(PoolControlReply {
        applied: true,
        key,
        link_id: conn.link_id.clone(),
        conn_id: index,
        priority,
        effective_priority: conn.effective_priority(),
    })
}
