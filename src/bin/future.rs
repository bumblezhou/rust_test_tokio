use std::future::{Future};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

struct Delay {
    when: Instant
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.when {
            println!("Delay->poll: Hello world");
            Poll::Ready("done")
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[tokio::main]
async fn main() {
    let when = Instant::now() + Duration::from_millis(10);
    let future = Delay { when };

    let out = future.await;
    println!("main: future.await out: {:?}", &out);
    assert_eq!(out, "done");
}