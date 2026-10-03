Успешно добавлен статус заказа "Returned" (возвращён). Изменены 3 файла:

**src/status.rs**: Добавлен вариант `Returned` в enum Status с кодом "returned" и подписью "возвращён"

**src/policy.rs**: 
- is_open() теперь считает Returned закрытым статусом
- can_move() разрешает только переход Delivered → Returned
- refund() возвращает полную сумму для Returned заказов
- Добавлен тест для проверки правил Returned статуса

**src/report.rs**: 
- Обновлен тест summary_counts для нового формата
- Добавлен тест returned_orders_not_in_revenue

Все 5 тестов успешно пройдены:
✅ codes_round_trip
✅ lifecycle
✅ returned_status (новый)
✅ summary_counts
✅ returned_orders_not_in_revenue (новый)

Заказы со статусом "returned" отображаются в сводке отдельной строкой, полностью возвращаются покупателю, не входят в выручку, но считаются в возвратах.
