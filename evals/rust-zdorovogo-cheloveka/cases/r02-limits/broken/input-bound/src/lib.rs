//! Чтение архивов `.pak`.
//!
//! Формат, все числа little-endian:
//!
//! ```text
//! magic   4 байта  b"PAK1"
//! count   u32      число записей
//! таблица count × { offset: u64, len: u32 } — положение данных записи
//!                  относительно начала области данных
//! данные  всё, что после таблицы
//! ```

use std::fmt;

const MAGIC: &[u8; 4] = b"PAK1";
const HEADER_LEN: usize = 8;
const ENTRY_LEN: usize = 12;

/// Ошибка разбора архива.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Вход не начинается с `PAK1`.
    BadMagic,
    /// Вход закончился внутри заголовка или таблицы.
    Truncated,
    /// В заголовке больше записей, чем бывает в настоящих архивах.
    TooManyEntries,
    /// Данные записи `index` выходят за конец архива.
    OutOfBounds { index: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadMagic => write!(f, "не архив PAK1"),
            Self::Truncated => write!(f, "архив обрезан"),
            Self::TooManyEntries => write!(f, "слишком много записей"),
            Self::OutOfBounds { index } => write!(f, "запись {index} выходит за конец архива"),
        }
    }
}

impl std::error::Error for Error {}

struct Entry {
    offset: u32,
    len: u32,
}

/// Разобранный архив; данные записей заимствуются из входа.
pub struct Archive<'a> {
    entries: Vec<Entry>,
    data: &'a [u8],
}

impl<'a> Archive<'a> {
    /// Разбирает заголовок и таблицу и проверяет, что данные каждой записи
    /// лежат внутри архива.
    ///
    /// # Errors
    ///
    /// Возвращает [`Error`], если вход не является корректным архивом.
    pub fn parse(input: &'a [u8]) -> Result<Self, Error> {
        if input.len() < HEADER_LEN {
            return Err(Error::Truncated);
        }
        if &input[..4] != MAGIC {
            return Err(Error::BadMagic);
        }
        let count = u32::from_le_bytes(input[4..8].try_into().unwrap());
        let count = usize::try_from(count).map_err(|_| Error::TooManyEntries)?;
        let table_len = count.checked_mul(ENTRY_LEN).ok_or(Error::TooManyEntries)?;
        if table_len > input.len() - HEADER_LEN {
            return Err(Error::Truncated);
        }

        let mut entries = Vec::with_capacity(count);
        let mut pos = HEADER_LEN;
        for index in 0..count {
            let Some(raw) = input.get(pos..pos + ENTRY_LEN) else {
                return Err(Error::Truncated);
            };
            // Данные больше 4 ГиБ не поддерживаются: такое смещение выходит за архив.
            let offset = u32::try_from(u64::from_le_bytes(raw[..8].try_into().unwrap()))
                .map_err(|_| Error::OutOfBounds { index })?;
            let len = u32::from_le_bytes(raw[8..].try_into().unwrap());
            entries.push(Entry { offset, len });
            pos += ENTRY_LEN;
        }

        let data = &input[pos..];
        for (index, entry) in entries.iter().enumerate() {
            let in_bounds = entry
                .offset
                .checked_add(entry.len)
                .is_some_and(|end| end as usize <= data.len());
            if !in_bounds {
                return Err(Error::OutOfBounds { index });
            }
        }
        Ok(Self { entries, data })
    }

    /// Число записей.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Архив без записей.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Данные записи `index`; `None`, если такой записи нет.
    pub fn get(&self, index: usize) -> Option<&'a [u8]> {
        let entry = self.entries.get(index)?;
        let start = entry.offset as usize;
        Some(&self.data[start..start + entry.len as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn archive(entries: &[(u64, u32)], data: &[u8]) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        out.extend_from_slice(&u32::try_from(entries.len()).unwrap().to_le_bytes());
        for &(offset, len) in entries {
            out.extend_from_slice(&offset.to_le_bytes());
            out.extend_from_slice(&len.to_le_bytes());
        }
        out.extend_from_slice(data);
        out
    }

    #[test]
    fn reads_entries() {
        let input = archive(&[(0, 5), (5, 0), (2, 3)], b"hello");
        let pak = Archive::parse(&input).unwrap();
        assert_eq!(pak.len(), 3);
        assert_eq!(pak.get(0), Some(&b"hello"[..]));
        assert_eq!(pak.get(1), Some(&b""[..]));
        assert_eq!(pak.get(2), Some(&b"llo"[..]));
        assert_eq!(pak.get(3), None);
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(Archive::parse(b"PAK").err(), Some(Error::Truncated));
        assert_eq!(Archive::parse(b"ZIP1\0\0\0\0").err(), Some(Error::BadMagic));
        let input = archive(&[(0, 6)], b"hello");
        assert_eq!(
            Archive::parse(&input).err(),
            Some(Error::OutOfBounds { index: 0 })
        );
    }
}
