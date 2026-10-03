// Скрытые тесты оценщика r02-limits.
//
// Глобальный аллокатор отказывает в любом запросе больше 64 МиБ: выделение
// памяти по размеру из заголовка без проверки аварийно завершает тесты
// («memory allocation of N bytes failed»), а не съедает память машины.
use pak::Archive;
use std::alloc::{GlobalAlloc, Layout, System};

struct Capped;

const CAP: usize = 64 << 20;

unsafe impl GlobalAlloc for Capped {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() > CAP {
            return std::ptr::null_mut();
        }
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if new_size > CAP {
            return std::ptr::null_mut();
        }
        System.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
static ALLOC: Capped = Capped;

// Архив с произвольным count и таблицей entries.
fn raw(count: u32, entries: &[(u64, u32)], data: &[u8]) -> Vec<u8> {
    let mut out = b"PAK1".to_vec();
    out.extend_from_slice(&count.to_le_bytes());
    for &(offset, len) in entries {
        out.extend_from_slice(&offset.to_le_bytes());
        out.extend_from_slice(&len.to_le_bytes());
    }
    out.extend_from_slice(data);
    out
}

fn archive(entries: &[(u64, u32)], data: &[u8]) -> Vec<u8> {
    raw(entries.len() as u32, entries, data)
}

#[track_caller]
fn assert_rejected(input: &[u8], what: &str) {
    if let Ok(pak) = Archive::parse(input) {
        let contents: Vec<_> = (0..pak.len().min(4)).map(|i| pak.get(i)).collect();
        panic!(
            "{what}: ожидалась ошибка, архив принят: {} записей, первые: {contents:?}",
            pak.len()
        );
    }
}

#[test]
fn grader_valid_archive() {
    let input = archive(&[(0, 5), (5, 0), (2, 3), (5, 0)], b"hello");
    let pak = Archive::parse(&input).unwrap();
    let got: Vec<_> = (0..pak.len()).map(|i| pak.get(i).unwrap()).collect();
    assert_eq!(got, vec![&b"hello"[..], b"", b"llo", b""]);
    assert_eq!(pak.get(4), None);
    let empty = archive(&[], b"tail");
    assert!(Archive::parse(&empty).unwrap().is_empty());
}

#[test]
fn grader_huge_count() {
    for count in [u32::MAX, 1 << 31, 10_000_000, 65_537] {
        assert_rejected(&raw(count, &[(0, 0)], b""), &format!("count = {count}"));
    }
}

#[test]
fn grader_entry_limit() {
    let zero = vec![(0, 0); 65_536];
    let input = archive(&zero, b"");
    let pak = Archive::parse(&input).expect("65 536 записей — допустимый архив");
    assert_eq!(pak.len(), 65_536);
    let mut more = zero;
    more.push((0, 0));
    assert_rejected(&archive(&more, b""), "65 537 записей с полной таблицей");
}

#[test]
fn grader_offset_truncation() {
    // Смещение больше 4 ГиБ не должно превращаться в маленькое.
    let data = b"0123456789";
    for offset in [1u64 << 32, (1 << 32) + 2, u64::MAX - 1, u64::MAX] {
        assert_rejected(
            &archive(&[(offset, 3)], data),
            &format!("offset = {offset}"),
        );
    }
}

#[test]
fn grader_offset_overflow() {
    let data = b"0123456789";
    for (offset, len) in [
        (u64::from(u32::MAX) - 1, 10),
        (u64::from(u32::MAX), u32::MAX),
        (1, u32::MAX),
        (0, u32::MAX),
        (5, 6),
    ] {
        assert_rejected(
            &archive(&[(offset, len)], data),
            &format!("offset = {offset}, len = {len}"),
        );
    }
    // Граница: запись ровно до конца данных допустима.
    let input = archive(&[(5, 5), (10, 0)], data);
    let pak = Archive::parse(&input).unwrap();
    assert_eq!(pak.get(0), Some(&b"56789"[..]));
    assert_eq!(pak.get(1), Some(&b""[..]));
}

#[test]
fn grader_truncated_table() {
    let full = archive(&[(0, 1), (1, 1)], b"ab");
    for cut in 0..8 + 24 {
        assert_rejected(&full[..cut], &format!("обрезано до {cut} байт"));
    }
}
