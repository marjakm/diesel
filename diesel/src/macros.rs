// FIXME(https://github.com/rust-lang/rust/issues/19630) Remove this work-around
#[macro_export]
#[doc(hidden)]
macro_rules! diesel_internal_expr_conversion {
    ($e:expr) => { $e }
}

#[macro_export]
#[doc(hidden)]
macro_rules! column {
    ($($table:ident)::*, $column_name:ident -> $Type:ty) => {
        #[allow(non_camel_case_types, dead_code)]
        #[derive(Debug, Clone, Copy)]
        pub struct $column_name;

        impl $crate::expression::Expression for $column_name {
            type SqlType = $Type;
        }

        impl<DB> $crate::query_builder::QueryFragment<DB> for $column_name where
            DB: $crate::backend::Backend,
        {
            fn to_sql(&self, out: &mut DB::QueryBuilder) -> $crate::query_builder::BuildQueryResult {
                try!(out.push_identifier($($table)::*::name()));
                out.push_sql(".");
                out.push_identifier(stringify!($column_name))
            }
        }

        impl $crate::expression::SelectableExpression<$($table)::*> for $column_name {}

        impl<'a, ST, Left, Right> SelectableExpression<
            $crate::WithQuerySource<'a, Left, Right>, ST> for $column_name where
            $column_name: SelectableExpression<Left, ST>
        {
        }

        impl $crate::expression::NonAggregate for $column_name {}

        impl $crate::query_source::Column for $column_name {
            type Table = $($table)::*;

            fn name() -> &'static str {
                stringify!($column_name)
            }
        }
    }
}

