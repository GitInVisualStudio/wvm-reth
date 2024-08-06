use super::protocol::proto::{CustomRlpxProtoMessage, CustomRlpxProtoMessageKind};
use futures::{Stream, StreamExt};
use reth_eth_wire::multiplex::ProtocolConnection;
use reth_primitives::BytesMut;
use std::{
    pin::Pin,
    task::{ready, Context, Poll},
};
use tokio::sync::oneshot;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub(crate) mod handler;

/// We define some custom commands that the subprotocol supports.
pub(crate) enum CustomCommand {
    /// Sends a message to the peer
    Message {
        msg: String,
        /// The response will be sent to this channel.
        response: oneshot::Sender<String>,
    },
}

/// The connection handler for the custom RLPx protocol.
pub(crate) struct CustomRlpxConnection {
    conn: ProtocolConnection,
    initial_ping: Option<CustomRlpxProtoMessage>,
    commands: UnboundedReceiverStream<CustomCommand>,
    pending_pong: Option<oneshot::Sender<String>>,
}

impl Stream for CustomRlpxConnection {
    type Item = BytesMut;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if let Some(initial_ping) = this.initial_ping.take() {
<<<<<<< HEAD
            return Poll::Ready(Some(initial_ping.encoded()));
=======
            return Poll::Ready(Some(initial_ping.encoded()))
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
        }

        loop {
            if let Poll::Ready(Some(cmd)) = this.commands.poll_next_unpin(cx) {
                return match cmd {
                    CustomCommand::Message { msg, response } => {
                        this.pending_pong = Some(response);
                        Poll::Ready(Some(CustomRlpxProtoMessage::ping_message(msg).encoded()))
                    }
<<<<<<< HEAD
                };
=======
                }
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
            }

            let Some(msg) = ready!(this.conn.poll_next_unpin(cx)) else { return Poll::Ready(None) };

            let Some(msg) = CustomRlpxProtoMessage::decode_message(&mut &msg[..]) else {
<<<<<<< HEAD
                return Poll::Ready(None);
=======
                return Poll::Ready(None)
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
            };

            match msg.message {
                CustomRlpxProtoMessageKind::Ping => {
                    return Poll::Ready(Some(CustomRlpxProtoMessage::pong().encoded()))
                }
                CustomRlpxProtoMessageKind::Pong => {}
                CustomRlpxProtoMessageKind::PingMessage(msg) => {
                    return Poll::Ready(Some(CustomRlpxProtoMessage::pong_message(msg).encoded()))
                }
                CustomRlpxProtoMessageKind::PongMessage(msg) => {
                    if let Some(sender) = this.pending_pong.take() {
                        sender.send(msg).ok();
                    }
<<<<<<< HEAD
                    continue;
                }
            }

            return Poll::Pending;
=======
                    continue
                }
            }

            return Poll::Pending
>>>>>>> c4b5f5e9c9a88783b2def3ab1cc880b8d41867e1
        }
    }
}
