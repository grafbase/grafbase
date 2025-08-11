import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const PROTO_PATH = path.join(__dirname, '../../../proto/products.proto');

const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
  keepCase: true,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true,
  includeDirs: [path.join(__dirname, '../../../proto')]
});

const productsProto: any = grpc.loadPackageDefinition(packageDefinition).products;

// Hardcoded data with references to warehouse locations and parts
const products = [
  {
    id: 'prod-001',
    sku: 'SKU-BIKE-001',
    name: 'Mountain Bike Pro',
    description: 'Professional mountain bike with carbon fiber frame',
    price: 2499.99,
    warehouse_id: 'loc-001',  // References Seattle warehouse
    quantity_in_stock: 15,
    category: 'Bicycles'
  },
  {
    id: 'prod-002',
    sku: 'SKU-EBIKE-001',
    name: 'Electric City Cruiser',
    description: 'Electric bike for urban commuting',
    price: 1899.99,
    warehouse_id: 'loc-002',  // References Portland warehouse
    quantity_in_stock: 8,
    category: 'E-Bikes'
  },
  {
    id: 'prod-003',
    sku: 'SKU-ROAD-001',
    name: 'Speed Demon Road Bike',
    description: 'Lightweight road bike for racing',
    price: 3299.99,
    warehouse_id: 'loc-001',  // References Seattle warehouse
    quantity_in_stock: 5,
    category: 'Bicycles'
  }
];

// Product-part relationships with quantities
const productParts = [
  // Mountain Bike Pro parts
  { product_id: 'prod-001', part_id: 'part-001', quantity_required: 1 },  // Carbon fiber frame
  { product_id: 'prod-001', part_id: 'part-002', quantity_required: 2 },  // Wheels
  { product_id: 'prod-001', part_id: 'part-003', quantity_required: 1 },  // Derailleur
  { product_id: 'prod-001', part_id: 'part-004', quantity_required: 1 },  // Handlebars
  { product_id: 'prod-001', part_id: 'part-005', quantity_required: 2 },  // Brake sets
  
  // Electric City Cruiser parts
  { product_id: 'prod-002', part_id: 'part-006', quantity_required: 1 },  // Electric motor
  { product_id: 'prod-002', part_id: 'part-007', quantity_required: 1 },  // Battery pack
  { product_id: 'prod-002', part_id: 'part-002', quantity_required: 2 },  // Wheels (shared)
  { product_id: 'prod-002', part_id: 'part-004', quantity_required: 1 },  // Handlebars (shared)
  { product_id: 'prod-002', part_id: 'part-008', quantity_required: 1 },  // Display unit
  
  // Speed Demon Road Bike parts
  { product_id: 'prod-003', part_id: 'part-009', quantity_required: 1 },  // Lightweight frame
  { product_id: 'prod-003', part_id: 'part-010', quantity_required: 2 }, // Racing wheels
  { product_id: 'prod-003', part_id: 'part-003', quantity_required: 1 },  // Derailleur (shared)
  { product_id: 'prod-003', part_id: 'part-011', quantity_required: 1 }, // Drop handlebars
  { product_id: 'prod-003', part_id: 'part-005', quantity_required: 2 }  // Brake sets (shared)
];

// Service implementation
const productService = {
  GetProduct: (call: any, callback: any) => {
    const product = products.find(p => p.id === call.request.id);
    if (product) {
      callback(null, { product });
    } else {
      callback({
        code: grpc.status.NOT_FOUND,
        details: `Product with id ${call.request.id} not found`
      });
    }
  },

  BatchGetProducts: (call: any, callback: any) => {
    const requestedIds = call.request.ids || [];
    const foundProducts = requestedIds
      .map(id => products.find(p => p.id === id))
      .filter(p => p !== undefined);
    callback(null, { products: foundProducts });
  },

  GetProductParts: (call: any, callback: any) => {
    const parts = productParts.filter(pp => pp.product_id === call.request.product_id);
    callback(null, { product_parts: parts });
  },

  GetProductsForPart: (call: any, callback: any) => {
    const parts = productParts.filter(pp => pp.part_id === call.request.part_id);
    callback(null, { product_parts: parts });
  },

  SearchProducts: (call: any, callback: any) => {
    let filteredProducts = [...products];
    const filters = call.request;

    // Apply filters if provided
    if (filters.name) {
      filteredProducts = filteredProducts.filter(p => 
        p.name.toLowerCase().includes(filters.name.toLowerCase())
      );
    }

    if (filters.sku) {
      filteredProducts = filteredProducts.filter(p => 
        p.sku.toLowerCase().includes(filters.sku.toLowerCase())
      );
    }

    if (filters.category) {
      filteredProducts = filteredProducts.filter(p => 
        p.category.toLowerCase() === filters.category.toLowerCase()
      );
    }

    if (filters.warehouse_id) {
      filteredProducts = filteredProducts.filter(p => 
        p.warehouse_id === filters.warehouse_id
      );
    }

    if (filters.min_price !== undefined && filters.min_price > 0) {
      filteredProducts = filteredProducts.filter(p => 
        p.price >= filters.min_price
      );
    }

    if (filters.max_price !== undefined && filters.max_price > 0) {
      filteredProducts = filteredProducts.filter(p => 
        p.price <= filters.max_price
      );
    }

    if (filters.min_quantity !== undefined && filters.min_quantity > 0) {
      filteredProducts = filteredProducts.filter(p => 
        p.quantity_in_stock >= filters.min_quantity
      );
    }

    callback(null, { products: filteredProducts });
  }
};

// Start the server
const server = new grpc.Server();
server.addService(productsProto.ProductService.service, productService);

const PORT = process.env.PORT || '50051';
server.bindAsync(
  `0.0.0.0:${PORT}`,
  grpc.ServerCredentials.createInsecure(),
  (err, port) => {
    if (err) {
      console.error('Failed to start server:', err);
      return;
    }
    console.log(`Products service (TypeScript) running on port ${port}`);
    server.start();
  }
);