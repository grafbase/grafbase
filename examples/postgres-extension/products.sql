CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE products (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
  sku VARCHAR(50) NOT NULL UNIQUE,
  name VARCHAR(255) NOT NULL,
  slug VARCHAR(255) NOT NULL UNIQUE,
  description TEXT,
  price DECIMAL(10, 2) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE variants (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
  product_id UUID NOT NULL,
  sku VARCHAR(50) NOT NULL UNIQUE,
  name VARCHAR(255),
  price MONEY,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT variants_to_products FOREIGN KEY (product_id) REFERENCES products (id) ON DELETE CASCADE
);

-- Insert Products
INSERT INTO
  products (id, sku, name, slug, description, price)
VALUES
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d401',
    'TSHIRT-001',
    'Classic Cotton T-Shirt',
    'classic-cotton-tshirt',
    'A comfortable, everyday cotton t-shirt suitable for all occasions.',
    24.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d402',
    'JEANS-001',
    'Slim Fit Jeans',
    'slim-fit-jeans',
    'Modern slim fit jeans with stretch technology for maximum comfort.',
    59.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d403',
    'SNEAKER-001',
    'Urban Sneakers',
    'urban-sneakers',
    'Stylish urban sneakers perfect for casual everyday wear.',
    89.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d404',
    'WATCH-001',
    'Minimalist Watch',
    'minimalist-watch',
    'Clean design analog watch with leather strap.',
    129.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d405',
    'LAPTOP-001',
    'ProBook X5',
    'probook-x5',
    '15-inch laptop with 16GB RAM and 512GB SSD.',
    1299.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d406',
    'PHONE-001',
    'SmartPhone Z10',
    'smartphone-z10',
    'Latest generation smartphone with 128GB storage and 6.5-inch display.',
    899.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d407',
    'HEADPHONE-001',
    'Wireless Headphones',
    'wireless-headphones',
    'Noise-cancelling wireless headphones with 20-hour battery life.',
    149.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d408',
    'BOTTLE-001',
    'Insulated Water Bottle',
    'insulated-water-bottle',
    'Stainless steel insulated bottle that keeps drinks cold for 24 hours.',
    29.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d409',
    'BACKPACK-001',
    'Urban Backpack',
    'urban-backpack',
    'Sleek backpack with laptop compartment and multiple pockets.',
    79.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d410',
    'CHAIR-001',
    'Ergonomic Office Chair',
    'ergonomic-office-chair',
    'Fully adjustable ergonomic chair with lumbar support.',
    249.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d411',
    'COFFEE-001',
    'Premium Coffee Maker',
    'premium-coffee-maker',
    'Programmable coffee maker with thermal carafe.',
    119.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d412',
    'PLANT-001',
    'Indoor Plant Kit',
    'indoor-plant-kit',
    'Set of 3 low-maintenance indoor plants with decorative pots.',
    49.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d413',
    'LAMP-001',
    'Smart LED Desk Lamp',
    'smart-led-desk-lamp',
    'Adjustable desk lamp with multiple brightness settings and color modes.',
    69.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d414',
    'SPEAKER-001',
    'Portable Bluetooth Speaker',
    'portable-bluetooth-speaker',
    'Compact speaker with powerful sound and 10-hour playback.',
    59.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d415',
    'TOWEL-001',
    'Luxury Bath Towel Set',
    'luxury-bath-towel-set',
    'Set of 4 premium cotton bath towels.',
    39.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d416',
    'KNIFE-001',
    'Professional Chef Knife',
    'professional-chef-knife',
    'High carbon stainless steel 8-inch chef knife.',
    79.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d417',
    'TABLET-001',
    'Graphics Tablet Pro',
    'graphics-tablet-pro',
    '10-inch drawing tablet with pressure sensitivity.',
    199.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d418',
    'CAMERA-001',
    'Digital SLR Camera',
    'digital-slr-camera',
    'Entry-level DSLR with 24MP sensor and 18-55mm lens.',
    649.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d419',
    'MONITOR-001',
    '27-inch Monitor',
    '27-inch-monitor',
    'Ultra HD IPS monitor with adjustable stand.',
    349.99
  ),
  (
    'f47ac10b-58cc-4372-a567-0e02b2c3d420',
    'PILLOW-001',
    'Memory Foam Pillow',
    'memory-foam-pillow',
    'Ergonomic pillow that adjusts to your sleeping position.',
    49.99
  );

