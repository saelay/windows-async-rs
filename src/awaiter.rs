use std::future::{Future};
use std::pin::{Pin};
use std::task::{Poll, Context};

use windows::core::{
    RuntimeType,
};

/* required feature "Foundation" */
use windows::Foundation::{
    IAsyncAction,
    IAsyncOperation,
    AsyncOperationCompletedHandler,
    AsyncActionCompletedHandler,
    AsyncStatus,
};

/// convert into `AsyncActionAwaiter` or `AsyncOperationAwaiter`.
pub trait IntoAwaiter
{
    type Awaiter;
    fn into_awaiter(self) -> Self::Awaiter;
}

impl IntoAwaiter for IAsyncAction
{
    type Awaiter = AsyncActionAwaiter;

    fn into_awaiter(self) -> Self::Awaiter {
        AsyncActionAwaiter::new(self)
    }
}

impl<T> IntoAwaiter for IAsyncOperation<T>
where
    T: RuntimeType + 'static,
{
    type Awaiter = AsyncOperationAwaiter<T>;

    fn into_awaiter(self) -> Self::Awaiter {
        AsyncOperationAwaiter::new(self)
    }
}

/// Wrapper type of `IAsyncAction` that post window message to executer on completed.
#[derive(Debug)]
pub struct AsyncActionAwaiter
{
    asyncact: IAsyncAction,
    is_first_poll: bool,
}

impl AsyncActionAwaiter {
    pub fn new(asyncact: IAsyncAction) -> Self {
        log::trace!("Create new AsyncActionAwaiter");
        Self { asyncact, is_first_poll: true }
    }
}

impl Future for AsyncActionAwaiter
{
    type Output = windows::core::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        if this.is_first_poll {
            this.is_first_poll = false;
            match this.asyncact.Status() {
                Ok(status) => {
                    if status == AsyncStatus::Started {
                        let waker = cx.waker().clone();
                        let handler = AsyncActionCompletedHandler::new(
                            move |_result, _status|{
                                let waker = waker.clone();
                                waker.wake();
                                Ok(())
                            }
                        );
            
                        if let Err(e) = this.asyncact.SetCompleted(handler) {
                            return Poll::Ready(Err(e));
                        }
                    }
                }
                Err(e) => { return Poll::Ready(Err(e)); }
            }
        }

        let pin = unsafe { Pin::new_unchecked(&mut this.asyncact) };
        Future::poll(pin, cx)
    }
}

/// Wrapper type of `IAsyncOperation` that post window message to executer on completed.
#[derive(Debug)]
pub struct AsyncOperationAwaiter<T>
where
    T: RuntimeType + 'static,
{
    asyncop: IAsyncOperation<T>,
    is_first_poll: bool,
}

impl<T> AsyncOperationAwaiter<T>
where
    T: RuntimeType + 'static,
{
    pub fn new(asyncop: IAsyncOperation<T>) -> Self {
        log::trace!("Create new AsyncOperationAwaiter");
        Self { asyncop, is_first_poll: true }
    }
}

impl<T> Future for AsyncOperationAwaiter<T>
where
    T: RuntimeType + 'static,
{
    type Output = windows::core::Result<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        if this.is_first_poll {
            this.is_first_poll = false;
            match this.asyncop.Status() {
                Ok(status) => {
                    if status == AsyncStatus::Started {
                        let waker = cx.waker().clone();
                        let handler = AsyncOperationCompletedHandler::new(
                            move |_result: &Option<IAsyncOperation<T>>, _status|{
                                let waker = waker.clone();
                                waker.wake();
                                Ok(())
                            }
                        );
            
                        if let Err(e) = this.asyncop.SetCompleted(handler) {
                            return Poll::Ready(Err(e));
                        }
                    }
                }
                Err(e) => { return Poll::Ready(Err(e)); }
            }
        }

        let pin = unsafe { Pin::new_unchecked(&mut this.asyncop) };
        Future::poll(pin, cx)
    }
}
