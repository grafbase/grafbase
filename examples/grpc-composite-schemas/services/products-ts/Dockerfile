# Build stage
FROM node:20-alpine AS builder

# Install pnpm
RUN corepack enable && corepack prepare pnpm@latest --activate

WORKDIR /app

# Copy package files
COPY services/products-ts/package.json ./
COPY services/products-ts/pnpm-lock.yaml ./
COPY services/products-ts/tsconfig.json ./

# Install dependencies
RUN pnpm install --frozen-lockfile

# Copy source code
COPY services/products-ts/src ./src

# Copy proto files
COPY proto /proto

# Production stage
FROM node:20-alpine

# Install pnpm
RUN corepack enable && corepack prepare pnpm@latest --activate

WORKDIR /app

# Copy package files
COPY services/products-ts/package.json ./
COPY services/products-ts/pnpm-lock.yaml ./

# Install production dependencies only
RUN pnpm install --frozen-lockfile --prod

# Copy built application
COPY --from=builder /app/src ./src
COPY --from=builder /app/tsconfig.json ./
COPY --from=builder /proto /proto

# Expose gRPC port
EXPOSE 50051

# Run the service
CMD ["pnpm", "start"]