-- Product 1 (T-Shirt) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a01',
    'f47ac10b-58cc-4372-a567-0e02b2c3d401',
    'TSHIRT-001-BLK-S',
    'Black Small',
    24.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a02',
    'f47ac10b-58cc-4372-a567-0e02b2c3d401',
    'TSHIRT-001-BLK-M',
    'Black Medium',
    24.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a03',
    'f47ac10b-58cc-4372-a567-0e02b2c3d401',
    'TSHIRT-001-BLK-L',
    'Black Large',
    24.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a27',
    'f47ac10b-58cc-4372-a567-0e02b2c3d401',
    'TSHIRT-001-WHT-S',
    'White Small',
    24.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a28',
    'f47ac10b-58cc-4372-a567-0e02b2c3d401',
    'TSHIRT-001-WHT-M',
    'White Medium',
    24.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a29',
    'f47ac10b-58cc-4372-a567-0e02b2c3d401',
    'TSHIRT-001-WHT-L',
    'White Large',
    24.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a30',
    'f47ac10b-58cc-4372-a567-0e02b2c3d401',
    'TSHIRT-001-RED-M',
    'Red Medium',
    26.99
  );

-- Product 2 (Jeans) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a04',
    'f47ac10b-58cc-4372-a567-0e02b2c3d402',
    'JEANS-001-BLU-30',
    'Blue W30',
    59.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a05',
    'f47ac10b-58cc-4372-a567-0e02b2c3d402',
    'JEANS-001-BLU-32',
    'Blue W32',
    59.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a06',
    'f47ac10b-58cc-4372-a567-0e02b2c3d402',
    'JEANS-001-BLU-34',
    'Blue W34',
    59.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a31',
    'f47ac10b-58cc-4372-a567-0e02b2c3d402',
    'JEANS-001-BLK-30',
    'Black W30',
    59.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a32',
    'f47ac10b-58cc-4372-a567-0e02b2c3d402',
    'JEANS-001-BLK-32',
    'Black W32',
    59.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a33',
    'f47ac10b-58cc-4372-a567-0e02b2c3d402',
    'JEANS-001-BLK-34',
    'Black W34',
    59.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a34',
    'f47ac10b-58cc-4372-a567-0e02b2c3d402',
    'JEANS-001-GRY-32',
    'Grey W32',
    64.99
  );

-- Product 3 (Sneakers) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a07',
    'f47ac10b-58cc-4372-a567-0e02b2c3d403',
    'SNEAKER-001-WHT-9',
    'White Size 9',
    89.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a08',
    'f47ac10b-58cc-4372-a567-0e02b2c3d403',
    'SNEAKER-001-WHT-10',
    'White Size 10',
    89.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a09',
    'f47ac10b-58cc-4372-a567-0e02b2c3d403',
    'SNEAKER-001-BLK-9',
    'Black Size 9',
    89.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a35',
    'f47ac10b-58cc-4372-a567-0e02b2c3d403',
    'SNEAKER-001-BLK-10',
    'Black Size 10',
    89.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a36',
    'f47ac10b-58cc-4372-a567-0e02b2c3d403',
    'SNEAKER-001-RED-9',
    'Red Size 9',
    94.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a37',
    'f47ac10b-58cc-4372-a567-0e02b2c3d403',
    'SNEAKER-001-RED-10',
    'Red Size 10',
    94.99
  );

-- Product 4 (Watch) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a10',
    'f47ac10b-58cc-4372-a567-0e02b2c3d404',
    'WATCH-001-BRN',
    'Brown Leather',
    129.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
    'f47ac10b-58cc-4372-a567-0e02b2c3d404',
    'WATCH-001-BLK',
    'Black Leather',
    129.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a38',
    'f47ac10b-58cc-4372-a567-0e02b2c3d404',
    'WATCH-001-SLV',
    'Silver Metal',
    149.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a39',
    'f47ac10b-58cc-4372-a567-0e02b2c3d404',
    'WATCH-001-GLD',
    'Gold Metal',
    169.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a40',
    'f47ac10b-58cc-4372-a567-0e02b2c3d404',
    'WATCH-001-BLU',
    'Blue Leather',
    129.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a41',
    'f47ac10b-58cc-4372-a567-0e02b2c3d404',
    'WATCH-001-SPORT',
    'Sport Band',
    119.99
  );

-- Product 5 (Laptop) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a12',
    'f47ac10b-58cc-4372-a567-0e02b2c3d405',
    'LAPTOP-001-8GB',
    '8GB RAM / 256GB SSD',
    1099.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a13',
    'f47ac10b-58cc-4372-a567-0e02b2c3d405',
    'LAPTOP-001-16GB',
    '16GB RAM / 512GB SSD',
    1299.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a42',
    'f47ac10b-58cc-4372-a567-0e02b2c3d405',
    'LAPTOP-001-32GB',
    '32GB RAM / 1TB SSD',
    1599.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a43',
    'f47ac10b-58cc-4372-a567-0e02b2c3d405',
    'LAPTOP-001-16GB-i7',
    '16GB RAM / 512GB SSD / i7 CPU',
    1499.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a44',
    'f47ac10b-58cc-4372-a567-0e02b2c3d405',
    'LAPTOP-001-16GB-AMD',
    '16GB RAM / 512GB SSD / AMD CPU',
    1349.99
  );

