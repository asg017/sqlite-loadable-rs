//! Opininated parsing for SQLite virtual table constructor arguments.
//!
//! A "constructor" comes from the CREATE VIRTUAL TABLE statement
//! of a virtual table, like:
//! ```sql
//! CREATE VIRTUAL TABLE xxx USING custom_vtab(
//!   mode="production",
//!   state=null,
//!   name TEXT,
//!   age INTEGER,
//!   progress REAL
//! )
//! ```
//!
//! sqlite_loadable passes down the arguments between `custom_vtab(...)`
//! as a vector of strings within `VTabArguments.arguments`, where each
//! comma-seperated argument is its own element in the vector.
//!
//! Virtual table statements are allowed to parse these arguments however
//! they want, and this module is one opinionated option, loosely based
//! on [FTS5 virtual tables](https://www.sqlite.org/fts5.html).
//!

use crate::api::ColumnAffinity;

/// A successfully parsed Argument from a virtual table constructor.
/// A single constructor can have multiple arguments, this struct
/// only represents a single one.
///
/// In this parser, the above constructor in `xxx` has 5 arguments -
/// 2 "configuration options" and "column declarations." The `mode`
/// argument is a configuration option with a key of `"mode"` and
/// a quoted string value of `"production"`. Similarly, `state` is
/// a configuration argument with key `"state"` and a bareword
/// value of `null`. On the other hand, `name`, `age`, and `progress`
/// are arguments that declare columns, with declared types `text`,
/// `integer`, and `real`, respectfully.
///
/// The virtual table implementations can do whatever they want with
/// the parsed arguments, including by not limited to erroring on
/// invalid options, creating new columns on the virtual table
/// based on the column definitions, requiring certain config options,
/// or anything else they want.

/// A single parsed argument from a virtual table constructor. Can
/// be a column declaration onf configuration option.
#[derive(Debug, PartialEq, Eq)]
pub enum Argument {
    /// The argument declares a column - ex "name text" or "age integer".
    /// Like SQLite, a column declartion can have 0 or any declared types,
    /// and also tries to capture any "constraints".
    Column(ColumnDeclaration),

    /// The argument defines a configuration option - ex "mode=fast"
    /// or "tokenize = 'porter ascii'". The key is always a string,
    /// the value can be "rich" types like strings, booleans, numbers,
    /// sqlite_parameters, or barewords.
    Config(ConfigOption),
    // TODO support wildcard column selection? '* EXCLUDE', '* REPLACE',
    // maybe 'COLUMNS(/only_/)', etc.
}
/// A column declaration that defines a single column.
/// Example: `"name text"` would parse to a  column with the name `"name"`
/// and declared type of `"text"`.

// TODO can this also support "aliased" or "computed/generated" columns,
// like "'/item/name' as name" (xml) or "FirstName as first_name" (csv)
// or "'$.name.first' as first_name" (json)?
#[derive(Debug, PartialEq, Eq)]
pub struct ColumnDeclaration {
    /// Name of declared column
    pub name: String,
    // Declared type of the column
    pub declared_type: Option<String>,
    pub constraints: Option<String>,
}
impl ColumnDeclaration {
    fn new(
        name: &str,
        declared_type: Option<&str>,
        constraints: Option<&str>,
    ) -> ColumnDeclaration {
        ColumnDeclaration {
            name: name.to_owned(),
            declared_type: declared_type.map(|d| d.to_owned()),
            constraints: constraints.map(|d| d.to_owned()),
        }
    }

    /// Determines the column declaration's "affinity", based on
    /// the parsed declared type. Uses the same rules as
    /// <https://www.sqlite.org/datatype3.html#determination_of_column_affinity>.
    pub fn affinity(&self) -> ColumnAffinity {
        match &self.declared_type {
            Some(declared_type) => ColumnAffinity::from_declared_type(declared_type.as_str()),
            None => crate::api::ColumnAffinity::Numeric,
        }
    }

    /// Formats the column declaration into a way that a CREATE TABLE
    /// statement expects ("escaping" the column name).
    // TODO is this safe lol
    pub fn vtab_declaration(&self) -> String {
        format!(
            "'{}' {}",
            self.name,
            self.declared_type.as_ref().map_or("", |d| d.as_str())
        )
    }
}

