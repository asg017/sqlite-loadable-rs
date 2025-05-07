//! cargo build --example series
//! sqlite3 :memory: '.read examples/test.sql'

use sqlite_loadable::prelude::*;

use sqlite_loadable::{
    api, define_table_function,
    table::{BestIndexError, ConstraintOperator, IndexInfo, VTab, VTabArguments, VTabCursor},
    Result,
};

use std::mem;

static CREATE_SQL: &str = "CREATE TABLE x(value, input hidden)";
enum Columns {
    Value,
    Input,
}
fn column(index: i32) -> Option<Columns> {
    match index {
        0 => Some(Columns::Value),
        1 => Some(Columns::Input),
        _ => None,
    }
}

#[repr(C)]
pub struct CharactersTable {
    /// must be first
    base: sqlite3_vtab,
}

impl<'vtab> VTab<'vtab> for CharactersTable {
    type Aux = ();
    type Cursor = CharactersCursor;

    fn connect(
        _db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, CharactersTable)> {
        let vtab = CharactersTable { base: unsafe { mem::zeroed() } };
        // TODO db.config(VTabConfig::Innocuous)?;
        Ok((CREATE_SQL.to_owned(), vtab))
    }
    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn best_index(&self, mut info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        let mut has_input = false;
        for mut constraint in info.constraints() {
            match column(constraint.column_idx()) {
                Some(Columns::Input) => {
                    if constraint.usable() && constraint.op() == Some(ConstraintOperator::EQ) {
                        constraint.set_omit(true);
                        constraint.set_argv_index(1);
                        has_input = true;
                    } else {
                        return Err(BestIndexError::Constraint);
                    }
                }
                _ => todo!(),
            }
        }
        if !has_input {
            return Err(BestIndexError::Error);
        }
        info.set_estimated_cost(100000.0);
        info.set_estimated_rows(100000);
        info.set_idxnum(1);

        Ok(())
    }

    fn open(&mut self) -> Result<CharactersCursor> {
        Ok(CharactersCursor::new())
    }
}

#[repr(C)]
pub struct CharactersCursor {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    input: Option<String>,
    characters: Option<Vec<char>>,
    idx: usize,
}
impl CharactersCursor {
    fn new() -> CharactersCursor {
        CharactersCursor {
            base: unsafe { mem::zeroed() },
            input: None,
            characters: None,
            idx: 0,
        }
    }
}

impl VTabCursor for CharactersCursor {
    fn filter(
        &mut self,
        _idx_num: i32,
        _idx_str: Option<&str>,
        values: &[*mut sqlite3_value],
    ) -> Result<()> {
        let input =
            api::value_text(values.get(0).expect("1st input constraint is required"))?.to_owned();
        self.characters = Some(input.chars().collect());
        self.input = Some(input);
        self.idx = 0;
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.idx += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        match &self.characters {
            Some(chars) => chars.get(self.idx).is_none(),
            None => true,
        }
    }

    fn column(&self, context: *mut sqlite3_context, i: i32) -> Result<()> {
        match column(i) {
            Some(Columns::Value) => {
                api::result_text(
                    context,
                    self.characters
                        .as_ref()
                        .unwrap()
                        .get(self.idx)
                        .unwrap()
                        .to_string()
                        .as_str(),
                )?;
            }
            Some(Columns::Input) => {
                api::result_text(context, self.input.as_ref().unwrap())?;
            }
            _ => (),
        }
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.idx as i64)
    }
}

#[sqlite_entrypoint]
pub fn sqlite3_characters_init(db: *mut sqlite3) -> Result<()> {
    define_table_function::<CharactersTable>(db, "characters", None)?;
    Ok(())
}
