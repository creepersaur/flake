use std::{future::Future, pin::Pin, task::{Context, Poll}};

pub struct NextFrame(bool);

pub fn next_frame() -> NextFrame {
    NextFrame(false)
}

impl Future for NextFrame {
    type Output = ();
    
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        if self.0 {
            Poll::Ready(())
        } else {
            self.0 = true;
            Poll::Pending
        }
    }
}