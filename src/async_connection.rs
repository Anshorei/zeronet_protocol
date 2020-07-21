use crate::error::Error;
use crate::message::value::Value;
use crate::requestable::Requestable;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::clone::Clone;
use std::collections::HashMap;
use std::future::Future;
use std::io::{Read, Write};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

pub struct SharedState<T> {
  pub reader:   Arc<Mutex<dyn Read + Send>>,
  pub writer:   Arc<Mutex<dyn Write + Send>>,
  pub requests: HashMap<usize, (Arc<Mutex<Option<Result<T, Error>>>>, Option<Waker>)>,
  pub value:    Arc<Mutex<Option<Result<T, Error>>>>,
  pub waker:    Option<Waker>,
}

pub struct SendState<T> {
  pub writer: Arc<Mutex<dyn Write + Send>>,
  pub value:  Option<T>,
  pub result: Option<Result<(), Error>>,
}

pub struct SendFuture<T> {
  pub state: Arc<Mutex<SendState<T>>>,
  pub waker: Option<Waker>,
}

impl<T: 'static + DeserializeOwned + Serialize + Send + Requestable> Future for SendFuture<T> {
  type Output = Result<(), Error>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let mut state = self.state.lock().unwrap();
    if state.result.is_some() {
      return Poll::Ready(state.result.take().unwrap());
    }

    let waker = cx.waker().clone();
    let moved_state = self.state.clone();
    std::thread::spawn(move || {
      let mut state = moved_state.lock().unwrap();
      let writer = state.writer.clone();
      let mut writer = writer.lock().unwrap();

      // TODO: replace this once issue is resolved
      // first we have to serialize to json value
      // because rmp_serde gives UnknownLength error
      // https://github.com/3Hren/msgpack-rust/issues/196
      // when this get fixed we can just use:
      //
      // let result = rmp_serde::encode::write_named(&mut *writer, &state.value.take().unwrap())
      // 	.map_err(|err| Error::from(err));
      // state.result = Some(result);

      // TODO: add timeout for pending requests

      if let Some(value) = state.value.take() {
        let jsoned = match serde_json::to_value(value) {
          Ok(json) => json,
          Err(err) => {
            state.result = Some(Err(err.into()));
            waker.wake();
            return;
          }
        };
        let value: Result<Value, _> = serde_json::from_value(jsoned);
        let result = match value {
          Err(err) => Err(Error::from(err)),
          Ok(jsoned) => {
            rmp_serde::encode::write_named(&mut *writer, &jsoned).map_err(|err| Error::from(err))
          }
        };
        state.result = Some(result);
      }

      waker.wake();
    });

    Poll::Pending
  }
}

#[must_use = "futures do nothing unless polled"]
pub struct ReceiveFuture<T> {
  shared_state: Arc<Mutex<SharedState<T>>>,
  value:        Arc<Mutex<Option<Result<T, Error>>>>,
}

impl<T: 'static + DeserializeOwned + Serialize + Send + Requestable> Future for ReceiveFuture<T> {
  type Output = Result<T, Error>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    {
      let mut value = self.value.lock().unwrap();
      if let Some(value) = value.take() {
        return Poll::Ready(value);
      }
    }

    let waker = cx.waker().clone();
    {
      let mut shared_state = self.shared_state.lock().unwrap();
      shared_state.waker.replace(waker.clone());
    }
    recv(self.shared_state.clone(), waker);

    Poll::Pending
  }
}

#[must_use = "futures do nothing unless polled"]
pub struct ResponseFuture<T> {
  shared_state: Arc<Mutex<SharedState<T>>>,
  value:        Arc<Mutex<Option<Result<T, Error>>>>,
  req_id:       Option<usize>,
}