-- Product 6 (Smartphone) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a14',
    'f47ac10b-58cc-4372-a567-0e02b2c3d406',
    'PHONE-001-64GB',
    '64GB Storage',
    799.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a15',
    'f47ac10b-58cc-4372-a567-0e02b2c3d406',
    'PHONE-001-128GB',
    '128GB Storage',
    899.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a16',
    'f47ac10b-58cc-4372-a567-0e02b2c3d406',
    'PHONE-001-256GB',
    '256GB Storage',
    999.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a45',
    'f47ac10b-58cc-4372-a567-0e02b2c3d406',
    'PHONE-001-64GB-RED',
    '64GB Storage - Red',
    819.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a46',
    'f47ac10b-58cc-4372-a567-0e02b2c3d406',
    'PHONE-001-128GB-RED',
    '128GB Storage - Red',
    919.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a47',
    'f47ac10b-58cc-4372-a567-0e02b2c3d406',
    'PHONE-001-128GB-GLD',
    '128GB Storage - Gold',
    949.99
  );

-- Product 7 (Headphones) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a17',
    'f47ac10b-58cc-4372-a567-0e02b2c3d407',
    'HEADPHONE-001-BLK',
    'Black',
    149.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a18',
    'f47ac10b-58cc-4372-a567-0e02b2c3d407',
    'HEADPHONE-001-WHT',
    'White',
    149.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a48',
    'f47ac10b-58cc-4372-a567-0e02b2c3d407',
    'HEADPHONE-001-RED',
    'Red',
    149.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a49',
    'f47ac10b-58cc-4372-a567-0e02b2c3d407',
    'HEADPHONE-001-BLU',
    'Blue',
    149.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a50',
    'f47ac10b-58cc-4372-a567-0e02b2c3d407',
    'HEADPHONE-001-PRO',
    'Pro Edition - Black',
    199.99
  );

-- Product 8 (Water Bottle) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a51',
    'f47ac10b-58cc-4372-a567-0e02b2c3d408',
    'BOTTLE-001-BLU',
    'Blue - 20oz',
    29.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a52',
    'f47ac10b-58cc-4372-a567-0e02b2c3d408',
    'BOTTLE-001-BLK',
    'Black - 20oz',
    29.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a53',
    'f47ac10b-58cc-4372-a567-0e02b2c3d408',
    'BOTTLE-001-SLV',
    'Silver - 20oz',
    29.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a54',
    'f47ac10b-58cc-4372-a567-0e02b2c3d408',
    'BOTTLE-001-BLU-32',
    'Blue - 32oz',
    34.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a55',
    'f47ac10b-58cc-4372-a567-0e02b2c3d408',
    'BOTTLE-001-BLK-32',
    'Black - 32oz',
    34.99
  );

-- Product 9 (Backpack) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a19',
    'f47ac10b-58cc-4372-a567-0e02b2c3d409',
    'BACKPACK-001-BLK',
    'Black',
    79.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a20',
    'f47ac10b-58cc-4372-a567-0e02b2c3d409',
    'BACKPACK-001-NVY',
    'Navy',
    79.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a56',
    'f47ac10b-58cc-4372-a567-0e02b2c3d409',
    'BACKPACK-001-GRY',
    'Grey',
    79.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a57',
    'f47ac10b-58cc-4372-a567-0e02b2c3d409',
    'BACKPACK-001-RED',
    'Red',
    79.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a58',
    'f47ac10b-58cc-4372-a567-0e02b2c3d409',
    'BACKPACK-001-PRO',
    'Pro - Black',
    99.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a59',
    'f47ac10b-58cc-4372-a567-0e02b2c3d409',
    'BACKPACK-001-TRAVEL',
    'Travel Edition',
    109.99
  );

-- Product 10 (Ergonomic Chair) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a60',
    'f47ac10b-58cc-4372-a567-0e02b2c3d410',
    'CHAIR-001-BLK',
    'Black',
    249.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a61',
    'f47ac10b-58cc-4372-a567-0e02b2c3d410',
    'CHAIR-001-GRY',
    'Grey',
    249.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a62',
    'f47ac10b-58cc-4372-a567-0e02b2c3d410',
    'CHAIR-001-WHT',
    'White',
    249.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a63',
    'f47ac10b-58cc-4372-a567-0e02b2c3d410',
    'CHAIR-001-MESH',
    'Mesh Back - Black',
    279.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a64',
    'f47ac10b-58cc-4372-a567-0e02b2c3d410',
    'CHAIR-001-EXEC',
    'Executive Model',
    349.99
  );

