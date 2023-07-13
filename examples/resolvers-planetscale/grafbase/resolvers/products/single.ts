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

export default async function ProductsSingle(_, args) {
  const {
    by: { id, slug },
  } = args;

  try {
    if (id !== undefined) {
      const results = await conn.execute(
        "SELECT * FROM Products WHERE id = ? LIMIT 1",
        [id],
        options
      );

      console.log(JSON.stringify(results, null, 2));

      return results?.rows[0] ?? null;
    }

    if (slug !== undefined) {
      const results = await conn.execute(
        "SELECT * FROM Products WHERE slug = ? LIMIT 1",
        [slug],
        options
      );

      return results?.rows[0] ?? null;
    }

    // Throw new GraphQLError('ID or Slug must be provided')
  } catch (error) {
    console.log(error);

    return null;
  }
}
