from src.blueprint.abstraction import _libratus_card_descriptor
from src.blueprint.equity_buckets import class_key


def test_точный_флоп_сохраняет_канонический_класс():
    hole = ("2c", "3d")
    board = ("4h", "5s", "7c")
    assert _libratus_card_descriptor(hole, board, None) == ("flop", class_key(hole, board))
    assert _libratus_card_descriptor(hole, board, None) == ("flop", 4951760194179124435628326916)


def test_поздняя_улица_требует_таблицу():
    import pytest
    from src.blueprint.equity_buckets import EquityCards

    with pytest.raises(ValueError):
        _libratus_card_descriptor(("2c", "3d"), ("4h", "5s", "7c", "8d"), None)
    with pytest.raises(ValueError):
        _libratus_card_descriptor(("2c", "3d"), ("4h", "5s", "7c", "8d"), EquityCards.__new__(EquityCards))


def test_загрузчик_k50_отклоняет_новую_схему(tmp_path):
    import pytest
    from src.blueprint.abstraction import HU20_LIBRATUS_SCHEMA
    from src.blueprint.equity_buckets import load_cards

    for directory in (None, tmp_path):
        with pytest.raises(ValueError):
            load_cards(HU20_LIBRATUS_SCHEMA, {}, directory)


def test_диагностика_явно_отклоняет_новую_схему():
    import pytest
    from src.blueprint.abstraction import HU20_LIBRATUS_SCHEMA
    from src.blueprint.average import checked_header

    with pytest.raises(ValueError, match="Unknown A/C diagnostic schema"):
        checked_header({}, {}, expected_schema=HU20_LIBRATUS_SCHEMA)


def test_каталог_арены_явно_отклоняет_новую_схему():
    import pytest
    from src.arena.catalog import Checkpoint
    from src.blueprint.abstraction import HU20_LIBRATUS_SCHEMA

    with pytest.raises(ValueError, match="Unsupported checkpoint abstraction"):
        Checkpoint("test", "model.gz", "a" * 64, abstraction=HU20_LIBRATUS_SCHEMA)


def test_web_явно_отклоняет_новую_схему():
    from types import SimpleNamespace
    import pytest
    from src.blueprint.abstraction import HU20_LIBRATUS_SCHEMA
    from src.blueprint.solver import HU20_UNCAPPED_GAME
    from src.play_api.configuration import PlayTable

    policy = SimpleNamespace(game=HU20_UNCAPPED_GAME, abstraction=HU20_LIBRATUS_SCHEMA,
                             players=2, raise_cap=None)
    with pytest.raises(ValueError, match="incompatible"):
        PlayTable().validate_policy(policy)


def test_поиск_и_lbr_явно_отклоняют_новую_схему():
    from types import SimpleNamespace
    import pytest
    from src.blueprint.abstraction import HU20_LIBRATUS_SCHEMA
    from src.blueprint.search import LiveBlueprint, SearchPlayer
    from src.blueprint.hu20_turn_search import HU20TurnSearchPolicy
    from src.diagnostics.robustness import LocalBestResponse

    source = SimpleNamespace(abstraction=HU20_LIBRATUS_SCHEMA)
    trainer = SimpleNamespace(config=SimpleNamespace(abstraction=HU20_LIBRATUS_SCHEMA))
    for create in (lambda: LiveBlueprint(trainer), lambda: SearchPlayer(source, 1),
                   lambda: HU20TurnSearchPolicy(source, None), lambda: LocalBestResponse(source, 1)):
        with pytest.raises(ValueError, match="Libratus"):
            create()


def test_экспорт_читается_штатным_загрузчиком(monkeypatch):
    import os
    import hashlib
    from pathlib import Path
    import pytest
    from src.arena.catalog import Checkpoint
    from src.blueprint import equity_buckets
    from src.blueprint.abstraction import HU20_LIBRATUS_SCHEMA
    from src.blueprint.artifact import FrozenBlueprint, HU20_UNCAPPED_FORMAT

    directory = os.environ.get("HU20_LIBRATUS_TEST_DIR")
    if directory is None:
        pytest.skip("не заданы тестовые таблицы и экспорт")
    directory = Path(directory)
    policy = directory / "policy.gz"
    spec = Checkpoint("test", str(policy), hashlib.sha256(policy.read_bytes()).hexdigest(), HU20_UNCAPPED_FORMAT)
    monkeypatch.setattr(equity_buckets, "_require_libratus_production_deck", lambda manifest: None)
    loaded = FrozenBlueprint(spec, policy, card_buckets=directory)
    assert loaded.abstraction == HU20_LIBRATUS_SCHEMA
    assert len(loaded.entries) == 1
    from src.game.hand import Hand, Table
    from src.game.types import Action, ActionKind

    deck = tuple(rank + suit for rank in "89TJQKA234567" for suit in "cdhs")
    hand = Hand.from_deck(Table(("a", "b"), (2000, 2000)), hand_id="reader", deck=deck)
    for street in range(4):
        actor = hand.actor
        assert actor is not None
        menu, probabilities, found = loaded.distribution(hand.observe(actor))
        assert len(menu) == len(probabilities)
        assert found == (street == 1)
        if found:
            assert len(menu) > 1
            assert probabilities == (1.0,) + (0.0,) * (len(menu) - 1)
        if street < 3:
            hand = hand.apply(Action(ActionKind.CALL if street == 0 else ActionKind.CHECK))
            hand = hand.apply(Action(ActionKind.CHECK))