impl<T: 'static + DeserializeOwned + Serialize + Send + Requestable> Future for ResponseFuture<T> {
  type Output = Result<T, Error>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    {
      let mut value = self.value.lock().unwrap();
      if let Some(value) = value.take() {
        return Poll::Ready(value);
      }
    }

    let waker = cx.waker().clone();
    {
      let mut shared_state = self.shared_state.lock().unwrap();
      if let Some(req_id) = self.req_id {
        shared_state
          .requests
          .insert(req_id, (self.value.clone(), Some(waker.clone())));
      } else {
        return Poll::Ready(Err(Error::MissingReqId));
      }
    }
    recv(self.shared_state.clone(), waker);

    Poll::Pending
  }
}

fn recv<T>(shared_state: Arc<Mutex<SharedState<T>>>, waker: Waker)
where
  T: 'static + DeserializeOwned + Serialize + Send + Requestable,
{
  let shared_state_g = shared_state.lock().unwrap();

  {
    if shared_state_g.reader.try_lock().is_err() {
      return;
    }
  }

  let reader = shared_state_g.reader.clone();
  let moved_state = shared_state.clone();
  std::thread::spawn(move || {
    let mut reader = reader.lock().unwrap();
    let response: Result<T, _> = rmp_serde::from_read(&mut *reader);
    let mut moved_state = moved_state.lock().unwrap();

    if let Err(err) = response {
      println!("Connection closed: {:?}", err);
      close_connection(&moved_state);
      return;
    }

    let response = response.unwrap();

    match response.to() {
      Some(to) => {
        let (value, other_waker) = moved_state.requests.remove(&to).unwrap();
        let mut value = value.lock().unwrap();
        *value = Some(Ok(response));
        if let Some(other_waker) = other_waker {
          other_waker.wake();
        }
      }
      None => {
        {
          let mut value = moved_state.value.lock().unwrap();
          *value = Some(Ok(response));
        }
        if let Some(other_waker) = moved_state.waker.take() {
          other_waker.wake();
        }
      }
    }
    waker.wake();
  });
}

fn close_connection<T>(shared_state: &SharedState<T>) {
  for (value, waker) in shared_state.requests.values() {
    let mut value = value.lock().unwrap();
    match *value {
      None => {
        *value = Some(Err(Error::ConnectionClosed));
      }
      Some(_) => {}
    }
    if let Some(waker) = waker.clone() {
      waker.wake();
    }
  }
  let mut value = shared_state.value.lock().unwrap();
  *value = Some(Err(Error::ConnectionClosed));
  if let Some(waker) = shared_state.waker.clone() {
    waker.wake();
  }
}

pub struct Connection<T>
where
  T: 'static + DeserializeOwned + Serialize + Send + Requestable,
{
  pub shared_state: Arc<Mutex<SharedState<T>>>,
}

impl<T: 'static + DeserializeOwned + Serialize + Send + Requestable> Connection<T> {
  pub fn send(&mut self, message: T) -> impl Future<Output = Result<(), Error>> {
    let shared_state = self.shared_state.lock().unwrap();
    let state = SendState {
      writer: shared_state.writer.clone(),
      result: None,
      value:  Some(message),
    };
    SendFuture {
      state: Arc::new(Mutex::new(state)),
      waker: None,
    }
  }

  pub fn recv(&mut self) -> impl Future<Output = Result<T, Error>> {
    let value = Arc::new(Mutex::new(None));
    let mut shared_state = self.shared_state.lock().unwrap();
    shared_state.value = value.clone();

    ReceiveFuture {
      shared_state: self.shared_state.clone(),
      value:        value,
    }
  }

  pub fn request(&mut self, message: T) -> impl Future<Output = Result<T, Error>> {
    let value = Arc::new(Mutex::new(None));

    {
      let mut shared_state = self.shared_state.lock().unwrap();
      if let Some(req_id) = message.req_id() {
        shared_state.requests.insert(req_id, (value.clone(), None));
      }
    }

    let future = ResponseFuture {
      shared_state: self.shared_state.clone(),
      value:        value,
      req_id:       message.req_id(),
    };

    let send_future = self.send(message);

    return async {
      let res = send_future.await;
      if res.is_ok() {
        future.await
      } else {
        Err(res.unwrap_err())
      }
    };
  }
}
