CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE inventory (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  sku VARCHAR(50) NOT NULL UNIQUE,
  quantity INTEGER NOT NULL DEFAULT 0,
  warehouse_location VARCHAR(100),
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Inventory data based on variant SKUs from products.sql

-- Product 1 (T-Shirt) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('TSHIRT-001-BLK-S', 5, 'WAREHOUSE-A01'),
  ('TSHIRT-001-BLK-M', 8, 'WAREHOUSE-A01'),
  ('TSHIRT-001-BLK-L', 6, 'WAREHOUSE-A01'),
  ('TSHIRT-001-WHT-S', 4, 'WAREHOUSE-A01'),
  ('TSHIRT-001-WHT-M', 7, 'WAREHOUSE-A01'),
  ('TSHIRT-001-WHT-L', 5, 'WAREHOUSE-A01'),
  ('TSHIRT-001-RED-M', 3, 'WAREHOUSE-A01');

-- Product 2 (Jeans) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('JEANS-001-BLU-30', 3, 'WAREHOUSE-B02'),
  ('JEANS-001-BLU-32', 5, 'WAREHOUSE-B02'),
  ('JEANS-001-BLU-34', 4, 'WAREHOUSE-B02'),
  ('JEANS-001-BLK-30', 2, 'WAREHOUSE-B02'),
  ('JEANS-001-BLK-32', 6, 'WAREHOUSE-B02'),
  ('JEANS-001-BLK-34', 3, 'WAREHOUSE-B02'),
  ('JEANS-001-GRY-32', 2, 'WAREHOUSE-B02');

-- Product 3 (Sneakers) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('SNEAKER-001-WHT-9', 4, 'WAREHOUSE-C03'),
  ('SNEAKER-001-WHT-10', 5, 'WAREHOUSE-C03'),
  ('SNEAKER-001-BLK-9', 3, 'WAREHOUSE-C03'),
  ('SNEAKER-001-BLK-10', 6, 'WAREHOUSE-C03'),
  ('SNEAKER-001-RED-9', 2, 'WAREHOUSE-C03'),
  ('SNEAKER-001-RED-10', 3, 'WAREHOUSE-C03');

-- Product 4 (Watch) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('WATCH-001-BRN', 5, 'WAREHOUSE-D04'),
  ('WATCH-001-BLK', 7, 'WAREHOUSE-D04'),
  ('WATCH-001-SLV', 4, 'WAREHOUSE-D04'),
  ('WATCH-001-GLD', 2, 'WAREHOUSE-D04'),
  ('WATCH-001-BLU', 3, 'WAREHOUSE-D04'),
  ('WATCH-001-SPORT', 4, 'WAREHOUSE-D04');

-- Product 5 (Laptop) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('LAPTOP-001-8GB', 2, 'WAREHOUSE-E05'),
  ('LAPTOP-001-16GB', 3, 'WAREHOUSE-E05'),
  ('LAPTOP-001-32GB', 1, 'WAREHOUSE-E05'),
  ('LAPTOP-001-16GB-i7', 2, 'WAREHOUSE-E05'),
  ('LAPTOP-001-16GB-AMD', 2, 'WAREHOUSE-E05');

-- Product 6 (Smartphone) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('PHONE-001-64GB', 8, 'WAREHOUSE-E06'),
  ('PHONE-001-128GB', 10, 'WAREHOUSE-E06'),
  ('PHONE-001-256GB', 5, 'WAREHOUSE-E06'),
  ('PHONE-001-64GB-RED', 4, 'WAREHOUSE-E06'),
  ('PHONE-001-128GB-RED', 6, 'WAREHOUSE-E06'),
  ('PHONE-001-128GB-GLD', 3, 'WAREHOUSE-E06');

-- Product 7 (Headphones) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('HEADPHONE-001-BLK', 7, 'WAREHOUSE-D07'),
  ('HEADPHONE-001-WHT', 5, 'WAREHOUSE-D07'),
  ('HEADPHONE-001-RED', 3, 'WAREHOUSE-D07'),
  ('HEADPHONE-001-BLU', 4, 'WAREHOUSE-D07'),
  ('HEADPHONE-001-PRO', 2, 'WAREHOUSE-D07');

-- Product 8 (Water Bottle) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('BOTTLE-001-BLU', 10, 'WAREHOUSE-A12'),
  ('BOTTLE-001-BLK', 12, 'WAREHOUSE-A12'),
  ('BOTTLE-001-SLV', 8, 'WAREHOUSE-A12'),
  ('BOTTLE-001-BLU-32', 6, 'WAREHOUSE-A12'),
  ('BOTTLE-001-BLK-32', 7, 'WAREHOUSE-A12');

-- Product 9 (Backpack) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('BACKPACK-001-BLK', 6, 'WAREHOUSE-B09'),
  ('BACKPACK-001-NVY', 5, 'WAREHOUSE-B09'),
  ('BACKPACK-001-GRY', 4, 'WAREHOUSE-B09'),
  ('BACKPACK-001-RED', 3, 'WAREHOUSE-B09'),
  ('BACKPACK-001-PRO', 2, 'WAREHOUSE-B09'),
  ('BACKPACK-001-TRAVEL', 3, 'WAREHOUSE-B09');

-- Product 10 (Ergonomic Office Chair) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('CHAIR-001-BLK', 4, 'WAREHOUSE-B05'),
  ('CHAIR-001-GRY', 3, 'WAREHOUSE-B05'),
  ('CHAIR-001-WHT', 2, 'WAREHOUSE-B05'),
  ('CHAIR-001-MESH', 3, 'WAREHOUSE-B05'),
  ('CHAIR-001-EXEC', 1, 'WAREHOUSE-B05');

-- Product 11 (Coffee Maker) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('COFFEE-001-10C', 5, 'WAREHOUSE-C11'),
  ('COFFEE-001-12C', 4, 'WAREHOUSE-C11'),
  ('COFFEE-001-SS', 3, 'WAREHOUSE-C11'),
  ('COFFEE-001-SING', 6, 'WAREHOUSE-C11'),
  ('COFFEE-001-COLD', 2, 'WAREHOUSE-C11');

-- Product 12 (Indoor Plant Kit) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('PLANT-001-SUCC', 8, 'WAREHOUSE-C09'),
  ('PLANT-001-FERN', 6, 'WAREHOUSE-C09'),
  ('PLANT-001-HERB', 7, 'WAREHOUSE-C09'),
  ('PLANT-001-AIR', 5, 'WAREHOUSE-C09'),
  ('PLANT-001-MINI', 9, 'WAREHOUSE-C09');

-- Product 13 (Smart LED Desk Lamp) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('LAMP-001-BLK', 6, 'WAREHOUSE-A23'),
  ('LAMP-001-WHT', 5, 'WAREHOUSE-A23'),
  ('LAMP-001-SLV', 4, 'WAREHOUSE-A23'),
  ('LAMP-001-WLESS', 3, 'WAREHOUSE-A23'),
  ('LAMP-001-PRO', 2, 'WAREHOUSE-A23');

-- Product 14 (Portable Bluetooth Speaker) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('SPEAKER-001-BLK', 9, 'WAREHOUSE-D11'),
  ('SPEAKER-001-WHT', 7, 'WAREHOUSE-D11'),
  ('SPEAKER-001-BLU', 6, 'WAREHOUSE-D11'),
  ('SPEAKER-001-RED', 5, 'WAREHOUSE-D11'),
  ('SPEAKER-001-MINI', 10, 'WAREHOUSE-D11'),
  ('SPEAKER-001-WATER', 4, 'WAREHOUSE-D11');

-- Product 15 (Luxury Bath Towel Set) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('TOWEL-001-WHT', 15, 'WAREHOUSE-A15'),
  ('TOWEL-001-GRY', 12, 'WAREHOUSE-A15'),
  ('TOWEL-001-BLK', 10, 'WAREHOUSE-A15'),
  ('TOWEL-001-NVY', 8, 'WAREHOUSE-A15'),
  ('TOWEL-001-LUX', 5, 'WAREHOUSE-A15');

-- Product 16 (Professional Chef Knife) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('KNIFE-001-8IN', 7, 'WAREHOUSE-B14'),
  ('KNIFE-001-6IN', 6, 'WAREHOUSE-B14'),
  ('KNIFE-001-10IN', 4, 'WAREHOUSE-B14'),
  ('KNIFE-001-SET3', 3, 'WAREHOUSE-B14'),
  ('KNIFE-001-DAM', 2, 'WAREHOUSE-B14');

-- Product 17 (Graphics Tablet Pro) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('TABLET-001-10', 5, 'WAREHOUSE-C17'),
  ('TABLET-001-12', 4, 'WAREHOUSE-C17'),
  ('TABLET-001-16', 2, 'WAREHOUSE-C17'),
  ('TABLET-001-10-BUN', 3, 'WAREHOUSE-C17'),
  ('TABLET-001-12-BUN', 2, 'WAREHOUSE-C17');

-- Product 18 (Digital SLR Camera) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('CAMERA-001-BDY', 3, 'WAREHOUSE-E18'),
  ('CAMERA-001-KIT', 4, 'WAREHOUSE-E18'),
  ('CAMERA-001-PRO', 1, 'WAREHOUSE-E18'),
  ('CAMERA-001-PRO-KIT', 2, 'WAREHOUSE-E18'),
  ('CAMERA-001-BUN', 3, 'WAREHOUSE-E18');

-- Product 19 (27-inch Monitor) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('MONITOR-001-27-HD', 6, 'WAREHOUSE-D08'),
  ('MONITOR-001-27-4K', 4, 'WAREHOUSE-D08'),
  ('MONITOR-001-32-4K', 3, 'WAREHOUSE-D08'),
  ('MONITOR-001-27-CURVE', 3, 'WAREHOUSE-D08'),
  ('MONITOR-001-34-WIDE', 2, 'WAREHOUSE-D08');

-- Product 20 (Memory Foam Pillow) variants
INSERT INTO
  inventory (sku, quantity, warehouse_location)
VALUES
  ('PILLOW-001-STD', 10, 'WAREHOUSE-A19'),
  ('PILLOW-001-KING', 8, 'WAREHOUSE-A19'),
  ('PILLOW-001-COOL', 7, 'WAREHOUSE-A19'),
  ('PILLOW-001-CONTOUR', 9, 'WAREHOUSE-A19'),
  ('PILLOW-001-TRAVEL', 12, 'WAREHOUSE-A19');

CREATE INDEX idx_inventory_sku ON inventory(sku);