-- Product 11 (Coffee Maker) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a21',
    'f47ac10b-58cc-4372-a567-0e02b2c3d411',
    'COFFEE-001-10C',
    '10-Cup',
    119.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22',
    'f47ac10b-58cc-4372-a567-0e02b2c3d411',
    'COFFEE-001-12C',
    '12-Cup',
    129.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a65',
    'f47ac10b-58cc-4372-a567-0e02b2c3d411',
    'COFFEE-001-SS',
    'Stainless Steel - 10-Cup',
    139.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a66',
    'f47ac10b-58cc-4372-a567-0e02b2c3d411',
    'COFFEE-001-SING',
    'Single Serve',
    99.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a67',
    'f47ac10b-58cc-4372-a567-0e02b2c3d411',
    'COFFEE-001-COLD',
    'Cold Brew',
    149.99
  );

-- Product 12 (Indoor Plant Kit) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a68',
    'f47ac10b-58cc-4372-a567-0e02b2c3d412',
    'PLANT-001-SUCC',
    'Succulent Set',
    49.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a69',
    'f47ac10b-58cc-4372-a567-0e02b2c3d412',
    'PLANT-001-FERN',
    'Fern Collection',
    49.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a70',
    'f47ac10b-58cc-4372-a567-0e02b2c3d412',
    'PLANT-001-HERB',
    'Herb Garden',
    59.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a71',
    'f47ac10b-58cc-4372-a567-0e02b2c3d412',
    'PLANT-001-AIR',
    'Air Plant Collection',
    44.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a72',
    'f47ac10b-58cc-4372-a567-0e02b2c3d412',
    'PLANT-001-MINI',
    'Mini Plant Set',
    39.99
  );

-- Product 13 (Smart LED Desk Lamp) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a73',
    'f47ac10b-58cc-4372-a567-0e02b2c3d413',
    'LAMP-001-BLK',
    'Black',
    69.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a74',
    'f47ac10b-58cc-4372-a567-0e02b2c3d413',
    'LAMP-001-WHT',
    'White',
    69.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a75',
    'f47ac10b-58cc-4372-a567-0e02b2c3d413',
    'LAMP-001-SLV',
    'Silver',
    69.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a76',
    'f47ac10b-58cc-4372-a567-0e02b2c3d413',
    'LAMP-001-WLESS',
    'Wireless Charging Model - Black',
    89.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a77',
    'f47ac10b-58cc-4372-a567-0e02b2c3d413',
    'LAMP-001-PRO',
    'Pro Model with USB Ports',
    99.99
  );

-- Product 14 (Portable Bluetooth Speaker) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a78',
    'f47ac10b-58cc-4372-a567-0e02b2c3d414',
    'SPEAKER-001-BLK',
    'Black',
    59.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a79',
    'f47ac10b-58cc-4372-a567-0e02b2c3d414',
    'SPEAKER-001-WHT',
    'White',
    59.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a80',
    'f47ac10b-58cc-4372-a567-0e02b2c3d414',
    'SPEAKER-001-BLU',
    'Blue',
    59.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a81',
    'f47ac10b-58cc-4372-a567-0e02b2c3d414',
    'SPEAKER-001-RED',
    'Red',
    59.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a82',
    'f47ac10b-58cc-4372-a567-0e02b2c3d414',
    'SPEAKER-001-MINI',
    'Mini - Black',
    39.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a83',
    'f47ac10b-58cc-4372-a567-0e02b2c3d414',
    'SPEAKER-001-WATER',
    'Waterproof - Black',
    79.99
  );

-- Product 15 (Luxury Bath Towel Set) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a84',
    'f47ac10b-58cc-4372-a567-0e02b2c3d415',
    'TOWEL-001-WHT',
    'White',
    39.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a85',
    'f47ac10b-58cc-4372-a567-0e02b2c3d415',
    'TOWEL-001-GRY',
    'Grey',
    39.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a86',
    'f47ac10b-58cc-4372-a567-0e02b2c3d415',
    'TOWEL-001-BLK',
    'Black',
    39.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a87',
    'f47ac10b-58cc-4372-a567-0e02b2c3d415',
    'TOWEL-001-NVY',
    'Navy',
    39.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a88',
    'f47ac10b-58cc-4372-a567-0e02b2c3d415',
    'TOWEL-001-LUX',
    'Luxury Edition',
    59.99
  );