/// Specifies that a table exists, and what columns it has. This will create a
/// new public module, with the same name, as the name of the table. In this
/// module, you'll find a unit struct named `table`, and a unit struct with the
/// names of each of the columns. In the definition, you can also specify an
/// additional set of columns which exist, but should not be selected by default
/// (for example, for things like full text search)
///
/// Example usage
/// -------------
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// table! {
///     users {
///         id -> Integer,
///         name -> VarChar,
///         favorite_color -> Nullable<VarChar>,
///     }
/// }
/// # fn main() {}
/// ```
///
/// More complex usage:
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// table! {
///     users (non_standard_primary_key) {
///         non_standard_primary_key -> Integer,
///         name -> VarChar,
///         favorite_color -> Nullable<VarChar>,
///     } no select {
///         complex_index_column -> Text,
///     }
/// }
/// # fn main() {}
/// ```
///
/// This module will also contain several helper types:
///
/// dsl
/// ---
///
/// This simply re-exports the table, renamed to the same name as the module,
/// and each of the columns. This is useful to glob import when you're dealing
/// primarily with one table, to allow writing `users.filter(name.eq("Sean"))`
/// instead of `users::table.filter(users::name.eq("Sean"))`.
///
/// all_columns
/// -----------
///
/// A constant will be assigned called `all_columns`, which will be a tuple of
/// all the columns that aren't in the "no select" group. This is what will be
/// selected if you don't otherwise specify a select clause. It's type will be
/// `table::AllColumns`. You can also get this value from the
/// `Table::all_columns` function.
///
/// star
/// ----
///
/// This will be the qualified "star" expression for this table (e.g.
/// `users.*`). Internally, we read columns by index, not by name, so this
/// column is not safe to read data out of, and it has had it's SQL type set to
/// `()` to prevent accidentally using it as such. It is sometimes useful for
/// count statements however. It can also be accessed through the `Table.star()`
/// method.
#[macro_export]
macro_rules! table {
    (
        $name:ident {
            $($column_name:ident -> $Type:ty,)+
        }
    ) => {
        table! {
            $name {
                $($column_name -> $Type,)+
            } custom_types {}
        }
    };
    (
        $name:ident {
            $($column_name:ident -> $Type:ty,)+
        } custom_types {
            $($(::$ut:ident)*,)*
        }
    ) => {
        table! {
            $name (id) {
                $($column_name -> $Type,)+
            } custom_types {
                $($(::$ut)*,)*
            }
        }
    };
    (
        $name:ident ($pk:ident) {
            $($column_name:ident -> $Type:ty,)+
        } custom_types {
            $($(::$ut:ident)*,)*
        }
    ) => {
        table! {
            $name ($pk) {
                $($column_name -> $Type,)+
            } no select {} custom_types {
                $($(::$ut)*,)*
            }
        }
    };
    (
        $name:ident ($pk:ident) {
            $($column_name:ident -> $Type:ty,)+
        }
    ) => {
        table! {
            $name ($pk) {
                $($column_name -> $Type,)+
            } no select {} custom_types {}
        }
    };
    (
        $name:ident {
            $($column_name:ident -> $Type:ty,)+
        } no select {
            $($no_select_column_name:ident -> $no_select_type:ty,)*
        } custom_types {
            $($(::$ut:ident)*,)*
        }
    ) => {
        table! {
            $name (id) {
                $($column_name -> $Type,)+
            } no select {
                $($no_select_column_name -> $no_select_type,)*
            } custom_types {
                $($(::$ut)*,)*
            }
        }
    };
    (
        $name:ident ($pk:ident) {
            $($column_name:ident -> $Type:ty,)+
        } no select {
            $($no_select_column_name:ident -> $no_select_type:ty,)*
        } custom_types {
            $($(::$ut:ident)*,)*
        }
    ) => {
        table_body! {
            $name ($pk) {
                $($column_name -> $Type,)+
            } no select {
                $($no_select_column_name -> $no_select_type,)*
            } custom_types {
                $($(::$ut)*,)*
            }
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! table_body {
    (
        $name:ident ($pk:ident) {
            $($column_name:ident -> $Type:ty,)+
        } no select {
            $($no_select_column_name:ident -> $no_select_type:ty,)*
        }
    ) => {
        table_body! {
            $name ($pk) {
                $($column_name -> $Type,)+
            } no select {
                $($no_select_column_name -> $no_select_type,)*
            } custom_types { }
        }
    };
    (
        $name:ident ($pk:ident) {
            $($column_name:ident -> $Type:ty,)+
        } no select {
            $($no_select_column_name:ident -> $no_select_type:ty,)*
        } custom_types {
            $($(::$ut:ident)*,)*
        }
    ) => {
        pub mod $name {
            $(use $(::$ut)*;)*
            use $crate::{
                QuerySource,
                Table,
            };
            use $crate::query_builder::*;
            use $crate::query_builder::nodes::Identifier;
            use $crate::types::*;
            pub use self::columns::*;

            pub mod dsl {
                pub use super::columns::{$($column_name),+};
                pub use super::table as $name;
            }

            #[allow(non_upper_case_globals, dead_code)]
            pub const all_columns: ($($column_name),+) = ($($column_name),+);

            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy)]
            pub struct table;

            impl table {
                #[allow(dead_code)]
                pub fn star(&self) -> star {
                    star
                }
            }

            pub type SqlType = ($($Type),+);

            impl QuerySource for table {
                type FromClause = Identifier<'static>;

                fn from_clause(&self) -> Self::FromClause {
                    Identifier(stringify!($name))
                }
            }

            impl AsQuery for table {
                type SqlType = SqlType;
                type Query = SelectStatement<SqlType, ($($column_name),+), Self>;

                fn as_query(self) -> Self::Query {
                    SelectStatement::simple(all_columns, self)
                }
            }

            impl Table for table {
                type PrimaryKey = columns::$pk;
                type AllColumns = ($($column_name),+);

                fn name() -> &'static str {
                    stringify!($name)
                }

                fn primary_key(&self) -> Self::PrimaryKey {
                    columns::$pk
                }

                fn all_columns() -> Self::AllColumns {
                    ($($column_name),+)
                }
            }

            pub mod columns {
                $(use $(::$ut)*;)*
                use super::table;
                use $crate::{Table, Column, Expression, SelectableExpression};
                use $crate::backend::Backend;
                use $crate::query_builder::{QueryBuilder, BuildQueryResult, QueryFragment};
                use $crate::types::*;

                #[allow(non_camel_case_types, dead_code)]
                #[derive(Debug, Clone, Copy)]
                pub struct star;

                impl Expression for star {
                    type SqlType = ();
                }

                impl<DB: Backend> QueryFragment<DB> for star {
                    fn to_sql(&self, out: &mut DB::QueryBuilder) -> BuildQueryResult {
                        try!(out.push_identifier(table::name()));
                        out.push_sql(".*");
                        Ok(())
                    }
                }

                impl SelectableExpression<table> for star {}

                $(column!(table, $column_name -> $Type);)+
                $(column!(table, $no_select_column_name -> $no_select_type);)*
            }
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! joinable {
    ($child:ident -> $parent:ident ($source:ident)) => {
        joinable_inner!($child::table => $parent::table : ($child::$source = $parent::table));
        joinable_inner!($parent::table => $child::table : ($child::$source = $parent::table));
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! joinable_inner {
    ($left_table:path => $right_table:path : ($foreign_key:path = $parent_table:path)) => {
        impl<JoinType> $crate::JoinTo<$right_table, JoinType> for $left_table {
            type JoinClause = $crate::query_builder::nodes::Join<
                <$left_table as $crate::QuerySource>::FromClause,
                <$right_table as $crate::QuerySource>::FromClause,
                $crate::expression::helper_types::Eq<
                    $crate::expression::nullable::Nullable<$foreign_key>,
                    $crate::expression::nullable::Nullable<
                        <$parent_table as $crate::query_source::Table>::PrimaryKey>,
                >,
                JoinType,
            >;

            fn join_clause(&self, join_type: JoinType) -> Self::JoinClause {
                use $crate::QuerySource;

                $crate::query_builder::nodes::Join::new(
                    self.from_clause(),
                    $right_table.from_clause(),
                    $foreign_key.nullable().eq($parent_table.primary_key().nullable()),
                    join_type,
                )
            }
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! select_column_workaround {
    ($parent:ident -> $child:ident ($($column_name:ident),+)) => {
        $(select_column_inner!($parent -> $child $column_name);)+
        select_column_inner!($parent -> $child star);
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! select_column_inner {
    ($parent:ident -> $child:ident $column_name:ident) => {
        impl $crate::expression::SelectableExpression<
            $crate::query_source::InnerJoinSource<$child::table, $parent::table>,
        > for $parent::$column_name
        {
        }

        impl $crate::expression::SelectableExpression<
            $crate::query_source::InnerJoinSource<$parent::table, $child::table>,
        > for $parent::$column_name
        {
        }

        impl $crate::expression::SelectableExpression<
            $crate::query_source::LeftOuterJoinSource<$child::table, $parent::table>,
            <<$parent::$column_name as $crate::Expression>::SqlType
                as $crate::types::IntoNullable>::Nullable,
        > for $parent::$column_name
        {
        }

        impl $crate::expression::SelectableExpression<
            $crate::query_source::LeftOuterJoinSource<$parent::table, $child::table>,
        > for $parent::$column_name
        {
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! join_through {
    ($parent:ident -> $through:ident -> $child:ident) => {
        impl<JoinType: Copy> $crate::JoinTo<$child::table, JoinType> for $parent::table {
            type JoinClause = <
                <$parent::table as $crate::JoinTo<$through::table, JoinType>>::JoinClause
                as $crate::query_builder::nodes::CombinedJoin<
                    <$through::table as $crate::JoinTo<$child::table, JoinType>>::JoinClause,
                >>::Output;

            fn join_clause(&self, join_type: JoinType) -> Self::JoinClause {
                use $crate::query_builder::nodes::CombinedJoin;
                let parent_to_through = $crate::JoinTo::<$through::table, JoinType>
                    ::join_clause(&$parent::table, join_type);
                let through_to_child = $crate::JoinTo::<$child::table, JoinType>
                    ::join_clause(&$through::table, join_type);
                parent_to_through.combine_with(through_to_child)
            }
        }
    }
}

/// Takes a query QueryFragment expression as an argument and returns a string
/// of SQL with placeholders for the dynamic values.
///
/// # Example
///
/// ### Returning SQL from a count statment:
/// #
/// # ```rust
/// # // example requires setup for users table
/// # use diesel::users::dsl::*;
/// # use diesel::query_builder::QueryFragment;
/// #
/// # fn main() {
/// let sql = debug_sql!(users.count());
/// assert_eq!(sql, "SELECT COUNT(*) FROM users");
/// # }
/// # ```
#[macro_export]
macro_rules! debug_sql {
    ($query:expr) => {{
        use $crate::query_builder::QueryFragment;
        use $crate::query_builder::debug::DebugQueryBuilder;
        let mut query_builder = DebugQueryBuilder::new();
        QueryFragment::<$crate::backend::Debug>::to_sql(&$query, &mut query_builder).unwrap();
        query_builder.sql
    }};
}

/// Takes takes a query QueryFragment expression as an argument and prints out
/// the SQL with placeholders for the dynamic values.
///
/// # Example
///
/// ### Printing SQL from a count statment:
/// #
/// # ```rust
/// # // example requires setup for users table
/// # use diesel::users::dsl::*;
/// # use diesel::query_builder::QueryFragment;
/// #
/// # fn main() {
/// print_sql!(users.count());
/// # }
/// # ```
#[macro_export]
macro_rules! print_sql {
    ($query:expr) => {
        println!("{}", &debug_sql!($query));
    };
}