def test_экспорт_отклоняет_подмену_меню_до_таблиц(tmp_path, monkeypatch):
    import gzip
    import hashlib
    import json
    import os
    from pathlib import Path
    from types import SimpleNamespace
    import pytest
    from src.blueprint import equity_buckets
    from src.blueprint.abstraction import HU20_LIBRATUS_SCHEMA
    from src.blueprint.artifact import FrozenBlueprint, HU20_UNCAPPED_FORMAT

    root = os.environ.get("HU20_LIBRATUS_TEST_DIR")
    if root is None:
        pytest.skip("не задан тестовый экспорт")
    with gzip.open(Path(root) / "policy.gz", "rt") as stream:
        document = json.load(stream)
    document["identity"]["action_menu"] = "v2"
    path = tmp_path / "wrong-menu.gz"
    path.write_bytes(gzip.compress(json.dumps(document).encode()))
    spec = SimpleNamespace(sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                           format=HU20_UNCAPPED_FORMAT, abstraction=HU20_LIBRATUS_SCHEMA)
    monkeypatch.setattr(equity_buckets, "load_libratus_cards", lambda *args: pytest.fail("таблицы открыты до проверки меню"))
    with pytest.raises(ValueError, match="Идентичность"):
        FrozenBlueprint(spec, path, card_buckets=tmp_path)


def test_экспорт_отклоняет_подмену_таблицы(tmp_path, monkeypatch):
    import hashlib
    import os
    import shutil
    from pathlib import Path
    from types import SimpleNamespace
    import pytest
    from src.blueprint import equity_buckets
    from src.blueprint.abstraction import HU20_LIBRATUS_SCHEMA
    from src.blueprint.artifact import FrozenBlueprint, HU20_UNCAPPED_FORMAT

    root = os.environ.get("HU20_LIBRATUS_TEST_DIR")
    if root is None:
        pytest.skip("не заданы тестовые таблицы")
    root = Path(root)
    for name in ("policy.gz", "libratus-turn.bin", "libratus-turn.json", "libratus-river.bin", "libratus-river.json"):
        shutil.copyfile(root / name, tmp_path / name)
    table = tmp_path / "libratus-turn.bin"
    with table.open("r+b") as stream:
        stream.seek(-1, 2)
        last = stream.read(1)
        stream.seek(-1, 2)
        stream.write(bytes([last[0] ^ 1]))
    policy = tmp_path / "policy.gz"
    spec = SimpleNamespace(sha256=hashlib.sha256(policy.read_bytes()).hexdigest(),
                           format=HU20_UNCAPPED_FORMAT, abstraction=HU20_LIBRATUS_SCHEMA)
    monkeypatch.setattr(equity_buckets, "_require_libratus_production_deck", lambda manifest: None)
    with pytest.raises(ValueError):
        FrozenBlueprint(spec, policy, card_buckets=tmp_path)


def test_рабочий_загрузчик_отклоняет_малую_и_непроверенную_колоду(tmp_path):
    import json
    import os
    import shutil
    from pathlib import Path
    import pytest
    from src.blueprint.equity_buckets import LibratusCards

    root = os.environ.get("HU20_LIBRATUS_TEST_DIR")
    if root is None:
        pytest.skip("не заданы тестовые таблицы")
    for name in ("libratus-turn.bin", "libratus-turn.json", "libratus-river.bin", "libratus-river.json"):
        shutil.copyfile(Path(root) / name, tmp_path / name)
    with pytest.raises(ValueError, match="полную колоду"):
        LibratusCards(tmp_path)
    manifest = tmp_path / "libratus-turn.json"
    data = json.loads(manifest.read_text())
    data["ranks"] = 13
    manifest.write_text(json.dumps(data))
    with pytest.raises(ValueError, match="unsupported deck"):
        LibratusCards(tmp_path)


