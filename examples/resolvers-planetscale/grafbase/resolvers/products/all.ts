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

export default async function ProductsAll(_, args) {
  const { first, last, before, after } = args;

  // Throw GraphQLError when arg is missing when supported

  try {
    if (first !== undefined && after !== undefined) {
      const results = await conn.execute(
        "SELECT * FROM Products WHERE id > ? ORDER BY id ASC LIMIT ?",
        [after, first],
        options
      );

      return results?.rows || [];
    }

    if (last !== undefined && before !== undefined) {
      const results = await conn.execute(
        `SELECT * FROM (
          SELECT * FROM Products WHERE id < ? ORDER BY id DESC LIMIT ?
        ) AS sub ORDER BY id ASC`,
        [before, last],
        options
      );

      return results?.rows || [];
    }

    if (first !== undefined) {
      const results = await conn.execute(
        "SELECT * FROM Products ORDER BY id ASC LIMIT ?",
        [first],
        options
      );

      return results?.rows || [];
    }

    if (last !== undefined) {
      const results = await conn.execute(
        `SELECT * FROM (
          SELECT * FROM Products ORDER BY id DESC LIMIT ?
        ) AS sub ORDER BY id ASC`,
        [last],
        options
      );

      return results?.rows || [];
    }
  } catch (error) {
    console.log(error);

    return [];
  }
}
