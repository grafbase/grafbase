FROM oven/bun:1.1.22-alpine
WORKDIR /grafbase
COPY . .

RUN bun install
CMD ["bun", "run", "/grafbase/src/main.js"]

EXPOSE 4092
