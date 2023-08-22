import os
import sqlite3
import typing


DeepFixSample = typing.NamedTuple(
    "DeepFixSample",
    [
        ("code_id", str),
        ("user_id", str),
        ("problem_id", str),
        ("code", str),
        ("error", str),
        ("errorcount", str),
    ],
)

DEEP_FIX_DB_NAME = "prutor-deepfix-09-12-2017.db"


def _load():
    if not os.path.exists(DEEP_FIX_DB_NAME):
        os.system(f"gzip -k -d {DEEP_FIX_DB_NAME}.gz")
    conn = sqlite3.connect(DEEP_FIX_DB_NAME)
    cursor = conn.cursor()
    cursor.execute(
        "SELECT `code_id`, `user_id`, `problem_id`, `code`, `error`, `errorcount` FROM `Code` WHERE `errorcount` != 0;"
    )
    return cursor


def load_iter() -> typing.Iterable[DeepFixSample]:
    cursor = _load()
    while True:
        row = cursor.fetchone()
        if row is None:
            break
        yield DeepFixSample(*row)


def load_all() -> list[DeepFixSample]:
    return list(load_iter())


def load_all_with_error() -> list[DeepFixSample]:
    return list(filter(lambda x: x.errorcount, load_iter()))
