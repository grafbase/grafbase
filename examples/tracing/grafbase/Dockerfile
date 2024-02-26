FROM oven/bun:alpine

RUN apk update && apk add npm
RUN npm install -g grafbase

WORKDIR /grafbase/tracing

COPY schema.graphql .
COPY resolvers resolvers

ENTRYPOINT ["gb"]
CMD ["gb"]

