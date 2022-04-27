//! This crate provides a wrapper type of Tokio's JoinHandle: `ChildTask`, which aborts the task when it's dropped.
//! `ChildTask` can still be awaited to join the child-task, and abort-on-drop will still trigger while it is being awaited.
//!
//! For example, if task A spawned task B but is doing something else, and task B is waiting for task C to join,
//! aborting A will also abort both B and C.

use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::task::JoinHandle;

pin_project! {
    pub struct ChildTask<T> {
        #[pin]
        inner: JoinHandle<T>
    }

    impl<T> PinnedDrop for ChildTask<T> {
        fn drop(this: Pin<&mut Self>) {
            this.inner.abort()
        }
    }
}

impl<T> Future for ChildTask<T> {
    type Output = <JoinHandle<T> as Future>::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

impl<T> From<JoinHandle<T>> for ChildTask<T> {
    fn from(inner: JoinHandle<T>) -> Self {
        Self { inner }
    }
}

#[cfg(test)]
mod tests {
    use super::ChildTask;
    use futures_util::future::pending;
    use std::sync::{Arc, RwLock};
    use tokio::task::yield_now;

    struct Sentry(Arc<RwLock<bool>>);
    impl Drop for Sentry {
        fn drop(&mut self) {
            *self.0.write().unwrap() = true
        }
    }

    #[tokio::test]
    async fn drop_while_not_waiting_for_join() {
        let dropped = Arc::new(RwLock::new(false));
        let sentry = Sentry(dropped.clone());
        let task = ChildTask::from(tokio::spawn(async move {
            let _sentry = sentry;
            pending::<()>().await
        }));
        yield_now().await;
        assert!(!*dropped.read().unwrap());
        drop(task);
        yield_now().await;
        assert!(*dropped.read().unwrap());
    }

    #[tokio::test]
    async fn drop_while_waiting_for_join() {
        let dropped = Arc::new(RwLock::new(false));
        let sentry = Sentry(dropped.clone());
        let handle = tokio::spawn(async move {
            ChildTask::from(tokio::spawn(async move {
                let _sentry = sentry;
                pending::<()>().await
            }))
            .await
            .unwrap()
        });
        yield_now().await;
        assert!(!*dropped.read().unwrap());
        handle.abort();
        yield_now().await;
        assert!(*dropped.read().unwrap());
    }

    #[tokio::test]
    async fn no_drop_only_join() {
        assert_eq!(
            ChildTask::from(tokio::spawn(async {
                yield_now().await;
                5
            }))
            .await
            .unwrap(),
            5
        )
    }
}
