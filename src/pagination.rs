use crate::{error::PointercrateError, model::Model};
use diesel::{
    pg::Pg,
    query_builder::{BoxedSelectStatement, QueryFragment},
    sql_types::HasSqlType,
    Expression, PgConnection, QuerySource,
};

pub trait Paginatable: Clone {
    type ColumnType;
    type Result: Model;

    fn filter<'a, ST>(
        &'a self, query: BoxedSelectStatement<'a, ST, <Self::Result as Model>::Table, Pg>,
    ) -> BoxedSelectStatement<'a, ST, <Self::Result as Model>::Table, Pg>;

    fn query(
        &self,
    ) -> BoxedSelectStatement<
        <<Self::Result as Model>::Columns as Expression>::SqlType,
        <Self::Result as Model>::Table,
        Pg,
    >;

    fn result(&self, connection: &PgConnection) -> Result<Vec<Self::Result>, PointercrateError>
    where
        Self::Result: diesel::Queryable<
            <<Self::Result as Model>::Columns as Expression>::SqlType,
            Pg,
        >,
        Pg: HasSqlType<<<Self::Result as Model>::Columns as Expression>::SqlType>,
        <<Self::Result as Model>::Table as QuerySource>::FromClause: QueryFragment<diesel::pg::Pg>,
    {
        use diesel::RunQueryDsl;

        self.query()
            .load(connection)
            .map_err(PointercrateError::database)
    }

    /// Gets the `after` value for the query in the `next` link
    ///
    /// + If a `before` value is currently set, `after` will be `Some(before - 1)` if there exists
    /// any object with `id >= before`, or `None` otherwise
    /// + Otherwise, we try to get `limit.unwrap_or(50) + 1` objects and either return the ID of
    /// the (limits + 1)th object - 1, or `None` if the object doesn't exist
    fn next_after(
        &self, conn: &PgConnection,
    ) -> Result<Option<Self::ColumnType>, PointercrateError>;

    /// Gets the `before` value for the query in the `prev` link
    ///
    /// + If a `after` value is currently set, `before` will be `Some(after + 1)` if there exists
    /// any object with `id <= after` or `None` otherwise
    /// + Otherwise, we try to get `limit.unwrap_or(50) + 1` objects in reversed order and
    /// either return the (limits + 1)th object + 1, or `None` if the object doesn't exist
    fn prev_before(
        &self, conn: &PgConnection,
    ) -> Result<Option<Self::ColumnType>, PointercrateError>;

    fn first(&self, conn: &PgConnection) -> Result<Option<Self::ColumnType>, PointercrateError>;
    fn last(&self, conn: &PgConnection) -> Result<Option<Self::ColumnType>, PointercrateError>;

    fn clone_with(&self, after: Option<Self::ColumnType>, before: Option<Self::ColumnType>)
        -> Self;
}