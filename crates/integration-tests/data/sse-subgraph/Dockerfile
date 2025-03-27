FROM oven/bun:1.2.7-alpine

RUN adduser -g wheel -D grafbase -h "/grafbase" && mkdir -p /grafbase && chown grafbase: /grafbase
USER grafbase

WORKDIR /grafbase
COPY . .

RUN bun install
CMD ["bun", "run", "/grafbase/src/main.js"]

EXPOSE 4092
