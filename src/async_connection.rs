use std::{
  collections::HashMap,
  io::{Read, Write},
  sync::{Arc, Mutex},
};

use serde::{de::DeserializeOwned, Serialize};
use serde_bytes::ByteBuf;

use crate::{error::Error, requestable::Requestable, state::*};

#[derive(Clone)]
pub struct Connection<T: 'static + DeserializeOwned + Serialize + Send + Requestable> {
  pub shared_state: Arc<Mutex<SharedState<T>>>,
}

impl<T: 'static + DeserializeOwned + Serialize + Send + Requestable> Connection<T> {
  pub fn new(reader: Box<dyn Read + Send>, writer: Box<dyn Write + Send>) -> Self {
    let shared_state = SharedState::<T> {
      reader:   Arc::new(Mutex::new(reader)),
      writer:   Arc::new(Mutex::new(writer)),
      requests: HashMap::new(),
      values:   Arc::new(Mutex::new(vec![])),
      wakers:   vec![],
      closed:   false,
    };
    return Self {
      shared_state: Arc::new(Mutex::new(shared_state)),
    };
  }

  pub fn is_closed(&self) -> bool {
    let shared_state = self.shared_state.lock().unwrap();
    return shared_state.closed;
  }

  pub async fn send(&mut self, message: T, buf: Option<ByteBuf>) -> Result<(), Error> {
    let shared_state = self.shared_state.lock().unwrap();
    let state = SendState {
      writer: shared_state.writer.clone(),
      result: None,
      value: Some(message),
      buf,
    };
    SendFuture {
      state: Arc::new(Mutex::new(state)),
      waker: None,
    }
    .await
  }

  pub async fn recv(&mut self) -> Result<T, Error> {
    let shared_state = self.shared_state.lock().unwrap();

    ReceiveFuture {
      shared_state: self.shared_state.clone(),
      values:       shared_state.values.clone(),
    }
    .await
  }

  pub async fn request(&mut self, message: T) -> Result<T, Error> {
    let value = Arc::new(Mutex::new(None));

    {
      let mut shared_state = self.shared_state.lock().unwrap();
      if let Some(req_id) = message.req_id() {
        shared_state.requests.insert(req_id, (value.clone(), None));
      }
    }

    let future = ResponseFuture {
      shared_state: self.shared_state.clone(),
      value,
      req_id: message.req_id(),
    };

    let res = self.send(message, None).await;
    if res.is_ok() {
      future.await
    } else {
      Err(res.unwrap_err())
    }
  }
}
