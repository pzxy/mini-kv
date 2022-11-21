use std::sync::Arc;

use http::StatusCode;
use tracing::{debug, info};

use crate::{command_request::RequestData, *}; //作为service 应该引入全部的crate
mod command_service;

/// 对 Command 的处理的抽象
pub trait CommandService {
    /// 处理 Command，返回 Response
    fn execute(self, store: &impl Storage) -> CommandResponse;
}

/// Service 数据结构
/// 内部有一个 ServiceInner 存放实际的数据结构，
/// Service 只是用 Arc 包裹了 ServiceInner。
/// 这也是 Rust 的一个惯例，把需要在多线程下 clone 的主体和其内部结构分开，这样代码逻辑更加清晰
pub struct Service<Store = MemTable> {
    inner: Arc<ServiceInner<Store>>,
}
/// 自己实现Clone方法，克隆数据的引用，而不是深度克隆
impl<Store> Clone for Service<Store> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

pub struct ServiceInner<Store> {
    store: Store,
    on_received: Vec<fn(&CommandRequest)>,
    on_executed: Vec<fn(&CommandResponse)>,
    on_before_send: Vec<fn(&mut CommandResponse)>,
    on_after_send: Vec<fn()>,
}

impl<Store: Storage> From<ServiceInner<Store>> for Service<Store> {
    fn from(v: ServiceInner<Store>) -> Self {
        Self { inner: Arc::new(v) }
    }
}

impl<Store: Storage> Service<Store> {
    // pub fn new(store: Store) -> Self {
    //     Self {
    //         inner: Arc::new(ServiceInner::new(store)),
    //     }
    // }

    pub fn execute(&self, cmd: CommandRequest) -> CommandResponse {
        debug!("Got request: {:?}", cmd);
        // 发送on_received 事件
        self.inner.on_received.notify(&cmd);
        let mut res = dispatch(cmd, &self.inner.store);
        //  发送on_excuted 事件
        debug!("Executed response: {:?}", res);
        self.inner.on_executed.notify(&res);
        self.inner.on_before_send.notify(&mut res);
        // 如果非空则进入
        if !self.inner.on_before_send.is_empty() {
            debug!("Modified response: {:?}", res);
        }
        res
    }
}

impl<Store: Storage> ServiceInner<Store> {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            on_received: vec![],
            on_executed: vec![],
            on_before_send: vec![],
            on_after_send: vec![],
        }
    }

    fn fn_received(mut self, f: fn(&CommandRequest)) -> Self {
        self.on_received.push(f);
        self
    }

    fn fn_executed(mut self, f: fn(&CommandResponse)) -> Self {
        self.on_executed.push(f);
        self
    }

    fn fn_before_send(mut self, f: fn(&mut CommandResponse)) -> Self {
        self.on_before_send.push(f);
        self
    }

    fn fn_after_send(mut self, f: fn()) -> Self {
        self.on_after_send.push(f);
        self
    }
}

fn dispatch(cmd: CommandRequest, store: &impl Storage) -> CommandResponse {
    match cmd.request_data {
        Some(RequestData::Hget(param)) => param.execute(store),
        Some(RequestData::Hgetall(param)) => param.execute(store),
        Some(RequestData::Hset(param)) => param.execute(store),
        None => KvError::InvalidCommand("Request has no data".into()).into(),
        _ => KvError::Internal("Not implemented".into()).into(),
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;
    use crate::{MemTable, Value};

    #[test]
    fn service_should_works() {
        // 我们需要一个 service 结构至少包含 Storage
        let service: Service = ServiceInner::new(MemTable::default()).into();

        // service 可以运行在多线程环境下，它的 clone 应该是轻量级的
        let cloned = service.clone();

        // 创建一个线程，在 table t1 中写入 k1, v1
        let handle = thread::spawn(move || {
            let res = cloned.execute(CommandRequest::new_hset("t1", "k1", "v1".into()));
            assert_res_ok(res, &[Value::default()], &[]);
        });
        handle.join().unwrap();

        // 在当前线程下读取 table t1 的 k1，应该返回 v1
        let res = service.execute(CommandRequest::new_hget("t1", "k1"));
        assert_res_ok(res, &["v1".into()], &[]);
    }
}

#[cfg(test)]
use crate::{Kvpair, Value};

// 测试成功返回的结果
#[cfg(test)]
pub fn assert_res_ok(mut res: CommandResponse, values: &[Value], pairs: &[Kvpair]) {
    res.pairs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(res.status, 200);
    assert_eq!(res.message, "");
    assert_eq!(res.values, values);
    assert_eq!(res.pairs, pairs);
}

// 测试失败返回的结果
#[cfg(test)]
pub fn assert_res_error(res: CommandResponse, code: u32, msg: &str) {
    assert_eq!(res.status, code);
    assert!(res.message.contains(msg));
    assert_eq!(res.values, &[]);
    assert_eq!(res.pairs, &[]);
}

#[test]
fn evnet_registration_should_work() {
    fn b(cmd: &CommandRequest) {
        info!("Got {:?}", cmd);
    }
    fn c(cmd: &CommandResponse) {
        info!("{:?}", cmd);
    }
    fn d(res: &mut CommandResponse) {
        res.status = StatusCode::CREATED.as_u16() as _;
    }
    fn e() {
        info!("Data is sent")
    }
    let service: Service = ServiceInner::new(MemTable::default())
        .fn_received(|_: &CommandRequest| {})
        .fn_received(b)
        .fn_executed(c)
        .fn_before_send(d)
        .fn_after_send(e)
        .into();

    let res = service.execute(CommandRequest::new_hset("t1", "k1", "v1".into()));
    assert_eq!(res.status, StatusCode::CREATED.as_u16() as _);
    assert_eq!(res.message, "");
    assert_eq!(res.values, vec![Value::default()]);
}

/// 实现通知trait，因为事件有可变，不可变两种，所以要区分开。因为泛型只能同时存在一种类型。
/// 可变事件通知trait
/// Arg 就是 fn(&CommandRequest);
pub trait NotifyMut<Arg> {
    fn notify(&self, arg: &mut Arg);
}
/// 不可变 trait
pub trait Notify<Arg> {
    fn notify(&self, arg: &Arg);
}

impl<Arg> NotifyMut<Arg> for Vec<fn(&mut Arg)> {
    /// inline 告诉编译器，直接在调用函数哪里展开就行了，不要进行函数调用。
    #[inline]
    fn notify(&self, arg: &mut Arg) {
        for f in self {
            f(arg);
        }
    }
}

impl<Arg> Notify<Arg> for Vec<fn(&Arg)> {
    #[inline]
    fn notify(&self, arg: &Arg) {
        for f in self {
            f(arg);
        }
    }
}
