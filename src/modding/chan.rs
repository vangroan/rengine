use super::cmd::ModCmd;
use crossbeam::{
    channel,
    channel::{RecvError, SendError},
};

#[derive(Clone)]
pub struct ChannelPair {
    sender: channel::Sender<Vec<ModCmd>>,
    receiver: channel::Receiver<Vec<ModCmd>>,
}

pub type ModCmdChannel = (channel::Sender<Vec<ModCmd>>, channel::Receiver<Vec<ModCmd>>);

impl ChannelPair {
    /// Creates two channel pairs, that are linked to eachother.
    pub fn create() -> (Self, Self) {
        // Unbounded channel will block on both
        // send and receive, until the other end
        // is ready.
        let (a_send, b_recv): ModCmdChannel = channel::bounded(0);
        let (b_send, a_recv): ModCmdChannel = channel::bounded(0);

        let a = ChannelPair {
            sender: a_send,
            receiver: a_recv,
        };
        let b = ChannelPair {
            sender: b_send,
            receiver: b_recv,
        };

        (a, b)
    }

    pub fn send(&mut self, val: Vec<ModCmd>) -> Result<(), SendError<Vec<ModCmd>>> {
        self.sender.send(val)
    }

    pub fn receive(&mut self) -> Result<Vec<ModCmd>, RecvError> {
        self.receiver.recv()
    }
}