/// A parsed configuration option, that always contain a key/value
/// pair. These can be used as "table-options" to configure special
/// behavior or settings for the virtual table implementation.
/// Example: the `tokenize` and `prefix` config options on FTS5
/// virtual tables <https://www.sqlite.org/fts5.html#fts5_table_creation_and_initialization>
#[derive(Debug, PartialEq, Eq)]
pub struct ConfigOption {
    pub key: String,
    pub value: ConfigOptionValue,
}

/// Possible options for the "values" of configuration options.
///
#[derive(Debug, PartialEq, Eq)]
pub enum ConfigOptionValue {
    ///
    Quoted(String),
    ///
    SqliteParameter(String),
    ///
    Bareword(String),
}

/// Given a raw argument, returns a parsed [`Argument`]. Should already by
/// comma (?) delimited, typically sourced from [`VTabArguments`](crate::table::VTabArguments).
pub fn parse_argument(argument: &str) -> std::result::Result<Argument, String> {
    match arg_is_config_option(argument) {
        Ok(Some(config_option)) => return Ok(Argument::Config(config_option)),
        Ok(None) => (),
        Err(err) => return Err(err),
    };
    match arg_is_column_declaration(argument) {
        Ok(Some(column_declaration)) => return Ok(Argument::Column(column_declaration)),
        Ok(None) => (),
        Err(err) => return Err(err),
    };
    Err("argument is neither a configuration option or column declaration.".to_owned())
}

/// TODO renamed "parameter" to "named argument"
fn arg_is_config_option(arg: &str) -> Result<Option<ConfigOption>, String> {
    let mut split = arg.split('=');
    let key = match split.next() {
        Some(k) => k,
        None => return Ok(None),
    };
    let value = match split.next() {
        Some(k) => k,
        None => return Ok(None),
    };
    Ok(Some(ConfigOption {
        key: key.to_owned(),
        value: parse_config_option_value(key.to_string(), value)?,
    }))
}
fn parse_config_option_value(key: String, value: &str) -> Result<ConfigOptionValue, String> {
    let value = value.trim();
    match value.chars().next() {
        Some('\'') | Some('"') => {
            // TODO ensure last starts with quote
            let mut chars = value.chars();
            chars.next();
            chars.next_back();
            Ok(ConfigOptionValue::Quoted(chars.as_str().to_owned()))
        }
        Some(':') | Some('@') => {
            // TODO ensure it's a proper sqlite_parameter
            // (not start with digit?? or spaces??)
            Ok(ConfigOptionValue::SqliteParameter(value.to_owned()))
        }
        Some(_) => {
            // TODO ensure there's no quote words in bareword?
            Ok(ConfigOptionValue::Bareword(value.to_owned()))
        }
        None => Err(format!("Empty value for key '{}'", key)),
    }
}
pub fn arg_is_column_declaration(arg: &str) -> Result<Option<ColumnDeclaration>, String> {
    if arg.trim().is_empty() {
        return Ok(None);
    }
    let mut split = arg.split(' ');
    let name = split.next().ok_or("asdf")?;
    let declared_type = split.next();
    let constraints = None;
    Ok(Some(ColumnDeclaration::new(
        name,
        declared_type,
        constraints,
    )))
}

#[cfg(test)]
mod tests {
    use crate::vtab_argparse::*;
    #[test]
    fn test_parse_argument() {
        assert_eq!(
            parse_argument("name text"),
            Ok(Argument::Column(ColumnDeclaration::new(
                "name",
                Some("text"),
                None,
            )))
        );
        assert_eq!(
            parse_argument("name"),
            Ok(Argument::Column(
                ColumnDeclaration::new("name", None, None,)
            ))
        );
        assert_eq!(
            parse_argument("option='quoted'"),
            Ok(Argument::Config(ConfigOption {
                key: "option".to_owned(),
                value: ConfigOptionValue::Quoted("quoted".to_owned())
            }))
        );
        assert_eq!(
            parse_argument("option=:param"),
            Ok(Argument::Config(ConfigOption {
                key: "option".to_owned(),
                value: ConfigOptionValue::SqliteParameter(":param".to_owned())
            }))
        );
        assert_eq!(
            parse_argument("option=bareword"),
            Ok(Argument::Config(ConfigOption {
                key: "option".to_owned(),
                value: ConfigOptionValue::Bareword("bareword".to_owned())
            }))
        );
    }
}
