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
      case "onSale": {
        return Boolean(value);
      }
      default: {
        return cast(field, value);
      }
    }
  },
};

export default async function ProductsCreate(_, { input }) {
  const { name, slug, price, onSale } = input;

  // Check for unique constraint and error when supported

  try {
    const { insertId } = await conn.execute(
      "INSERT INTO Products (`name`, `slug`, `price`, `onSale`) VALUES (?, ?, ?, ?)",
      [name, slug, price, onSale],
      options
    );

    return {
      id: insertId,
      name,
      slug,
      price,
      onSale,
    };
  } catch (error) {
    console.log(error);

    return null;
  }
}
