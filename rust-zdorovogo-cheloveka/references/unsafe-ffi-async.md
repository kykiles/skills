# `unsafe`, FFI и async

## `unsafe`

Не вводи его без явной необходимости (FFI, доказанная измерением потребность в производительности, отсутствие безопасной альтернативы) и согласия пользователя. Каждый `unsafe`-блок сопровождай комментарием `// SAFETY:` о том, какие условия выполняются и почему; у `unsafe fn` — раздел `# Safety` в документации. Держи `unsafe` в небольшом модуле за безопасным API, чтобы инвариант проверялся в одном месте. Правя существующий `unsafe`-код, сохрани его инварианты и перечитай комментарии. Если проект использует Miri, прогони затронутые тесты через `cargo +nightly miri test`.

## FFI

Паника не должна выходить из Rust-функции, которую вызывает C: начиная с Rust 1.81 это аварийно завершает процесс. Если вызывающая сторона должна получить код ошибки, а не падение, оберни тело `extern "C" fn` в `std::panic::catch_unwind` и преобразуй панику в код ошибки; при `panic = "abort"` в профиле сборки `catch_unwind` панику не перехватит. Указатели из C проверяй на null до разыменования, строки принимай через `CStr::from_ptr` и не храни ссылки на чужую память дольше вызова.

## Async

Используй рантайм и примитивы, уже выбранные в проекте; не смешивай рантаймы. Не вызывай блокирующие операции (`std::fs`, `std::thread::sleep`, тяжёлые вычисления) внутри async-функций — используй асинхронные аналоги рантайма или вынос в `spawn_blocking`. Не держи guard от `std::sync::Mutex` через `.await` (Clippy: `await_holding_lock`); сократи область блокировки или используй асинхронный мьютекс, если блокировка действительно должна пережить `.await`.

Асинхронные методы трейтов (`impl Future + Send` вместо `#[async_trait]`) — в [api.md](api.md#трейты-документация-и-async).

## Источники

- `unsafe`: [The Rustonomicon](https://doc.rust-lang.org/nomicon/), [Clippy — undocumented_unsafe_blocks](https://rust-lang.github.io/rust-clippy/stable/index.html#undocumented_unsafe_blocks), [API Guidelines — Function docs include error, panic, and safety considerations](https://rust-lang.github.io/api-guidelines/documentation.html#c-failure), [Miri](https://github.com/rust-lang/miri).
- FFI: [Rust 1.81 — abort при панике в `extern "C"`](https://blog.rust-lang.org/2024/09/05/Rust-1.81.0/), [std::panic::catch_unwind](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html), [std::ffi::CStr](https://doc.rust-lang.org/std/ffi/struct.CStr.html).
- Async: [Clippy — await_holding_lock](https://rust-lang.github.io/rust-clippy/stable/index.html#await_holding_lock).
