import { connect, cast } from "@planetscale/database";

const config = {
  host: process.env.DATABASE_HOST,
  username: process.env.DATABASE_USERNAME,
  password: process.env.DATABASE_PASSWORD,
};

const conn = connect(config);

export const options = {
  cast(field, value) {
    switch (field.name) {
      case "id": {
        return String(value);
      }
      case "onSale": {
        return Boolean(value);
      }
      default: {
        return cast(field, value);
      }
    }
  },
};

export default async function ProductsDelete(_, args) {
  const {
    by: { id, slug },
  } = args;

  let statement;
  let params;

  if (id !== undefined) {
    statement = "DELETE FROM Products WHERE id = ?";
    params = [id];
  } else if (slug !== undefined) {
    statement = "DELETE FROM Products WHERE slug = ?";
    params = [slug];
  } else {
    // Throw new GraphQLError('ID or Slug must be provided')
  }

  try {
    const results = await conn.execute(statement, params, options);

    console.log(JSON.stringify(results, null, 2));

    if (results.rowsAffected === 1) {
      return { deleted: true };
    }

    return { deleted: false };
  } catch (error) {
    console.log(error);

    return { deleted: false };
  }
}
