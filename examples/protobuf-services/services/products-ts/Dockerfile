# Build stage
FROM node:20-alpine AS builder

WORKDIR /app

# Copy package files
COPY services/products-ts/package*.json ./
COPY services/products-ts/tsconfig.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY services/products-ts/src ./src

# Copy proto files
COPY proto /proto

# Production stage
FROM node:20-alpine

WORKDIR /app

# Copy package files
COPY services/products-ts/package*.json ./

# Install production dependencies only
RUN npm ci --only=production

# Copy built application
COPY --from=builder /app/src ./src
COPY --from=builder /app/tsconfig.json ./
COPY --from=builder /proto /proto

# Expose gRPC port
EXPOSE 50051

# Run the service
CMD ["npm", "start"]