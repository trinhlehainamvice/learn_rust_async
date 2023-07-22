use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;

pub struct FutureTimer {
    share_state: Arc<Mutex<ShareState>>,
}

impl Future for FutureTimer {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut share_state = self.share_state.lock().unwrap();
        match share_state.is_completed {
            true => Poll::Ready(()),
            _ => {
                share_state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl FutureTimer {
    pub fn new(time: Duration) -> Self {
        let share_state = Arc::new(Mutex::new(ShareState {
            is_completed: false,
            waker: None,
        }));

        let share_state_clone = share_state.clone();
        thread::spawn(move || {
            thread::sleep(time);
            let mut shared_state = share_state_clone.lock().unwrap();
            shared_state.is_completed = true;
            if let Some(waker) = shared_state.waker.take() {
                waker.wake();
            }
        });

        Self { share_state }
    }
}

struct ShareState {
    is_completed: bool,
    waker: Option<Waker>,
}
