mod error;
mod pb;
mod service;
mod storage;
mod network;

pub use error::KvError;
pub use pb::abi::*;
pub use service::*;
pub use storage::*;
pub use network::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memtable_basic_interface_should_work() {
        let store = MemTable::new();
        test_basi_interface(store);
    }

    #[test]
    fn memtable_get_all_should_work() {
        let store = MemTable::new();
        test_get_all(store);
    }

    fn test_basi_interface(store: impl Storage) {
        // 第一次 set 会创建 table，插入 key 并返回 None（之前没值）
        let v = store.set("t1", "hello".into(), "world".into());
        assert!(v.unwrap().is_none());
        // 再次 set 同样的 key 会更新，并返回之前的值
        let v1 = store.set("t1", "hello".into(), "world1".into());
        assert_eq!(v1, Ok(Some("world".into())));

        // get 存在的 key 会得到最新的值
        let v = store.get("t1", "hello");
        assert_eq!(v, Ok(Some("world1".into())));

        // get 不存在的 key 或者 table 会得到 None
        assert_eq!(Ok(None), store.get("t1", "hello1"));
        assert!(store.get("t2", "hello1").unwrap().is_none());

        // contains 纯在的 key 返回 true，否则 false
        assert_eq!(store.contains("t1", "hello"), Ok(true));
        assert_eq!(store.contains("t1", "hello1"), Ok(false));
        assert_eq!(store.contains("t2", "hello"), Ok(false));

        // del 存在的 key 返回之前的值
        let v = store.del("t1", "hello");
        assert_eq!(v, Ok(Some("world1".into())));

        // del 不存在的 key 或 table 返回 None
        assert_eq!(Ok(None), store.del("t1", "hello1"));
        assert_eq!(Ok(None), store.del("t2", "hello"));
    }

    fn test_get_all(store: impl Storage) {
        store.set("t2", "k1".into(), "v1".into()).unwrap();
        store.set("t2", "k2".into(), "v2".into()).unwrap();
        let mut data = store.get_all("t2").unwrap();
        data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(
            data,
            vec![
                Kvpair::new("k1", "v1".into()),
                Kvpair::new("k2", "v2".into())
            ]
        )
    }

}