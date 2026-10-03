// Скрытый тест оценщика r05-holdout: число выделений памяти и пик занятой
// памяти не растут с размером лога. Отдельный файл — отдельный процесс, чтобы
// другие тесты не влияли на счётчики.
use logtop::{bytes_by_status, top_paths};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

struct Counting;

static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static CURRENT: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

fn grow(size: usize) {
    let now = CURRENT.fetch_add(size, Relaxed) + size;
    PEAK.fetch_max(now, Relaxed);
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        grow(layout.size());
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        CURRENT.fetch_sub(layout.size(), Relaxed);
        System.dealloc(ptr, layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        CURRENT.fetch_sub(layout.size(), Relaxed);
        grow(new_size);
        System.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

const LINES: usize = 100_000;
const MAX_ALLOCS: usize = 1_000;
const MAX_PEAK: usize = 512 << 10;

fn big_log() -> String {
    let mut log = String::new();
    let statuses = [200, 404, 500, 302];
    for i in 0..LINES {
        if i % 97 == 0 {
            log.push_str("garbage line\n");
            continue;
        }
        let path = (i * 7919) % 50;
        let status = statuses[i % statuses.len()];
        log.push_str(&format!("GET /api/v1/item/{path} {status} {}\n", i % 5000));
    }
    log
}

// Выделения и прирост пика памяти за время вызова f.
fn measure<T>(f: impl FnOnce() -> T) -> (T, usize, usize) {
    let base = CURRENT.load(Relaxed);
    PEAK.store(base, Relaxed);
    let allocs = ALLOCS.load(Relaxed);
    let result = f();
    (
        result,
        ALLOCS.load(Relaxed) - allocs,
        PEAK.load(Relaxed).saturating_sub(base),
    )
}

#[test]
fn grader_allocations_do_not_grow_with_log() {
    let log = big_log();
    let (top, allocs, peak) = measure(|| top_paths(&log, 10));
    assert_eq!(top.len(), 10);
    assert!(
        allocs < MAX_ALLOCS,
        "top_paths: {allocs} выделений памяти на {LINES} строк лога"
    );
    assert!(
        peak < MAX_PEAK,
        "top_paths: пик памяти {peak} байт на логе {} байт",
        log.len()
    );

    let (totals, allocs, peak) = measure(|| bytes_by_status(&log));
    assert_eq!(totals.len(), 4);
    assert!(
        allocs < MAX_ALLOCS,
        "bytes_by_status: {allocs} выделений памяти на {LINES} строк лога"
    );
    assert!(
        peak < MAX_PEAK,
        "bytes_by_status: пик памяти {peak} байт на логе {} байт",
        log.len()
    );
}
