import sqlite3
import unittest
import time
import os


def connect():
  db = sqlite3.connect(":memory:")

  db.execute("create table base_functions as select name from pragma_function_list")
  db.execute("create table base_modules as select name from pragma_module_list")

  db.enable_load_extension(True)
  db.load_extension("target/release/examples/libhello")

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
    self.assertEqual(execute_all("select hello('world'), hello('Alex'), hello(1234);"), [
      {
        "hello('Alex')": 'hello, Alex!',
        "hello('world')": 'hello, world!',
        'hello(1234)': 'hello, 1234!'
    }
   ])
if __name__ == '__main__':
    unittest.main()