use rhosql::sqlite::{DataType, OpenFlag, SqliteHandle, StatementExt, StatementHandle, StepResult};

// https://sqlite.org/cintro.html#summary

fn main() -> rhosql::Result<()> {
    let db = SqliteHandle::open_v2(c":memory:", OpenFlag::OPEN_READWRITE_CREATE)?;

    let stmt = StatementHandle::prepare_v2(&db, c"select 420,'content',?1")?;

    stmt.bind_text(1, c"GG")?;

    assert_eq!(stmt.step()?, StepResult::Row);

    assert_eq!(stmt.data_count(), 3);

    assert_eq!(stmt.column_type(0), DataType::Int);
    assert_eq!(stmt.column_type(1), DataType::Text);
    assert_eq!(stmt.column_type(2), DataType::Text);

    assert_eq!(stmt.column_int(0), 420);
    assert_eq!(stmt.column_text(1)?, "content");
    assert_eq!(stmt.column_text(2)?, "GG");

    assert_eq!(stmt.step()?, StepResult::Done);

    Ok(())
}

