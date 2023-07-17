```shell
CREATE TABLE `products` (
    `id` INT NOT NULL AUTO_INCREMENT,
    `name` VARCHAR(255) NOT NULL,
    `slug` VARCHAR(255) NOT NULL UNIQUE,
    `price` INT NOT NULL DEFAULT 0,
    `onSale` BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (id),
    INDEX slug_index (slug),
);
```

```graphql
mutation {
  productCreate(input: { name: "Shoes", price: 1000, onSale: true }) {
    id
    name
    onSale
    price
  }
}
```

```graphql
{
  products(first: 100) {
    id
    name
    slug
    price
    onSale
  }
}
```

```graphql
{
  products(last: 100) {
    id
    name
    slug
    price
    onSale
  }
}
```

```graphql
{
  products(first: 100, after: "1") {
    id
    name
    slug
    price
    onSale
  }
}
```

```graphql
{
  products(last: 100, before: "7") {
    id
    name
    slug
    price
    onSale
  }
}
```

```graphql
{
  product(by: { id: "1" }) {
    id
    name
    slug
    onSale
    price
  }
}
```

```graphql
mutation {
  productUpdate(by: { id: "7" }, input: { name: "New shoes" }) {
    id
    name
    slug
    price
    onSale
  }
}
```

```graphql
mutation {
  productDelete(by: { id: "7" }) {
    deleted
  }
}
```
