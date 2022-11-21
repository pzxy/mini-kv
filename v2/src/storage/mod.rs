mod memory;
use crate::{KvError, Kvpair, Value};
pub use memory::MemTable;

/// 对存储的抽象，我们不关心数据存在哪儿，但需要定义外界如何和存储打交道
pub trait Storage {
    /// 从一个 HashTable 里获取一个 key 的 value
    fn get(&self, table: &str, key: &str) -> Result<Option<Value>, KvError>;
    /// 从一个 HashTable 里设置一个 key 的 value，返回旧的 value
    fn set(&self, table: &str, key: String, value: Value) -> Result<Option<Value>, KvError>;
    /// 查看 HashTable 中是否有 key
    fn contains(&self, table: &str, key: &str) -> Result<bool, KvError>;
    /// 从 HashTable 中删除一个 key
    fn del(&self, table: &str, key: &str) -> Result<Option<Value>, KvError>;
    /// 遍历 HashTable，返回所有 kv pair（这个接口不好）
    fn get_all(&self, table: &str) -> Result<Vec<Kvpair>, KvError>;
    /// 遍历 HashTable，返回 kv pair 的 Iterator
    fn get_iter(&self, table: &str) -> Result<Box<dyn Iterator<Item = Kvpair>>, KvError>;
}

// #[test]
// fn memtable_iter_should_work() {
//     let store = MemTable::new();
//     test_get_iter(store);
// }

pub struct StorageIter<T> {
    data: T,
}

impl<T> StorageIter<T> {
    fn new(data: T) -> Self {
        Self { data }
    }
}

/// 实现了这个trait，那里面的关联类型，也应该赋值才行。
/// todo 这里没看懂。以后再说。
impl<T> Iterator for StorageIter<T>
where
    T: Iterator,
    // Item 关联类型，必须在实现的时候给一个实际的默认类型。
    T::Item: Into<Kvpair>,
{
    type Item = Kvpair;
    fn next(&mut self) -> Option<Self::Item> {
        self.data.next().map(|v| v.into())
    }
}
