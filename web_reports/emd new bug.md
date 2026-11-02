Контрольный прогон выполнен один раз и не засчитан: generator завершился с exit 101.

Причина — новая защитная ветка NoProgress в первом flop assignment:
- stage: flop-clustering
- phase: assignment
- point: 1
- centroid: 9
- final status: flop-clustering, не complete
- созданы только river/turn таблицы; summary.json и flop-таблица отсутствуют
- диагностика сохранена в emd-failure-flop-clustering.txt
- launcher корректно записал exit_code=101 и сообщил отказ; дочерний процесс завершён.
Это не прежний пороговый DisconnectedTransportGraph, а конкретное срабатывание нового критерия прогресса. Повторный прогон не запускал.

Следующий шаг — test-only диагностика этого сохранённого случая: определить, почему положительная augmentation не изменила delivered, и уточнить критерий NoProgress без маскирования неполного транспорта.