def test_штатный_cli_отклоняет_малую_колоду_и_меню_v2(tmp_path):
    import os
    import subprocess
    from pathlib import Path
    import pytest

    root = os.environ.get("HU20_LIBRATUS_TEST_DIR")
    binary = Path("native/hu20-trainer/target/release/hu20-trainer")
    if root is None or not binary.is_file():
        pytest.skip("нет Linux trainer или тестовых таблиц")
    fixtures = tmp_path / "empty.jsonl"
    fixtures.write_text("")
    result = subprocess.run([str(binary), "parity", str(fixtures), "--libratus-buckets", root], capture_output=True, text=True)
    assert result.returncode != 0 and "неподдерживаемая колода" in result.stderr
    result = subprocess.run([str(binary), "parity", str(fixtures), "--libratus-buckets", root, "--menu", "v2"], capture_output=True, text=True)
    assert result.returncode != 0 and "меню V1" in result.stderr


def test_экспорт_отклоняет_другие_проверенные_таблицы(monkeypatch):
    import hashlib
    import os
    from pathlib import Path
    from types import SimpleNamespace
    import pytest
    from src.blueprint import equity_buckets
    from src.blueprint.abstraction import HU20_LIBRATUS_SCHEMA
    from src.blueprint.artifact import FrozenBlueprint, HU20_UNCAPPED_FORMAT

    root = os.environ.get("HU20_LIBRATUS_TEST_DIR")
    other = os.environ.get("HU20_LIBRATUS_OTHER_TEST_DIR")
    if root is None or other is None:
        pytest.skip("не заданы проверенные тестовые таблицы")
    policy = Path(root) / "policy.gz"
    spec = SimpleNamespace(sha256=hashlib.sha256(policy.read_bytes()).hexdigest(),
                           format=HU20_UNCAPPED_FORMAT, abstraction=HU20_LIBRATUS_SCHEMA)
    monkeypatch.setattr(equity_buckets, "_require_libratus_production_deck", lambda manifest: None)
    with pytest.raises(ValueError, match="Идентичность таблиц"):
        FrozenBlueprint(spec, policy, card_buckets=Path(other))


def test_полные_ключи_для_каждой_улицы(tmp_path):
    import pytest
    from src.blueprint.abstraction import HU20_LIBRATUS_SCHEMA, information_key, schema_choices
    from src.blueprint.equity_buckets import LibratusBucketTable, hash_key
    from src.game.hand import Hand, Table
    from src.game.types import Action, ActionKind

    class Таблицы:
        def bucket(self, hole, board):
            street = len(board) - 2
            key = hash_key(class_key(hole, board))
            path = tmp_path / f"street-{street}.bin"
            path.write_bytes(b"HU20LIB1" + (1).to_bytes(4, "little") + street.to_bytes(4, "little")
                             + (2).to_bytes(4, "little") + (1).to_bytes(8, "little")
                             + key.to_bytes(8, "little") + (0).to_bytes(2, "little"))
            table = LibratusBucketTable.__new__(LibratusBucketTable)
            table._load(path, None)
            return table.bucket(hole, board)

    deck = tuple(rank + suit for rank in "23456789TJQKA" for suit in "cdhs")
    hand = Hand.from_deck(Table(("a", "b"), (2000, 2000)), hand_id="parity", deck=deck)
    keys = []
    for street in range(4):
        view = hand.observe(hand.actor)
        menu = schema_choices(view, HU20_LIBRATUS_SCHEMA)
        with pytest.raises(ValueError):
            information_key(view, menu[:-1], schema=HU20_LIBRATUS_SCHEMA, libratus_cards=Таблицы())
        keys.append(information_key(view, menu, schema=HU20_LIBRATUS_SCHEMA, libratus_cards=Таблицы()))
        if street < 3:
            hand = hand.apply(Action(ActionKind.CALL if street == 0 else ActionKind.CHECK))
            hand = hand.apply(Action(ActionKind.CHECK))
    assert keys == ["3a2e2576b354d4b7e06966379d968b7c", "8a5cee923c9b932044194acad102f1d4",
                    "b9ead3bbc5804ab562074726285383cf", "abd7d303ed40aee625561a3771264876"]
    other = Hand.from_deck(Table(("a", "b"), (10000, 10000)), hand_id="other", deck=deck)
    view = other.observe(other.actor)
    with pytest.raises(ValueError):
        information_key(view, schema_choices(view, HU20_LIBRATUS_SCHEMA), schema=HU20_LIBRATUS_SCHEMA)
