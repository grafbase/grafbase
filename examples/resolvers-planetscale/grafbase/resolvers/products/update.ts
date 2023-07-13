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

export default async function ProductsUpdate(_, args) {
  const {
    by: { id, slug },
    input: { name: newName, slug: newSlug, price: newPrice, onSale: newOnSale },
  } = args;

  let updateStatement = "UPDATE Products SET ";
  let params = [];

  if (newName !== undefined) {
    updateStatement += "name = ?, ";
    params.push(newName);
  }
  if (newSlug !== undefined) {
    updateStatement += "slug = ?, ";
    params.push(newSlug);
  }
  if (newPrice !== undefined) {
    updateStatement += "price = ?, ";
    params.push(newPrice);
  }
  if (newOnSale !== undefined) {
    updateStatement += "onSale = ?, ";
    params.push(newOnSale);
  }

  if (params.length === 0) {
    throw new Error("At least one field to update must be provided.");
  }

  updateStatement = updateStatement.slice(0, -2);

  if (id !== undefined) {
    updateStatement += " WHERE id = ?";
    params.push(id);
  } else if (slug !== undefined) {
    updateStatement += " WHERE slug = ?";
    params.push(slug);
  } else {
    // Throw new GraphQLError('ID or Slug must be provided')
  }

  let selectStatement;
  let selectParams;

  if (id !== undefined) {
    selectParams = [id];
    selectStatement = "SELECT * FROM Products WHERE id = ?";
  } else {
    selectParams = [slug];
    selectStatement = "SELECT * FROM Products WHERE slug = ?";
  }

  try {
    const [_, results] = await conn.transaction(async (tx) => {
      const update = await tx.execute(updateStatement, params, options);
      const select = await tx.execute(selectStatement, selectParams, options);

      return [update, select];
    });

    return results?.rows[0] ?? null;
  } catch (error) {
    console.log(error);

    return null;
  }
}
