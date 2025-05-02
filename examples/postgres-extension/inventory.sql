CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE inventory (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
  product_id UUID,
  variant_id UUID,
  sku VARCHAR(50) NOT NULL,
  quantity INTEGER NOT NULL DEFAULT 0,
  warehouse_location VARCHAR(100),
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CHECK (
    product_id IS NOT NULL
    OR variant_id IS NOT NULL
  )
);