-- Product 16 (Professional Chef Knife) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a89',
    'f47ac10b-58cc-4372-a567-0e02b2c3d416',
    'KNIFE-001-8IN',
    '8-inch',
    79.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a90',
    'f47ac10b-58cc-4372-a567-0e02b2c3d416',
    'KNIFE-001-6IN',
    '6-inch',
    69.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a91',
    'f47ac10b-58cc-4372-a567-0e02b2c3d416',
    'KNIFE-001-10IN',
    '10-inch',
    89.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a92',
    'f47ac10b-58cc-4372-a567-0e02b2c3d416',
    'KNIFE-001-SET3',
    '3-Piece Set',
    199.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a93',
    'f47ac10b-58cc-4372-a567-0e02b2c3d416',
    'KNIFE-001-DAM',
    'Damascus Steel - 8-inch',
    149.99
  );

-- Product 17 (Graphics Tablet Pro) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a94',
    'f47ac10b-58cc-4372-a567-0e02b2c3d417',
    'TABLET-001-10',
    '10-inch Standard',
    199.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a95',
    'f47ac10b-58cc-4372-a567-0e02b2c3d417',
    'TABLET-001-12',
    '12-inch Standard',
    249.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a96',
    'f47ac10b-58cc-4372-a567-0e02b2c3d417',
    'TABLET-001-16',
    '16-inch Pro',
    349.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a97',
    'f47ac10b-58cc-4372-a567-0e02b2c3d417',
    'TABLET-001-10-BUN',
    '10-inch with Software Bundle',
    249.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a98',
    'f47ac10b-58cc-4372-a567-0e02b2c3d417',
    'TABLET-001-12-BUN',
    '12-inch with Software Bundle',
    299.99
  );

-- Product 18 (Digital SLR Camera) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a99',
    'f47ac10b-58cc-4372-a567-0e02b2c3d418',
    'CAMERA-001-BDY',
    'Body Only',
    499.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380b01',
    'f47ac10b-58cc-4372-a567-0e02b2c3d418',
    'CAMERA-001-KIT',
    'With Lens Kit',
    649.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380b02',
    'f47ac10b-58cc-4372-a567-0e02b2c3d418',
    'CAMERA-001-PRO',
    'Pro Body Only',
    799.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380b03',
    'f47ac10b-58cc-4372-a567-0e02b2c3d418',
    'CAMERA-001-PRO-KIT',
    'Pro with Lens Kit',
    999.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380b04',
    'f47ac10b-58cc-4372-a567-0e02b2c3d418',
    'CAMERA-001-BUN',
    'Standard Kit with Accessories',
    749.99
  );

-- Product 19 (27-inch Monitor) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380b05',
    'f47ac10b-58cc-4372-a567-0e02b2c3d419',
    'MONITOR-001-27-HD',
    '27-inch 1080p',
    249.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380b06',
    'f47ac10b-58cc-4372-a567-0e02b2c3d419',
    'MONITOR-001-27-4K',
    '27-inch 4K',
    349.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380b07',
    'f47ac10b-58cc-4372-a567-0e02b2c3d419',
    'MONITOR-001-32-4K',
    '32-inch 4K',
    449.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380b08',
    'f47ac10b-58cc-4372-a567-0e02b2c3d419',
    'MONITOR-001-27-CURVE',
    '27-inch Curved',
    399.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380b09',
    'f47ac10b-58cc-4372-a567-0e02b2c3d419',
    'MONITOR-001-34-WIDE',
    '34-inch Ultrawide',
    549.99
  );

-- Product 20 (Memory Foam Pillow) variants
INSERT INTO
  variants (id, product_id, sku, name, price)
VALUES
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380b10',
    'f47ac10b-58cc-4372-a567-0e02b2c3d420',
    'PILLOW-001-STD',
    'Standard',
    49.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380b11',
    'f47ac10b-58cc-4372-a567-0e02b2c3d420',
    'PILLOW-001-KING',
    'King Size',
    59.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380b12',
    'f47ac10b-58cc-4372-a567-0e02b2c3d420',
    'PILLOW-001-COOL',
    'Cooling Gel',
    69.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380b13',
    'f47ac10b-58cc-4372-a567-0e02b2c3d420',
    'PILLOW-001-CONTOUR',
    'Contour Support',
    54.99
  ),
  (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380b14',
    'f47ac10b-58cc-4372-a567-0e02b2c3d420',
    'PILLOW-001-TRAVEL',
    'Travel Size',
    29.99
  );
