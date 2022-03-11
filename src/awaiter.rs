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
    creation_result: windows::core::Result<()>,
}

impl AsyncActionAwaiter {
    pub fn new(asyncact: IAsyncAction) -> Self {
        let handler = AsyncActionCompletedHandler::new(
            |_action, _status| {
                crate::executor::post_message_asyncact_completed();
                Ok(())
            }
        );
        let creation_result = asyncact.SetCompleted(handler);
        if creation_result.is_err() {
            log::debug!("IAsyncAction::SetCompleted() fail {:?}", creation_result);
        }

        log::trace!("Create new AsyncActionAwaiter");
        Self { asyncact, creation_result }
    }
}

impl Future for AsyncActionAwaiter
{
    type Output = windows::core::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if let Err(e) = &self.creation_result {
            Poll::Ready(Err(e.clone()))
        }
        else {
            let this = unsafe { self.get_unchecked_mut() };
            let pin = unsafe { Pin::new_unchecked(&mut this.asyncact) };
            Future::poll(pin, cx)
        }
    }
}

/// Wrapper type of `IAsyncOperation` that post window message to executer on completed.
#[derive(Debug)]
pub struct AsyncOperationAwaiter<T>
where
    T: RuntimeType + 'static,
{
    asyncop: IAsyncOperation<T>,
    creation_result: windows::core::Result<()>,
}

impl<T> AsyncOperationAwaiter<T>
where
    T: RuntimeType + 'static,
{
    pub fn new(asyncop: IAsyncOperation<T>) -> Self {
        let handler = AsyncOperationCompletedHandler::new(
            |_result: &Option<IAsyncOperation<T>>, _status|{
                crate::executor::post_message_asyncop_completed();
                Ok(())
            }
        );
        let creation_result = asyncop.SetCompleted(handler);
        if creation_result.is_err() {
            log::debug!("IAsyncOperation<T>::SetCompleted() fail {:?}", creation_result);
        }

        log::trace!("Create new AsyncOperationAwaiter");
        Self { asyncop, creation_result }
    }
}

impl<T> Future for AsyncOperationAwaiter<T>
where
    T: RuntimeType + 'static,
{
    type Output = windows::core::Result<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if let Err(e) = &self.creation_result {
            Poll::Ready(Err(e.clone()))
        }
        else {
            let this = unsafe { self.get_unchecked_mut() };
            let pin = unsafe { Pin::new_unchecked(&mut this.asyncop) };
            Future::poll(pin, cx)
        }
    }
}
