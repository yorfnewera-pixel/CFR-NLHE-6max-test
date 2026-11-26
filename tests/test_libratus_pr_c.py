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
