//! Читалка, которая отдаёт ровно заявленный размер и считает sha256 отданного.
//!
//! Оба свойства нужны для tar: размер записи стоит в заголовке, до содержимого.
//! Если файл укоротится между `metadata()` и чтением — а в `data/` в это время
//! идут загрузки, — поток кончится раньше заголовка, и битым окажется **весь**
//! архив, а не один файл. Поэтому недостачу добиваем нулями, а лишнее не
//! читаем: содержимое обязано совпасть с заголовком байт в байт.
//!
//! Хешируется именно то, что ушло в архив, а не то, что лежало на диске, —
//! иначе `meta.json` описывал бы файл, которого в архиве нет.

use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::io::{self, Read};
use std::rc::Rc;

/// Общий счётчик хеша: `tar::Builder::append_data` забирает читалку по
/// значению, и достать её обратно, чтобы снять digest, уже нельзя.
pub type Digest256 = Rc<RefCell<Sha256>>;

pub struct HashRead<R> {
    inner: R,
    left: u64,
    out: Digest256,
}

impl<R: Read> HashRead<R> {
    pub fn new(inner: R, size: u64, out: Digest256) -> Self {
        Self {
            inner,
            left: size,
            out,
        }
    }
}

impl<R: Read> Read for HashRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.left == 0 {
            return Ok(0);
        }
        let cap = buf.len().min(self.left as usize);
        let mut n = self.inner.read(&mut buf[..cap])?;
        if n == 0 {
            // Файл укоротился на ходу — добиваем нулями до заявленного размера.
            buf[..cap].fill(0);
            n = cap;
        }
        self.out.borrow_mut().update(&buf[..n]);
        self.left -= n as u64;
        Ok(n)
    }
}

/// Снять hex-представление накопленного sha256.
pub fn finish(out: &Digest256) -> String {
    hex::encode(out.borrow().clone().finalize())
}

/// Пустой счётчик под новую часть.
pub fn digest() -> Digest256 {
    Rc::new(RefCell::new(Sha256::new()))
}
