import sqlite3
import unittest
import time
import os


def connect():
  db = sqlite3.connect(":memory:")

  db.execute("create table base_functions as select name from pragma_function_list")
  db.execute("create table base_modules as select name from pragma_module_list")

  db.enable_load_extension(True)
  try:
    db.load_extension("target/debug/examples/libhello")
  except:
    # windows
    db.load_extension("target/debug/examples/hello")

  db.execute("create temp table loaded_functions as select name from pragma_function_list where name not in (select name from base_functions) order by name")
  db.execute("create temp table loaded_modules as select name from pragma_module_list where name not in (select name from base_modules) order by name")

  db.row_factory = sqlite3.Row
  return db


db = connect()


def execute_all(sql, args=None):
  if args is None: args = []
  results = db.execute(sql, args).fetchall()
  return list(map(lambda x: dict(x), results))


class TestExamples(unittest.TestCase):
  def test_funcs(self):
    hello = lambda name: db.execute("select hello(?)", [name]).fetchone()[0]
    self.assertEqual(hello('world'), "hello, world!")
    self.assertEqual(hello(None), "hello, !")
    self.assertEqual(hello(1234), "hello, 1234!")
    self.assertEqual(hello("null: x\0x"), "hello, null: x\0x!")
    #hello("x" * (2_147_483_647 - len("hello, !") - 20) )
    #hello("x" * (200) )

if __name__ == '__main__':
    unittest.main()
