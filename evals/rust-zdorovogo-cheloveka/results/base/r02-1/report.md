## Исправлены все три критические уязвимости в библиотеке pak

### Изменения в `/tmp/ws/base/r02-1/src/lib.rs` (96 новых строк, 8 удалённых):

**1. Переполнение при парсинге (attempt to add with overflow)**
   - Заменена `pos += ENTRY_LEN` на `pos.checked_add(ENTRY_LEN)?`
   - Добавлена проверка `checked_add()` при вычислении `offset + len`

**2. OOM уязвимость (съел всю память на 12-байтном файле)**
   - Добавлен лимит: максимум 65536 записей в архиве
   - Проверка `if count > 65536` перед выделением памяти
   - Возвращается новый тип ошибки `Error::TooManyEntries`

**3. Неправильные байты в записях (усечение offset)**
   - Добавлена проверка `if offset_raw > u32::MAX`
   - Вместо `as u32` усечения теперь явная валидация offset
   - Возвращается новый тип ошибки `Error::InvalidOffset`

**4. Дополнительные улучшения:**
   - Изменена структура `Entry` с `u32` на `usize` для правильной работы слайсов
   - Добавлены проверки overflow в методе `get()` с использованием `checked_add()`

### Добавлены новые типы ошибок:
- `TooManyEntries` - архив содержит больше 65536 записей
- `InvalidOffset` - offset выходит за границы u32::MAX

### Результаты тестирования:
```
running 6 tests
test tests::reads_entries ... ok
test tests::get_with_large_offsets ... ok  
test tests::rejects_bad_input ... ok
test tests::rejects_invalid_offset ... ok
test tests::rejects_overflow_in_offset_plus_len ... ok
test tests::rejects_too_many_entries ... ok

test result: ok. 6 passed; 0 failed
```

✓ Все проверки успешны
✓ Существующие тесты продолжают проходить
✓ Добавлены тесты для всех трёх уязвимостей
✓ Код компилируется без ошибок и warnings
✓ На любом входе parse() возвращает Error вместо panic/undefined behavior
