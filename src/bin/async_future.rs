use std::future::Future;
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
            // println!("Delay->poll: cx.waker()");
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

enum MainFuture {
    Initialized,
    Waiting(Delay),
    Terminated,
}

impl Future for MainFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use MainFuture::*;

        loop {
            match *self {
                Initialized => {
                    println!("MainFuture->Initialized: delay = now + 10");
                    let when = Instant::now() + Duration::from_millis(10);
                    let future = Delay{ when };
                    *self = Waiting(future)
                }
                Waiting(ref mut my_future) => {
                    match Pin::new(my_future).poll(cx) {
                        Poll::Ready(out) => {
                            println!("MainFuture->Waiting: Ready out: {:?}", &out);
                            assert_eq!(out, "done");
                            *self = Terminated;
                            return Poll::Ready(());
                        }
                        Poll::Pending => {
                            // println!("MainFuture->Waiting: Pending");
                            return  Poll::Pending;
                        }
                    }
                }
                Terminated => {
                    panic!("future polled after completion")
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let main_future = MainFuture::Initialized;

    main_future.await;
}