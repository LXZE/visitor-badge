use diesel::prelude::*;

use crate::models;

type DbError = Box<dyn std::error::Error + Send + Sync>;

/// Run query using Diesel to find user by uid and return it.
pub fn get_user_viewcount(
    conn: &mut SqliteConnection,
    user: &String,
) -> Result<Option<models::Visitors>, DbError> {
    use crate::schema::visitors::dsl::*;

    let user = visitors
        .filter(id.eq(user))
        .first::<models::Visitors>(conn)
        .optional()?;
    Ok(user)
}

pub fn update_and_get_user_viewcount(
    conn: &mut SqliteConnection,
    user: &String,
) -> Result<usize, DbError> {
    use crate::schema::visitors::dsl::*;

	let updated_row = diesel::update(visitors.filter(id.eq(user)))
		.set(view_count.eq(view_count + 1))
		.execute(conn)?;
	Ok(updated_row)
}
